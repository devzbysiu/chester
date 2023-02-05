use crate::configuration::factories::Context;
use crate::result::SetupErr;
use crate::use_cases::repo::RepoRead;
use crate::use_cases::services::runner_shell::TestRunnerShell;
use crate::use_cases::services::sink_shell::ResultsSinkShell;
use crate::use_cases::services::watcher_shell::ChangeWatcherShell;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Response, Server};
use hyperlocal::UnixServerExt;
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

#[allow(unused)]
#[allow(clippy::needless_pass_by_value)]
pub fn setup_shells(ctx: Context) -> RepoRead {
    let Context {
        cfg: _,
        bus,
        change_watcher,
        test_runner,
        repo,
    } = ctx;

    let watcher_shell = ChangeWatcherShell::new(bus.clone());
    let runner_shell = TestRunnerShell::new(bus.clone());
    let sink_shell = ResultsSinkShell::new(bus.clone());

    watcher_shell.run(change_watcher);
    runner_shell.run(test_runner);
    sink_shell.run(repo.write());

    repo.read()
}

#[allow(clippy::needless_pass_by_value)]
pub async fn start_server(_repo_read: Option<RepoRead>) -> Result<(), SetupErr> {
    let runtime_path = dirs::runtime_dir().unwrap_or(PathBuf::from("/run"));
    let socket_path = runtime_path.join("chester.sock");

    if socket_path.exists() {
        debug!("socket file exists, removing");
        fs::remove_file(&socket_path)?;
    }

    let make_service = make_service_fn(|_| async {
        Ok::<_, hyper::Error>(service_fn(|_| async {
            Ok::<_, hyper::Error>(Response::new(Body::from(r#"{"status": "success"}"#)))
        }))
    });

    info!("binding {socket_path:?}");
    Server::bind_unix(socket_path)?.serve(make_service).await?;

    Ok(())
}
