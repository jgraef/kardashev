use std::task::{
    ready,
    Poll,
};

use hecs::{Entity, NoSuchEntity, Query, QueryOne, QueryOneError};
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
        world::World,
        Error,
        Plugin,
        RegisterPluginContext,
    },
    utils::futures::spawn_local_and_handle_error,
};

#[derive(Default)]
pub struct Builder {
    world: World,
    resources: Resources,
    startup_schedule: Schedule,
    schedule: Schedule,
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
            world: &mut self.world,
            resources: &mut self.resources,
            startup_schedule: &mut self.startup_schedule,
            schedule: &mut self.schedule,
        });
    }

    pub fn with_plugin(mut self, plugin: impl Plugin) -> Self {
        self.add_plugin(plugin);
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

    /// Spawns a system.
    ///
    /// This system will be run in the system schedule.
    pub fn spawn_system(&self, system: impl System) {
        self.send_command(Command::AddSystem {
            system: system.dyn_system(),
        });
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
    pub async fn run<F, R>(&self, f: F) -> R
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
        rx_result
            .await
            .expect("world server died while running the run-once")
    }

    pub async fn run_on_entity<F, R, Q>(&self, entity: Entity, f: F) -> Result<R, QueryOneError>
        where
            F: for<'q> FnOnce(Q::Item<'q>) -> R + Send + Sync + 'static,
            R: Send + Sync + 'static,
            Q: Query,
    {
        self.run(move |system_context| {
            let query = system_context.world.query_one_mut::<Q>(entity)?;
            Ok(f(query))
        }).await
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
    AddSystem {
        system: DynSystem,
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
    world: World,
    resources: Resources,
    startup_schedule: Schedule,
    schedule: Schedule,
}

impl Reactor {
    async fn run(mut self) -> Result<(), Error> {
        /// polls the HandleCommandsSystem first, then the given schedule
        fn poll_systems(
            handle_commands_system: &mut HandleCommandsSystem,
            schedule: &mut Schedule,
            task_context: &mut std::task::Context,
            system_context: &mut SystemContext,
        ) -> Poll<Result<(), Error>> {
            match handle_commands_system.poll_system(task_context, system_context) {
                Poll::Pending => {}
                Poll::Ready(result) => return Poll::Ready(result),
            }

            match schedule.poll_system(task_context, system_context) {
                Poll::Pending => {}
                Poll::Ready(result) => return Poll::Ready(result),
            }

            system_context.apply_buffered();
            for system in system_context.add_systems.drain(..) {
                schedule.add_system(system);
            }

            Poll::Pending
        }

        async fn run_systems(
            handle_commands_system: &mut HandleCommandsSystem,
            schedule: &mut Schedule,
            system_context: &mut SystemContext<'_>,
        ) -> Result<(), Error> {
            std::future::poll_fn(|task_context| {
                poll_systems(
                    handle_commands_system,
                    schedule,
                    task_context,
                    system_context,
                )
            })
            .await
        }

        let mut system_context = SystemContext {
            world: &mut self.world,
            resources: &mut self.resources,
            command_buffer: hecs::CommandBuffer::new(),
            add_systems: vec![],
        };

        let mut handle_commands_system = HandleCommandsSystem {
            rx_command: self.rx_command,
            running_system: None,
        };

        tracing::debug!("running startup systems");
        run_systems(
            &mut handle_commands_system,
            &mut self.startup_schedule,
            &mut system_context,
        )
        .await?;

        tracing::debug!("running systems");
        run_systems(
            &mut handle_commands_system,
            &mut self.schedule,
            &mut system_context,
        )
        .await?;

        tracing::debug!("all systems finished");

        Ok(())
    }
}

struct HandleCommandsSystem {
    rx_command: mpsc::UnboundedReceiver<Command>,
    running_system: Option<DynSystem>,
}

impl System for HandleCommandsSystem {
    type Error = Error;

    fn label(&self) -> &'static str {
        "handle-commands"
    }

    fn poll_system(
        &mut self,
        task_context: &mut std::task::Context<'_>,
        system_context: &mut SystemContext<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        loop {
            if let Some(running_system) = &mut self.running_system {
                ready!(running_system.poll_system(task_context, system_context)).map_err(
                    |error| {
                        Error::System {
                            system: running_system.label(),
                            error,
                        }
                    },
                )?;
                self.running_system = None;
            }

            if let Some(command) = ready!(self.rx_command.poll_recv(task_context)) {
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
                    Command::AddSystem { system } => {
                        system_context.add_system(system);
                    }
                    Command::RunSystem { system } => {
                        self.running_system = Some(system);
                    }
                    Command::RunOnce { f } => {
                        f(system_context);
                    }
                }
            }
            else {
                break;
            }
        }

        Poll::Ready(Ok(()))
    }
}
