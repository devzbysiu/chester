use crate::configuration::factories::Runtime;
use crate::use_cases::services::check_shell::CheckShell;
use crate::use_cases::services::coverage_shell::CoverageShell;
use crate::use_cases::services::tests_index_shell::TestsIndexShell;
use crate::use_cases::services::tests_shell::TestsShell;
use crate::use_cases::services::watcher_shell::ChangeWatcherShell;
use crate::use_cases::state::State;

#[allow(unused)]
pub fn setup_shells(rt: Runtime) -> State {
    let Runtime {
        cfg: _,
        bus,
        change_watcher,
        tests_index,
        test_runner,
        check_runner,
        coverage_runner,
        state,
    } = rt;

    let watcher_shell = ChangeWatcherShell::new(bus.clone());
    let check_shell = CheckShell::new(bus.clone());

    let tests_shell = TestsShell::new(bus.clone());
    let index_shell = TestsIndexShell::new(bus.clone());

    let coverage_shell = CoverageShell::new(bus.clone());

    watcher_shell.run(change_watcher, state.reader());
    index_shell.run(tests_index, state.clone());
    check_shell.run(check_runner, state.clone());
    tests_shell.run(test_runner, state.clone());
    coverage_shell.run(coverage_runner, state.clone());

    state
}
