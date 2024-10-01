use hecs::Entity;
use tokio::sync::{
    mpsc,
    oneshot,
};

use super::{
    resource::Resources,
    schedule::Scheduler,
    system::{
        DynOneshotSystem,
        OneshotSystem,
        System,
    },
    Plugin,
    RegisterPluginContext,
};
use crate::{
    error::Error,
    utils::spawn_local_and_handle_error,
    world::system::RunSystemContext,
};

#[derive(Default)]
pub struct Builder {
    world: hecs::World,
    resources: Resources,
    scheduler: Scheduler,
}

impl Builder {
    pub fn add_resource<R: 'static>(&mut self, resource: R) {
        self.resources.insert(resource);
    }

    pub fn with_resource<R: 'static>(mut self, resource: R) -> Self {
        self.add_resource(resource);
        self
    }

    pub fn add_startup_system(&mut self, system: impl OneshotSystem) {
        self.scheduler.add_startup_system(system);
    }

    pub fn with_startup_system(mut self, system: impl OneshotSystem) -> Self {
        self.scheduler.add_startup_system(system);
        self
    }

    pub fn add_update_system(&mut self, system: impl System) {
        self.scheduler.add_update_system(system);
    }

    pub fn with_update_system(mut self, system: impl System) -> Self {
        self.scheduler.add_update_system(system);
        self
    }

    pub fn add_render_system(&mut self, system: impl System) {
        self.scheduler.add_render_system(system);
    }

    pub fn with_render_system(mut self, system: impl System) -> Self {
        self.scheduler.add_render_system(system);
        self
    }

    pub fn add_plugin(&mut self, plugin: impl Plugin) {
        plugin.register(RegisterPluginContext {
            world: &mut self.world,
            resources: &mut self.resources,
            scheduler: &mut self.scheduler,
        });
    }

    pub fn with_plugin(mut self, plugin: impl Plugin) -> Self {
        self.add_plugin(plugin);
        self
    }

    pub fn build(self) -> World {
        let (tx_command, rx_command) = mpsc::channel(16);

        spawn_local_and_handle_error(async move {
            let server = WorldServer::new(rx_command, self);
            server.run().await
        });

        World { tx_command }
    }
}

#[derive(Clone, Debug)]
pub struct World {
    tx_command: mpsc::Sender<Command>,
}

impl World {
    pub fn builder() -> Builder {
        Builder::default()
    }

    async fn send_command(&self, command: Command) {
        self.tx_command
            .send(command)
            .await
            .expect("world server died");
    }

    pub async fn submit(&self, command_buffer: hecs::CommandBuffer) {
        self.send_command(Command::SubmitCommandBuffer { command_buffer })
            .await;
    }

    pub async fn spawn(&self, bundle: impl hecs::DynamicBundle) -> hecs::Entity {
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(bundle);
        let (tx_entity, rx_entity) = oneshot::channel();
        self.send_command(Command::SpawnEntity { builder, tx_entity })
            .await;
        rx_entity.await.unwrap()
    }

    pub async fn despawn(&self, entity: Entity) {
        self.send_command(Command::DespawnEntity { entity }).await;
    }

    pub async fn run_oneshot_system(&self, system: impl OneshotSystem) {
        self.send_command(Command::RunOneshotSystem {
            system: Box::new(system),
        })
        .await;
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
    RunOneshotSystem {
        system: Box<dyn DynOneshotSystem>,
    },
}

struct WorldServer {
    rx_command: mpsc::Receiver<Command>,
    world: hecs::World,
    resources: Resources,
    scheduler: Scheduler,
}

impl WorldServer {
    fn new(rx_command: mpsc::Receiver<Command>, builder: Builder) -> Self {
        Self {
            rx_command,
            world: builder.world,
            resources: builder.resources,
            scheduler: builder.scheduler,
        }
    }

    async fn run(mut self) -> Result<(), Error> {
        let mut context = RunSystemContext {
            command_buffer: hecs::CommandBuffer::new(),
            world: &mut self.world,
            resources: &mut self.resources,
        };
        for system in std::mem::take(&mut self.scheduler.startup_systems) {
            tracing::debug!(label = %system.label(), "running startup system");
            system.run(&mut context).await?;
        }
        context.apply_buffered();

        loop {
            tokio::select! {
                command_opt = self.rx_command.recv() => {
                    let Some(command) = command_opt else { break; };
                    self.handle_command(command).await?;
                }
                _ = self.scheduler.update_systems.wait() => {
                    let mut context = RunSystemContext {
                        command_buffer: hecs::CommandBuffer::new(),
                        world: &mut self.world,
                        resources: &mut self.resources,
                    };
                    self.scheduler.update_systems.run(&mut context).await?;
                    context.apply_buffered();
                }
                _ = self.scheduler.render_systems.wait() => {
                    let mut context = RunSystemContext {
                        command_buffer: hecs::CommandBuffer::new(),
                        world: &mut self.world,
                        resources: &mut self.resources,
                    };
                    self.scheduler.render_systems.run(&mut context).await?;
                    context.apply_buffered();
                }
            }
        }

        Ok(())
    }

    async fn handle_command(&mut self, command: Command) -> Result<(), Error> {
        match command {
            Command::SubmitCommandBuffer { mut command_buffer } => {
                command_buffer.run_on(&mut self.world);
            }
            Command::SpawnEntity {
                mut builder,
                tx_entity,
            } => {
                let entity = self.world.spawn(builder.build());
                let _ = tx_entity.send(entity);
            }
            Command::DespawnEntity { entity } => {
                self.world.despawn(entity);
            }
            Command::RunOneshotSystem { system } => {
                let mut context = RunSystemContext {
                    command_buffer: hecs::CommandBuffer::new(),
                    world: &mut self.world,
                    resources: &mut self.resources,
                };
                system.run(&mut context).await?;
                context.apply_buffered();
            }
        }

        Ok(())
    }
}
