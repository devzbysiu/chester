use crate::configuration::factories::Context;
use crate::use_cases::repo::RepoRead;
use crate::use_cases::services::runner_shell::TestRunnerShell;
use crate::use_cases::services::sink_shell::ResultsSinkShell;
use crate::use_cases::services::watcher_shell::ChangeWatcherShell;

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
