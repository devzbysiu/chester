#![allow(clippy::module_name_repetitions)]

use crate::configuration::tracing::init_tracing;
use crate::data_providers::server::start_server;

use anyhow::Result;
use configuration::factories::state;

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
    let state = state();
    start_server(state.reader()).await?;

    Ok(())
}
