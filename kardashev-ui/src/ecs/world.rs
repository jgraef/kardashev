use std::{
    any::TypeId, collections::{
        hash_map,
        HashMap,
    }, fmt::Debug, marker::PhantomData, ops::Range, task::{
        Context,
        Poll,
    }
};

use futures::FutureExt;
use hecs::{BuiltEntity, EntityBuilder};
pub use hecs::{
    Bundle,
    Component,
    ComponentError,
    DynamicBundle,
    Entity,
    EntityRef,
    NoSuchEntity,
    Query,
    QueryBorrow,
    QueryMut,
    QueryOne,
    QueryOneError,
    TakenEntity,
    View,
    ViewBorrow,
};
use tokio::sync::broadcast;

#[derive(Default)]
pub struct World {
    world: hecs::World,
    sender: EventSender,
}

impl World {
    const CHANNEL_CAPACITY: usize = 128;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, bundle: impl DynamicBundle) -> Entity {
        let entity = self.world.reserve_entity();
        self.spawn_at(entity, bundle);
        entity
    }

    pub fn spawn_at(&mut self, entity: Entity, bundle: impl DynamicBundle) {
        self.sender
            .put_dynamic_bundle(&bundle, Change::Added { entity });
        self.world.spawn_at(entity, bundle);
        self.sender.flush();
    }

    pub fn reserve_entity(&mut self) -> Entity {
        self.world.reserve_entity()
    }

    pub fn despawn(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.take(entity)?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.world.clear();
    }

    pub fn contains(&mut self, entity: Entity) -> bool {
        self.world.contains(entity)
    }

    pub fn take(&mut self, entity: Entity) -> Result<TakenEntity, NoSuchEntity> {
        let taken = self.world.take(entity)?;
        self.sender
            .put_dynamic_bundle(&taken, Change::Removed { entity });
        self.sender.flush();
        Ok(taken)
    }

    pub fn query<Q: Query>(&self) -> QueryBorrow<Q> {
        self.world.query::<Q>()
    }

    pub fn query_mut<Q: Query>(&mut self) -> QueryMut<Q> {
        self.world.query_mut::<Q>()
    }

    pub fn query_one<Q: Query>(&self, entity: Entity) -> Result<QueryOne<Q>, NoSuchEntity> {
        self.world.query_one::<Q>(entity)
    }

    pub fn query_one_mut<Q: Query>(
        &mut self,
        entity: Entity,
    ) -> Result<Q::Item<'_>, QueryOneError> {
        self.world.query_one_mut::<Q>(entity)
    }

    pub fn query_many_mut<Q: Query, const N: usize>(
        &mut self,
        entities: [Entity; N],
    ) -> [Result<Q::Item<'_>, QueryOneError>; N] {
        self.world.query_many_mut::<Q, N>(entities)
    }

    pub fn view<Q: Query>(&self) -> ViewBorrow<Q> {
        self.world.view::<Q>()
    }

    pub fn view_mut<Q: Query>(&mut self) -> View<Q> {
        self.world.view_mut::<Q>()
    }

    pub fn satisfies<Q: Query>(&self, entity: Entity) -> Result<bool, NoSuchEntity> {
        self.world.satisfies::<Q>(entity)
    }

    pub fn entity(&self, entity: Entity) -> Result<EntityRef, NoSuchEntity> {
        self.world.entity(entity)
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        bundle: impl DynamicBundle,
    ) -> Result<(), NoSuchEntity> {
        self.sender
            .put_dynamic_bundle(&bundle, Change::Added { entity });
        self.sender
            .flush_or_clear(|| self.world.insert(entity, bundle))
    }

    pub fn insert_one(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) -> Result<(), NoSuchEntity> {
        self.insert(entity, (component,))
    }

    pub fn remove<T: Bundle + 'static>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.sender.put_bundle::<T>(Change::Removed { entity });
        self.sender
            .flush_or_clear(|| self.world.remove::<T>(entity))
    }

    pub fn remove_one<T: Component>(&mut self, entity: Entity) -> Result<T, ComponentError> {
        self.remove::<(T,)>(entity).map(|(x,)| x)
    }

    pub fn changes<T: 'static>(&mut self) -> Changes<T> {
        let rx = match self.sender.queues.entry(TypeId::of::<T>()) {
            hash_map::Entry::Occupied(occupied) => occupied.get().subscribe(),
            hash_map::Entry::Vacant(vacant) => {
                let (tx, rx) = broadcast::channel(Self::CHANNEL_CAPACITY);
                vacant.insert(tx);
                rx
            }
        };

        Changes {
            rx,
            _ty: PhantomData,
        }
    }
}

impl Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("sender", &self.sender)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct Changes<T> {
    rx: broadcast::Receiver<Change>,
    _ty: PhantomData<T>,
}

impl<T> Changes<T> {
    pub async fn next(&mut self) -> Option<Change> {
        self.rx
            .recv()
            .await
            .or_else(|error| {
                match error {
                    broadcast::error::RecvError::Closed => Err(()),
                    broadcast::error::RecvError::Lagged(_) => Ok(Change::MissedEvents),
                }
            })
            .ok()
    }

