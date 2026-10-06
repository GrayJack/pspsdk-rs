use core::{
    ffi::CStr,
    fmt::Arguments,
    panic::{RefUnwindSafe, UnwindSafe},
    time::Duration,
};

use alloc::{format, string::String, vec::Vec};

use crate::{
    io::Write,
    os::fd::{FileDesc, FromRawFd},
    panic,
    process::{ExitCode, Termination},
    sys::{
        self,
        io::{FileFlags, Mode},
    },
    time::Instant,
};

pub const OUTPUT_FILENAME: &str = "psp_output_file.log";
pub const OUTPUT_FIFO: &str = "psp_output_pipe.fifo";

pub const STARTING_TOKEN: &str = "STARTING_TESTS";
pub const SUCCESS_TOKEN: &str = "FINAL_SUCCESS";
pub const FAILURE_TOKEN: &str = "FINAL_FAILURE";


pub struct TestRunner<'a> {
    mode: TestRunnerMode,
    failure: bool,
    failures: Vec<&'a str>,
    successed: usize,
    measured: usize,
}

enum TestRunnerMode {
    Fifo(FileDesc),
    File(FileDesc),
    Stdout { finish_process: bool },
    Screen,
}

impl<'a> TestRunner<'a> {
    pub fn fifo_runner() -> Self {
        let fd = get_test_output_pipe();
        Self {
            mode: TestRunnerMode::Fifo(fd),
            failure: false,
            failures: Vec::new(),
            successed: 0,
            measured: 0,
        }
    }

    pub fn file_runner(filepath: &CStr) -> Self {
        let fd = get_test_output_file(filepath);
        Self {
            mode: TestRunnerMode::File(fd),
            failure: false,
            failures: Vec::new(),
            successed: 0,
            measured: 0,
        }
    }

    pub fn stdout_runner(finish_process: bool) -> Self {
        Self {
            mode: TestRunnerMode::Stdout { finish_process },
            failure: false,
            failures: Vec::new(),
            successed: 0,
            measured: 0,
        }
    }

    pub fn screen_runner() -> Self {
        Self {
            mode: TestRunnerMode::Screen,
            failure: false,
            failures: Vec::new(),
            successed: 0,
            measured: 0,
        }
    }

    pub fn run_group<F>(&mut self, f: F)
    where
        F: Fn(&mut TestRunner),
    {
        f(self)
    }

    pub fn test<R, F>(&mut self, testcase_name: &'a str, f: F)
    where
        R: Termination + 'static,
        F: FnOnce() -> R + UnwindSafe,
    {
        let res = panic::catch_unwind(f);
        match res {
            Ok(r) => {
                let report = r.report();
                if report == ExitCode::SUCCESS {
                    self.successed += 1;
                    self.pass(testcase_name, "ok");
                } else {
                    self.failure = true;
                    self.failures.push(testcase_name);
                    self.write_args(format_args!(
                        "[FAIL]: ({testcase_name}) exited with: {}\n",
                        report.to_i32()
                    ));
                }
            },
            Err(err) => {
                self.failure = true;
                self.failures.push(testcase_name);

                if let Some(msg) = err.downcast_ref::<&'static str>() {
                    self.write_args(format_args!("[FAIL]: ({testcase_name}) panicked: {msg}\n"));
                } else if let Some(msg) = err.downcast_ref::<String>() {
                    self.write_args(format_args!("[FAIL]: ({testcase_name}) panicked: {msg}\n"));
                } else {
                    self.write_args(format_args!(
                        "[FAIL]: ({testcase_name}) panicked: unknown panic payload\n"
                    ));
                }
            },
        }
    }

