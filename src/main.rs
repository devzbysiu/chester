#![allow(clippy::module_name_repetitions)]

use crate::configuration::config::{Cmd, ConfigBuilder};
use crate::configuration::factories::Runtime;
use crate::configuration::tracing::init_tracing;
use crate::data_providers::server::start_server;
use crate::entities::ignored_path::IgnoredPath;
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
    let cfg = ConfigBuilder::default()
        .cmd(Cmd::new("cargo", "test"))
        .ignored_paths(vec![IgnoredPath::new("target")?, IgnoredPath::new(".git")?])
        .build()?;
    let reader = setup_shells(Runtime::new(cfg)?);
    start_server(reader).await?;

    Ok(())
}
