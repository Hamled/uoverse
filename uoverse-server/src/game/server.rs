use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use tokio::sync::mpsc;
use tracing::{debug, info, trace_span, trace};
use ultimaonline_net::{
    error::{Error, Result},
    packets::movement,
    types::{Direction, Serial},
};

use crate::game::client;

use super::client::{Client, ClientReceiver, ClientSender, WorldClient};

struct World {
    mob_x: u16,
    mob_dir: Direction,
}

pub struct Server {
    shutdown: AtomicBool,
    clients: Mutex<Vec<WorldClient>>,
    world: Mutex<World>,
}

const PLAYER_SERIAL: Serial = 3833;

impl Server {
    pub fn new() -> Self {
        Server {
            shutdown: AtomicBool::new(false),
            clients: Mutex::new(vec![]),
            world: Mutex::new(World {
                mob_x: 3668,
                mob_dir: Direction::East,
            }),
        }
    }

    pub async fn run_loop(&self) -> Result<()> {
        use ultimaonline_net::{packets::mobile, types};

        let span = trace_span!("server");
        let _ = span.enter();

        let mut frame = 0;
        while !self.shutdown.load(Ordering::Relaxed) {
            frame += 1;
            trace!("Frame: {}", frame);
            {
                // Update world state
                let mut world = self
                    .world
                    .lock()
                    .map_err(|_| Error::Message("Unable to lock world".to_string()))?;
                if (frame / 10) % 2 == 0 {
                    world.mob_x += 1;
                } else {
                    world.mob_x -= 1;
                }

                if frame % 10 == 0 {
                    world.mob_dir = match world.mob_dir {
                        Direction::East => Direction::West,
                        Direction::West => Direction::East,
                        _ => Direction::East,
                    };
                }

                let mut clients = self
                    .clients
                    .lock()
                    .map_err(|_| Error::Message("Unable to lock clients vec".to_string()))?;

                let mut closed_clients = HashSet::<usize>::new();

                // Receive client packets
                for (i, client) in clients.iter_mut().enumerate() {
                    if client.sender.is_closed() {
                        closed_clients.insert(i);
                        continue;
                    }

                    loop {
                        match client.recv()? {
                            None => break,
                            Some(client::codecs::InWorldFrameRecv::Request(req)) => {
                                // Always succeed for now
                                client.send(
                                    movement::Success {
                                        sequence: req.sequence,
                                        notoriety: types::Notoriety::Ally,
                                    }
                                    .into(),
                                )?;
                            }
                            _ => {} // Skip everything
                        }
                    }
                }

                for (i, client) in clients.iter_mut().enumerate() {
                    if client.sender.is_closed() {
                        closed_clients.insert(i);
                        continue;
                    }

                    client.send(
                        mobile::State {
                            serial: 55858,
                            body: 401,
                            x: world.mob_x,
                            y: 2625,
                            z: 0,
                            direction: world.mob_dir,
                            hue: 1003,
                            flags: mobile::EntityFlags::None,
                            notoriety: types::Notoriety::Ally,
                        }
                        .into(),
                    )?;
                }

                let mut closed_clients: Vec<&usize> = closed_clients.iter().collect();
                closed_clients.sort();
                closed_clients.reverse();
                for i in closed_clients {
                    clients.remove(*i);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        for client in self
            .clients
            .lock()
            .map_err(|_| Error::Message("Unable to lock clients vec".to_string()))?
            .iter_mut()
        {
            client.close();
        }

        info!("Server shutting down.");
        Ok(())
    }

    pub fn new_client(&self) -> Result<Client> {
        let (output_send, output_recv) =
            mpsc::unbounded_channel::<<WorldClient as ClientSender>::SendItem>();
        let (input_send, input_recv) =
            mpsc::unbounded_channel::<<WorldClient as ClientReceiver>::RecvItem>();

        let mut client = WorldClient {
            sender: output_send,
            receiver: input_recv,
        };

        self.enter_world(&mut client)?;
        debug!("Client completed enter world.");

        self.clients
            .lock()
            .map_err(|_| Error::Message("Unable to lock clients vec".to_string()))?
            .push(client);

        Ok(Client {
            sender: input_send,
            receiver: output_recv,
        })
    }

    fn enter_world(&self, client: &mut WorldClient) -> Result<()> {
        use ultimaonline_net::{packets::*, types};

        client.send(
            mobile::MobLightLevel {
                serial: PLAYER_SERIAL,
                level: 30,
            }
            .into(),
        )?;

        client.send(world::WorldLightLevel { level: 30 }.into())?;

        let world = self
            .world
            .lock()
            .map_err(|_| Error::Message("Unable to lock world".to_string()))?;

        client.send(
            mobile::Appearance {
                state: mobile::State {
                    serial: 55858,
                    body: 401,
                    x: world.mob_x,
                    y: 2625,
                    z: 0,
                    direction: world.mob_dir,
                    hue: 1003,
                    flags: mobile::EntityFlags::None,
                    notoriety: types::Notoriety::Ally,
                },
                items: vec![
                    mobile::Item {
                        serial: 0x40000001,
                        type_id: 0x1EFD, // Fancy Shirt
                        layer: 0x05,     // Shirt
                        hue: 1837,
                    },
                    mobile::Item {
                        serial: 0x40000002,
                        type_id: 0x1539, // Long Pants
                        layer: 0x04,     // Pants
                        hue: 1897,
                    },
                    mobile::Item {
                        serial: 0x40000003,
                        type_id: 0x170B, // Boots
                        layer: 0x04,     // Shoes
                        hue: 1900,
                    },
                    mobile::Item {
                        serial: 0x40000004,
                        type_id: 0x1515, // Cloak
                        layer: 0x14,     // Cloak
                        hue: 1811,
                    },
                    mobile::Item {
                        serial: 0x40000005,
                        type_id: 0x203C, // Long hair
                        layer: 0x0B,     // Hair
                        hue: 1111,
                    },
                ]
                .into(),
            }
            .into(),
        )?;

        Ok(())
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed)
    }
}
