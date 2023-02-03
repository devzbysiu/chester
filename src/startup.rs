use crate::configuration::factories::Context;
use crate::use_cases::services::runner::TestRunnerShell;
use crate::use_cases::services::sink::ResultsSinkShell;
use crate::use_cases::services::watcher::ChangeWatcherShell;

#[allow(unused)]
#[allow(clippy::needless_pass_by_value)]
pub fn setup(ctx: Context) {
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
}
