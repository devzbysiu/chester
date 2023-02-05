use crate::configuration::factories::Context;
use crate::result::SetupErr;
use crate::use_cases::repo::RepoRead;
use crate::use_cases::services::runner_shell::TestRunnerShell;
use crate::use_cases::services::sink_shell::ResultsSinkShell;
use crate::use_cases::services::watcher_shell::ChangeWatcherShell;

use interprocess::os::unix::udsocket::{UdStream, UdStreamListener};
use std::io::{self, prelude::*};
use std::net::Shutdown;
use tracing::error;

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
pub fn start_server(_repo_read: Option<RepoRead>) -> Result<(), SetupErr> {
    let listener = UdStreamListener::bind("/tmp/example.sock")?;
    for mut conn in listener.incoming().filter_map(handle_error) {
        conn.write_all(r#"{"status": "success"}"#.as_bytes())?;
        conn.shutdown(Shutdown::Write)?;
    }

    Ok(())
}

fn handle_error(result: io::Result<UdStream>) -> Option<UdStream> {
    match result {
        Ok(val) => Some(val),
        Err(error) => {
            error!("There was an error with an incoming connection: {error}");
            None
        }
    }
}
