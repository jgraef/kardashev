use std::task::{
    ready,
    Poll,
};

use hecs::Entity;
use tokio::sync::{
    mpsc,
    oneshot,
};

use crate::{
    utils::futures::spawn_local_and_handle_error,
    world::{
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
};

#[derive(Default)]
pub struct Builder {
    world: hecs::World,
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

    pub fn build(self) -> World {
        let (tx_command, rx_command) = mpsc::unbounded_channel();

        spawn_local_and_handle_error(async move {
            let server = WorldServer {
                rx_command,
                world: self.world,
                resources: self.resources,
                startup_schedule: self.startup_schedule,
                schedule: self.schedule,
            };
            server.run().await
        });

        World { tx_command }
    }
}

#[derive(Clone, Debug)]
pub struct World {
    tx_command: mpsc::UnboundedSender<Command>,
}

impl World {
    pub fn builder() -> Builder {
        Builder::default()
    }

    fn send_command(&self, command: Command) {
        self.tx_command.send(command).expect("world server died");
    }

    pub fn submit(&self, command_buffer: hecs::CommandBuffer) {
        self.send_command(Command::SubmitCommandBuffer { command_buffer })
    }

    pub async fn spawn(&self, bundle: impl hecs::DynamicBundle) -> hecs::Entity {
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(bundle);
        let (tx_entity, rx_entity) = oneshot::channel();
        self.send_command(Command::SpawnEntity { builder, tx_entity });
        rx_entity.await.unwrap()
    }

    pub fn despawn(&self, entity: Entity) {
        self.send_command(Command::DespawnEntity { entity });
    }

    pub fn add_system(&self, system: impl System) {
        self.send_command(Command::AddSystem {
            system: system.dyn_system(),
        });
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
}

struct WorldServer {
    rx_command: mpsc::UnboundedReceiver<Command>,
    world: hecs::World,
    resources: Resources,
    startup_schedule: Schedule,
    schedule: Schedule,
}

impl WorldServer {
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
        while let Some(command) = ready!(self.rx_command.poll_recv(task_context)) {
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
            }
        }

        Poll::Ready(Ok(()))
    }
}
