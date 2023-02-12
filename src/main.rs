#![allow(clippy::module_name_repetitions)]

use crate::configuration::config::Config;
use crate::configuration::factories::Context;
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
    let reader = setup_shells(Context::new(Config::default())?);
    start_server(reader).await?;

    Ok(())
}
