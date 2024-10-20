use std::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    time::Duration,
};

use futures::FutureExt;
use hecs::{
    Entity,
    Query,
    QueryOneError,
};
use tokio::sync::{
    mpsc,
    oneshot,
};

use crate::{
    ecs::{
        resource::Resources,
        schedule::Schedule,
        system::{
            DynSystem,
            System,
            SystemContext,
        },
        Error,
        Plugin,
        RegisterPluginContext,
    },
    utils::{
        futures::spawn_local_and_handle_error,
        time::{
            interval,
            Interval,
        },
    },
};

pub struct Builder {
    world: hecs::World,
    resources: Resources,
    startup_schedule: Schedule,
    schedule: Schedule,
    tps: u64,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            world: hecs::World::default(),
            resources: Resources::default(),
            startup_schedule: Schedule::default(),
            schedule: Schedule::default(),
            tps: 60,
        }
    }
}

impl Builder {
    pub fn add_resource<R: 'static>(&mut self, resource: R) {
        self.resources.insert(resource);
    }

    pub fn with_resource<R: 'static>(mut self, resource: R) -> Self {
        self.add_resource(resource);
        self
    }

    pub fn add_startup_system(&mut self, system: impl System) {
        self.startup_schedule.add_system(system);
    }

    pub fn with_startup_system(mut self, system: impl System) -> Self {
        self.startup_schedule.add_system(system);
        self
    }

    pub fn add_system(&mut self, system: impl System) {
        self.schedule.add_system(system);
    }

    pub fn with_system(mut self, system: impl System) -> Self {
        self.schedule.add_system(system);
        self
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        plugin.register(RegisterPluginContext {
            resources: &mut self.resources,
            startup_schedule: &mut self.startup_schedule,
            schedule: &mut self.schedule,
        });
    }

    pub fn with_plugin(mut self, plugin: impl Plugin) -> Self {
        self.add_plugin(plugin);
        self
    }

    pub fn with_tps(mut self, tps: u64) -> Self {
        self.tps = tps;
        self
    }

    pub fn build(self) -> WorldServer {
        let (tx_command, rx_command) = mpsc::unbounded_channel();

        spawn_local_and_handle_error(async move {
            let server = Reactor {
                rx_command,
                world: self.world,
                resources: self.resources,
                startup_schedule: self.startup_schedule,
                schedule: self.schedule,
                command_buffer: hecs::CommandBuffer::new(),
                tick: interval(Duration::from_millis(1000 / self.tps)),
            };
            server.run().await
        });

        WorldServer { tx_command }
    }
}

#[derive(Clone, Debug)]
pub struct WorldServer {
    tx_command: mpsc::UnboundedSender<Command>,
}

impl WorldServer {
    pub fn builder() -> Builder {
        Builder::default()
    }

    fn send_command(&self, command: Command) {
        self.tx_command.send(command).expect("world server died");
    }

    /// Submits an ECS command buffer to be executed
    pub fn submit(&self, command_buffer: hecs::CommandBuffer) {
        self.send_command(Command::SubmitCommandBuffer { command_buffer })
    }

    /// Spawns an entity
    pub async fn spawn_entity(&self, bundle: impl hecs::DynamicBundle) -> hecs::Entity {
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(bundle);
        let (tx_entity, rx_entity) = oneshot::channel();
        self.send_command(Command::SpawnEntity { builder, tx_entity });
        rx_entity.await.unwrap()
    }

    /// Despawns an entity
    pub fn despawn_entity(&self, entity: Entity) {
        self.send_command(Command::DespawnEntity { entity });
    }

    /// Runs a system.
    ///
    /// This system will be executed immediately. No other systems are executed,
    /// or other commands handles until this system finishes.
    pub fn run_system_now(&self, system: impl System) {
        self.send_command(Command::RunSystem {
            system: system.dyn_system(),
        })
    }

    /// Runs a closure once with system context.
    pub fn run<F, R>(&self, f: F) -> RunOnce<R>
    where
        F: FnOnce(&mut SystemContext) -> R + Send + Sync + 'static,
        R: Send + Sync + 'static,
    {
        let (tx_result, rx_result) = oneshot::channel();
        self.send_command(Command::RunOnce {
            f: Box::new(move |system_context| {
                let result = f(system_context);
                let _ = tx_result.send(result);
            }),
        });
        RunOnce { rx_result }
    }

    pub fn run_on_entity<F, R, Q>(&self, entity: Entity, f: F) -> RunOnce<Result<R, QueryOneError>>
    where
        F: for<'q> FnOnce(Q::Item<'q>) -> R + Send + Sync + 'static,
        R: Send + Sync + 'static,
        Q: Query,
    {
        self.run(move |system_context| {
            let query = system_context.world.query_one_mut::<Q>(entity)?;
            Ok(f(query))
        })
    }
}

enum Command {
    SubmitCommandBuffer {
        command_buffer: hecs::CommandBuffer,
    },
    SpawnEntity {
        builder: hecs::EntityBuilder,
        tx_entity: oneshot::Sender<hecs::Entity>,
    },
    DespawnEntity {
        entity: Entity,
    },
    RunSystem {
        system: DynSystem,
    },
    RunOnce {
        f: Box<dyn FnOnce(&mut SystemContext)>,
    },
}

struct Reactor {
    rx_command: mpsc::UnboundedReceiver<Command>,
    world: hecs::World,
    resources: Resources,
    startup_schedule: Schedule,
    schedule: Schedule,
    command_buffer: hecs::CommandBuffer,
    tick: Interval,
}

impl Reactor {
    async fn run(mut self) -> Result<(), Error> {
        let mut system_context = SystemContext {
            world: &mut self.world,
            resources: &mut self.resources,
            command_buffer: &mut self.command_buffer,
            tick: Tick(0),
        };

        tracing::debug!("running startup systems");
        self.startup_schedule.poll_system(&mut system_context)?;
        system_context.apply_buffered();
        drop(self.startup_schedule);

        tracing::debug!("running systems");
        loop {
            system_context.tick.0 += 1;

            tokio::select! {
                command_opt = self.rx_command.recv() => {
                    let Some(command) = command_opt else { break; };
                    match command {
                        Command::SubmitCommandBuffer { mut command_buffer } => {
                            command_buffer.run_on(&mut system_context.world);
                        }
                        Command::SpawnEntity {
                            mut builder,
                            tx_entity,
                        } => {
                            let entity = system_context.world.spawn(builder.build());
                            let _ = tx_entity.send(entity);
                        }
                        Command::DespawnEntity { entity } => {
                            let _ = system_context.world.despawn(entity);
                        }
                        Command::RunSystem { mut system } => {
                            system.poll_system(&mut system_context)
                                .map_err(|error| Error::System { system: system.label(), error })?;
                        }
                        Command::RunOnce { f } => {
                            f(&mut system_context);
                        }
                    }
                }
                _ = self.tick.tick() => {
                    self.schedule.poll_system(&mut system_context)?;
                    system_context.apply_buffered();
                }
            }
        }

        tracing::debug!("system server dropped");

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(u64);

#[derive(Debug)]
pub struct RunOnce<R> {
    rx_result: oneshot::Receiver<R>,
}

impl<R> Future for RunOnce<R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.rx_result
            .poll_unpin(cx)
            .map(|result| result.expect("world server died while running the run-once"))
    }
}
