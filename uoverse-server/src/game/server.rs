use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use tokio::sync::mpsc;
use ultimaonline_net::error::{Error, Result};

use super::client::{Client, ClientReceiver, ClientSender, WorldClient};

pub struct Server {
    shutdown: Arc<AtomicBool>,
    clients: Mutex<Vec<WorldClient>>,
}

impl Server {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        Server {
            shutdown,
            clients: Mutex::new(vec![]),
        }
    }

    pub async fn run_loop(&self) -> Result<()> {
        let mut frame = 0;
        while !self.shutdown.load(Ordering::Relaxed) {
            frame += 1;
            println!("Frame: {}", frame);

            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }

        println!("Server shutting down.");
        Ok(())
    }

    pub fn new_client(&self) -> Result<Client> {
        let (output_send, output_recv) =
            mpsc::unbounded_channel::<<WorldClient as ClientSender>::SendItem>();
        let (input_send, input_recv) =
            mpsc::unbounded_channel::<<WorldClient as ClientReceiver>::RecvItem>();

        self.clients
            .lock()
            .map_err(|_| Error::Message("Unable to lock clients vec".to_string()))?
            .push(WorldClient {
                sender: output_send,
                receiver: input_recv,
            });

        Ok(Client {
            sender: input_send,
            receiver: output_recv,
        })
    }
}
}
