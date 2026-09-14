#![feature(duration_constants)]
#![feature(duration_constructors)]
// #![feature(time_systemtime_limits)]
// #![feature(time_saturating_systemtime)]
#![no_std]
#![no_main]

use pspsdk::testrt::TestRunner;

extern crate alloc;

pspsdk::module!("UNIT_TESTS", 1, 1);

mod sync;
mod time;

fn psp_main() {
    let test_groups = [sync::test_group, time::test_group];

    let mut runner = TestRunner::file_runner();

    runner.start_run();

    for test in test_groups {
        runner.run_group(test);
    }

    runner.finish_run();
}
