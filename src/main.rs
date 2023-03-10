#![allow(clippy::module_name_repetitions)]

use crate::configuration::config::cfg;
use crate::configuration::factories::Runtime;
use crate::configuration::tracing::init_tracing;
use crate::data_providers::server::start_server;
use crate::startup::setup_shells;

use anyhow::Result;

mod configuration;
mod data_providers;
mod entities;
mod use_cases;

mod result;
mod startup;
#[cfg(test)]
mod testingtools;

#[actix_web::main]
async fn main() -> Result<()> {
    init_tracing();
    start_server(setup_shells(Runtime::new(cfg()?)?)).await?;

    Ok(())
}
