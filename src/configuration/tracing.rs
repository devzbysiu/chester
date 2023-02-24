use once_cell::sync::Lazy;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, EnvFilter};

static TRACING: Lazy<()> = Lazy::new(setup_global_subscriber);

fn setup_global_subscriber() {
    let logs_dir = dirs::state_dir()
        .unwrap_or(PathBuf::from("~/.local/state"))
        .join("chester");
    let file_appender = rolling::daily(logs_dir, "chester.log");
    let env_filter = EnvFilter::from_default_env();
    fmt()
        .with_env_filter(env_filter)
        .with_writer(file_appender)
        .init();
}

pub fn init_tracing() {
    Lazy::force(&TRACING);
}
