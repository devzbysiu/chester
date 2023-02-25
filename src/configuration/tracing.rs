use once_cell::sync::Lazy;
use tracing_subscriber::{fmt, EnvFilter};

static TRACING: Lazy<()> = Lazy::new(setup_global_subscriber);

fn setup_global_subscriber() {
    let env_filter = EnvFilter::from_default_env();

    // NOTE: Append logs to file when not in test mode
    #[cfg(not(test))]
    {
        use std::path::PathBuf;
        use tracing_appender::rolling;

        let logs_dir = dirs::state_dir()
            .unwrap_or(PathBuf::from("~/.local/state"))
            .join("chester");
        fmt()
            .with_env_filter(env_filter)
            .with_writer(rolling::daily(logs_dir, "chester.log"))
            .init();
    }

    // NOTE: Print logs to screen when testing
    #[cfg(test)]
    fmt().with_env_filter(env_filter).init();
}

pub fn init_tracing() {
    Lazy::force(&TRACING);
}
