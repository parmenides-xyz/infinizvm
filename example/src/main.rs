use anyhow::Result;
use derive::{da::DaDeriveImpl, instant::InstantDeriveImpl};
use infinizvm_interface::{l2::Engine, runner::Runner};
use l2::batcher::Batcher;
use zcash::{chain::MockZcash, stream::TxServer};
use runner::SimpleRunner;
use std::path::Path;
use tokio::sync::mpsc::channel;

mod derive;
mod l1;
mod l2;
mod zcash;
mod runner;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let (instant_sender, instant_receiver) = channel(1024);
    let instanct_driver = InstantDeriveImpl::new(instant_receiver);

    let (da_sender, da_receiver) = channel(1);
    let da_driver = DaDeriveImpl::default();

    let (attribute_sender, attribute_receiver) = channel(1024);
    let mut runner = SimpleRunner::new(Path::new("/tmp/infinizvm-example"), attribute_sender)?;

    runner.register_instant(instanct_driver);
    runner.register_da(da_driver.clone());

    MockZcash::new(1000, instant_sender).run();
    TxServer::new(runner.get_engine().stream().clone()).run();
    Batcher::new(da_sender).run(attribute_receiver);
    da_driver.run(da_receiver);

    loop {
        if let Err(e) = runner.advance().await {
            // We should match the error type and panic accordingly in production code
            error!("Error: {}", e);
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
}