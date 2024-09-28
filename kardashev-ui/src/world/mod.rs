use std::{
    future::Future,
    pin::Pin,
    time::Duration,
};

use hecs::EntityBuilder;
use tokio::{
    sync::{
        mpsc,
        oneshot,
    },
    time::Interval,
};

use crate::{
    error::Error,
    graphics::rendering_system::RenderingSystem,
    utils::spawn_local_and_handle_error,
};

#[derive(Clone, Debug)]
pub struct World {
    tx_command: mpsc::Sender<Command>,
}

impl World {
    pub fn new() -> Self {
        let (tx_command, rx_command) = mpsc::channel(16);

        let scheduler = Scheduler {
            update_system: Box::new(NullSystem),
            rendering_system: Box::new(RenderingSystem),
            update_interval: tokio::time::interval(Duration::from_millis(1000 / 60)),
            render_interval: tokio::time::interval(Duration::from_millis(1000 / 60)),
        };

        spawn_local_and_handle_error(async move {
            let server = WorldServer::new(rx_command, scheduler);
            server.run().await
        });

        Self { tx_command }
    }

    async fn send_command(&self, command: Command) {
        self.tx_command
            .send(command)
            .await
            .expect("world server died");
    }

    pub async fn submit(&self, command_buffer: hecs::CommandBuffer) {
        self.send_command(Command::SubmitCommandBuffer { command_buffer });
    }

    pub async fn spawn(&self, bundle: impl hecs::DynamicBundle) -> hecs::Entity {
        let mut builder = hecs::EntityBuilder::new();
        builder.add_bundle(bundle);
        let (tx_entity, rx_entity) = oneshot::channel();
        self.send_command(Command::SpawnEntity { builder, tx_entity })
            .await;
        rx_entity.await.unwrap()
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
}

struct WorldServer {
    rx_command: mpsc::Receiver<Command>,
    world: hecs::World,
    scheduler: Scheduler,
}

impl WorldServer {
    fn new(rx_command: mpsc::Receiver<Command>, scheduler: Scheduler) -> Self {
        Self {
            rx_command,
            world: hecs::World::new(),
            scheduler,
        }
    }

    async fn run(mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
                command_opt = self.rx_command.recv() => {
                    let Some(command) = command_opt else { break; };
                    self.handle_command(command).await?;
                }
                _ = self.scheduler.update_interval.tick() => {
                    self.scheduler.update_system.run(&mut self.world).await?;
                }
                _ = self.scheduler.render_interval.tick() => {
                    self.scheduler.rendering_system.run(&mut self.world).await?;
                }
            }
        }

        while let Some(command) = self.rx_command.recv().await {
            self.handle_command(command).await?;
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
                tx_entity.send(entity);
            }
        }

        Ok(())
    }
}

pub trait System {
    async fn run(&mut self, world: &mut hecs::World) -> Result<(), Error>;
}

trait DynSystem {
    fn run<'a>(
        &'a mut self,
        world: &'a mut hecs::World,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
}

impl<T: System> DynSystem for T {
    fn run<'a>(
        &'a mut self,
        world: &'a mut hecs::World,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(T::run(self, world))
    }
}

pub struct Scheduler {
    update_system: Box<dyn DynSystem>,
    rendering_system: Box<dyn DynSystem>,
    update_interval: Interval,
    render_interval: Interval,
}

pub struct NullSystem;

impl System for NullSystem {
    async fn run(&mut self, world: &mut hecs::World) -> Result<(), Error> {
        Ok(())
    }
}
