use crate::configuration::factories::Runtime;
use crate::use_cases::services::runner_shell::TestRunnerShell;
use crate::use_cases::services::sink_shell::ResultsSinkShell;
use crate::use_cases::services::watcher_shell::ChangeWatcherShell;
use crate::use_cases::state::State;

#[allow(unused)]
#[allow(clippy::needless_pass_by_value)]
pub fn setup_shells(rt: Runtime) -> State {
    let Runtime {
        cfg: _,
        bus,
        change_watcher,
        test_runner,
        state,
    } = rt;

    let watcher_shell = ChangeWatcherShell::new(bus.clone());
    let runner_shell = TestRunnerShell::new(bus.clone());
    let sink_shell = ResultsSinkShell::new(bus.clone());

    watcher_shell.run(change_watcher, state.reader());
    runner_shell.run(test_runner, state.reader());
    sink_shell.run(state.writer());

    state
}