    pub fn should_panic<R, F>(&mut self, testcase_name: &'a str, f: F)
    where
        R: Termination + 'static,
        F: FnOnce() -> R + UnwindSafe,
    {
        let res = panic::catch_unwind(f);
        match res {
            Ok(_) => {
                self.failure = true;
                self.failures.push(testcase_name);
                self.write_args(format_args!("[FAIL]: ({testcase_name}): does not panicked\n",));
            },
            Err(_) => {
                self.successed += 1;
                self.pass(testcase_name, "ok");
            },
        }
    }

    pub fn bench<R, F>(&mut self, testcase_name: &'a str, f: F, iterations: usize)
    where
        R: Termination + 'static,
        F: Fn() -> R + UnwindSafe + RefUnwindSafe,
    {
        if iterations == 0 {
            self.failure = true;
            self.failures.push(testcase_name);
            self.write_args(format_args!(
                "[FAIL]: ({testcase_name}) benchmark requires at least one iteration\n"
            ));
            return;
        }

        let mut samples = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let result = panic::catch_unwind(|| {
                let start = Instant::now();
                let res = f();
                let elapsed = start.elapsed();
                (res, elapsed)
            });

            match result {
                Ok((res, elapsed)) => {
                    let report = res.report();
                    if report == ExitCode::SUCCESS {
                        samples.push(elapsed);
                    } else {
                        self.failure = true;
                        self.failures.push(testcase_name);
                        self.write_args(format_args!(
                            "[FAIL]: ({testcase_name}) exited with: {}\n",
                            report.to_i32()
                        ));
                        return;
                    }
                },
                Err(_) => {
                    self.failure = true;
                    self.failures.push(testcase_name);
                    self.write_args(format_args!("[FAIL]: ({testcase_name}) panicked\n"));
                    return;
                },
            }
        }

        samples.sort_unstable();

        let median = if samples.len() % 2 == 1 {
            samples[samples.len() / 2]
        } else {
            let middle = samples.len() / 2;
            Duration::from_nanos_u128((samples[middle - 1] + samples[middle]).as_nanos() / 2)
        };

        let total_nanos: u128 = samples.iter().map(Duration::as_nanos).sum();

        let mean_nanos = total_nanos / samples.len() as u128;

        let variance = samples
            .iter()
            .map(|sample| {
                let difference = sample.as_nanos() as i128 - mean_nanos as i128;
                (difference * difference) as u128
            })
            .sum::<u128>()
            / samples.len() as u128;

        let deviation = Duration::from_nanos_u128(variance.isqrt());

        self.measured += 1;
        self.write_args(format_args!(
            "[BENCH]: ({testcase_name}) {median:?}/iter (+/- {deviation:?})\n",
        ));
    }

    pub fn start_run(&self) {
        self.write_args(format_args!("\n\n{}\n", STARTING_TOKEN));
    }

    pub fn finish_run(self) {
        // report
        self.write_args(format_args!(
            "\nTest results: {} passed; {} failed; {} measured\n",
            self.successed,
            self.failures.len(),
            self.measured
        ));

        if self.failure {
            self.write_args(format_args!("Failing tests: {:?}\n", self.failures));
            self.write_args(format_args!("{}\n", FAILURE_TOKEN));
        } else {
            self.write_args(format_args!("{}\n", SUCCESS_TOKEN));
        }
        self.quit();
    }

    pub fn pass(&self, testcase_name: &str, msg: &str) {
        self.write_args(format_args!("[PASS]: ({testcase_name}) {msg}\n"));
    }

    pub fn dbg(&self, testcase_name: &str, msg: &str) {
        self.write_args(format_args!("[NOTE]: ({testcase_name}) {msg}\n"));
    }

    pub fn write_args(&self, args: Arguments) {
        match &self.mode {
            TestRunnerMode::File(fd) | TestRunnerMode::Fifo(fd) => {
                write_to_psp_output_fd(fd, args);
            },
            TestRunnerMode::Screen => {
                crate::dprint!("{}", args);
            },
            TestRunnerMode::Stdout { .. } => {
                crate::print!("{}", args);
            },
        }
    }

    fn quit(self) {
        match self.mode {
            TestRunnerMode::File(fd) | TestRunnerMode::Fifo(fd) => {
                drop(fd);
                quit_game();
            },
            TestRunnerMode::Stdout { finish_process } => {
                if finish_process {
                    quit_game();
                } else {
                    sleep()
                }
            },
            TestRunnerMode::Screen => sleep(),
        }
    }
}

fn get_test_output_pipe() -> FileDesc {
    unsafe {
        let fd = sys::io::sceIoOpen(
            psp_filename(OUTPUT_FIFO),
            FileFlags::AppendMode | FileFlags::WriteOnly,
            Mode::from(0o777),
        )
        .into_result();

        match fd {
            Ok(fd) => FileDesc::from_raw_fd(fd.to_inner()),
            Err(err) => panic!(
                "Unable to open pipe \"{err}\" for output! You must create it yourself with \
                 `mkfifo`.",
            ),
        }
    }
}

fn get_test_output_file(filepath: &CStr) -> FileDesc {
    unsafe {
        let fd = sys::io::sceIoOpen(
            filepath.as_ptr().cast(),
            FileFlags::AppendMode | FileFlags::WriteOnly | FileFlags::CreateFile,
            Mode::from(0o777),
        )
        .into_result();

        match fd {
            Ok(fd) => FileDesc::from_raw_fd(fd.to_inner()),
            Err(err) => panic!("Unable to open file \"{}\" for output! {err}", OUTPUT_FILENAME),
        }
    }
}

fn psp_filename(filename: &str) -> *const u8 {
    format!("ms0:/PSP/GAME/ATEST/{}\0", filename).as_bytes().as_ptr()
}

fn write_to_psp_output_fd(mut fd: &FileDesc, args: Arguments) {
    let _res = (&mut fd).write_fmt(args);
}

fn sleep() {
    let _ = crate::sys::thread::sceKernelSleepThreadCB();
}

fn quit_game() {
    crate::process::exit(0);
}
