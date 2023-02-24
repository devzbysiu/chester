#![allow(clippy::module_name_repetitions)]

use crate::configuration::config::Config;
use crate::configuration::factories::Runtime;
use crate::configuration::tracing::init_tracing;
use crate::data_providers::server::start_server;
use crate::startup::setup_shells;

use anyhow::Result;
use entities::ignored_path::IgnoredPath;

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
    let ignored_paths = vec![IgnoredPath::new("target"), IgnoredPath::new(".git")];
    let reader = setup_shells(Runtime::new(Config { ignored_paths })?);
    start_server(reader).await?;

    Ok(())
}