    pub fn poll_next(&mut self, cx: &mut Context) -> Poll<Option<Change>> {
        std::pin::pin!(self.next()).poll_unpin(cx)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Change {
    Added { entity: Entity },
    Removed { entity: Entity },
    MissedEvents,
}

#[derive(Debug, Default)]
struct EventSender {
    queues: HashMap<TypeId, broadcast::Sender<Change>>,
    buffer: EventBuffer,
}

impl EventSender {
    fn put_type_ids(&mut self, type_ids: &[TypeId], event: Change) {
        self.buffer.put_type_ids(type_ids, change)
    }

    fn put_dynamic_bundle(&mut self, bundle: &impl DynamicBundle, event: Change) {
        bundle.with_ids(|type_ids| {
            self.put_type_ids(type_ids, event);
        });
    }

    fn put_bundle<T: Bundle + 'static>(&mut self, event: Change) {
        T::with_static_ids(|type_ids| {
            self.put_type_ids(type_ids, event);
        })
    }

    fn flush(&mut self) {
        // todo: can we batch these somehow without allocating tons of Vecs?
        for (type_id, event) in self.buffer.drain(..) {
            match self.queues.entry(type_id) {
                hash_map::Entry::Occupied(mut occupied_entry) => {
                    if occupied_entry.get_mut().send(event).is_err() {
                        occupied_entry.remove();
                    }
                }
                hash_map::Entry::Vacant(_vacant_entry) => {}
            }
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn flush_or_clear<F, T, E>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        let result = f();
        match &result {
            Ok(_) => self.flush(),
            Err(_) => self.clear(),
        }
        result
    }
}

#[derive(Debug, Default)]
struct EventBuffer {
    buffer: Vec<(TypeId, Change)>,
}

impl EventBuffer {
    fn put_type_ids(&mut self, type_ids: &[TypeId], change: Change) -> usize {
        let rollback_index = self.buffer.len();
        for type_id in type_ids {
                self.buffer.push((*type_id, change));
        }
        rollback_index
    }

    fn put_dynamic_bundle(&mut self, bundle: &impl DynamicBundle, change: Change) -> usize {
        bundle.with_ids(|type_ids| {
            self.put_type_ids(type_ids, change)
        })
    }

    fn put_bundle<T: Bundle + 'static>(&mut self, change: Change) -> usize {
        T::with_static_ids(|type_ids| {
            self.put_type_ids(type_ids, change)
        })
    }

    fn clear(&mut self) {
        self.buffer.clear();
    }

    fn rollback(mut self, rollback_index: usize) {
        self.buffer.resize_with(rollback_index, || panic!("didn't expect buffer to grow"));
    }
}

#[derive(Default)]
pub struct CommandBuffer {
    inner: hecs::CommandBuffer,
    commands: Vec<Command>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(
        &mut self,
        entity: Entity,
        bundle: impl DynamicBundle,
    ) {
        self.commands.push(Command::BeginInsert { entity });
        bundle.with_ids(|type_ids| {
            for type_id in type_ids {
                self.commands.push(Command::Component { type_id: *type_id, contains: |entity, world| world.satisfies::<&>(entity) });
            }
        });
        self.commands.push(Command::End);
        self.inner.insert(entity, bundle);
    }

    pub fn insert_one(
        &mut self,
        entity: Entity,
        component: impl Component,
    ) {
        self.insert(entity, (component,))
    }

    pub fn remove<T: Bundle + 'static>(&mut self, entity: Entity) {
        self.commands.push(Command::BeginRemove { entity });
        T::with_static_ids(|type_ids| {
            for type_id in type_ids {
                self.commands.push(Command::Component { type_id: *type_id });
            }
        });
        self.commands.push(Command::End);
        self.inner.remove::<T>(entity);
    }

    pub fn remove_one<T: Component>(&mut self, entity: Entity) {
        self.remove::<(T,)>(entity);
    }


    pub fn despawn(&mut self, entity: Entity) {
        self.commands.push(Command::Despawn { entity });
    }

    pub fn spawn(&mut self, bundle: impl DynamicBundle) {
        let mut builder = EntityBuilder::new();
        builder.add_bundle(bundle);
        self.commands.push(Command::Spawn { builder });
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.commands.clear();
    }

    pub fn run_on(&mut self, world: &mut World) {
        fn advance_to_end<'a>(commands: &mut std::slice::IterMut<'a, Command>) {
            while let Some(command) = commands.next() {
                if let Command::End = command {
                    break;
                }
            }
        }

        let mut commands = self.commands.iter_mut();

        while let Some(command) = commands.next() {
            match command {
                Command::BeginRemove { entity } => {
                    if world.contains(*entity) {
                        while let Some(command) = commands.next() {
                            match command {
                                Command::Component { type_id } => {
                                    // how do we check if the entity contains this type id?
                                }
                                Command::End => break,
                                _ => {},
                            }
                        }
                    }
                    else {
                        advance_to_end(&mut commands);
                    }
                },
                Command::BeginInsert { entity } => {
                    if world.contains(*entity) {
                        while let Some(command) = commands.next() {
                            match command {
                                Command::Component { type_id } => {
                                    // how do we check if the entity contains this type id?
                                }
                                Command::End => break,
                                _ => {},
                            }
                        }
                    }
                    else {
                        advance_to_end(&mut commands);
                    }
                }
                Command::Spawn { builder } => {

                }
            }
        }
    }

}

impl Debug for CommandBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandBuffer").field("commands", &self.commands).finish_non_exhaustive()
    }
}

enum Command {
    Nop,
    BeginInsert { entity: Entity },
    BeginRemove { entity: Entity },
    Component { type_id: TypeId, contains: fn(Entity, &mut hecs::World) -> bool },
    End,
    Spawn { builder: EntityBuilder },
    Despawn { entity: Entity },
}