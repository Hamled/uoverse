use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
use ultimaonline_net::{
    error::{Error, Result},
    types::{Direction, Serial},
};

use super::client::{Client, ClientReceiver, ClientSender, WorldClient};

struct World {
    mob_x: u16,
    mob_dir: Direction,
}

pub struct Server {
    shutdown: Arc<AtomicBool>,
    clients: Mutex<Vec<WorldClient>>,
    world: Mutex<World>,
}

const PLAYER_SERIAL: Serial = 3833;

impl Server {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        Server {
            shutdown,
            clients: Mutex::new(vec![]),
            world: Mutex::new(World {
                mob_x: 3668,
                mob_dir: Direction::East,
            }),
        }
    }

    pub async fn run_loop(&self) -> Result<()> {
        use ultimaonline_net::{packets::mobile, types};

        let mut frame = 0;
        while !self.shutdown.load(Ordering::Relaxed) {
            frame += 1;
            println!("Frame: {}", frame);
            {
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

        println!("Server shutting down.");
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
        println!("Client completed enter world.");

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
}
