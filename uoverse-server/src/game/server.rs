use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use ultimaonline_net::error::Result;

pub struct Server {
    shutdown: Arc<AtomicBool>,
}

impl Server {
    pub fn new(shutdown: Arc<AtomicBool>) -> Self {
        Server { shutdown }
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
}
