use crate::configuration::factories::Runtime;
use crate::use_cases::services::coverage_shell::CoverageShell;
use crate::use_cases::services::sink_shell::ResultsSinkShell;
use crate::use_cases::services::tests_shell::TestsShell;
use crate::use_cases::services::watcher_shell::ChangeWatcherShell;
use crate::use_cases::state::State;

#[allow(unused)]
pub fn setup_shells(rt: Runtime) -> State {
    let Runtime {
        cfg: _,
        bus,
        change_watcher,
        test_runner,
        coverage_runner,
        state,
    } = rt;

    let watcher_shell = ChangeWatcherShell::new(bus.clone());
    let tests_shell = TestsShell::new(bus.clone());
    let coverage_shell = CoverageShell::new(bus.clone());
    let sink_shell = ResultsSinkShell::new(bus.clone());

    watcher_shell.run(change_watcher, state.reader());
    tests_shell.run(test_runner, state.reader());
    coverage_shell.run(coverage_runner, state.reader());
    sink_shell.run(state.writer());

    state
}
