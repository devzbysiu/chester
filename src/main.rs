#![allow(clippy::module_name_repetitions)]

use crate::configuration::tracing::init_tracing;
use crate::startup::start_server;

use anyhow::Result;

mod configuration;
mod data_providers;
mod entities;
mod use_cases;

mod result;
mod startup;
#[cfg(test)]
mod testingtools;

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    start_server(None).await?;

    Ok(())
}
