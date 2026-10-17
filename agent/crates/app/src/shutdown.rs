use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static RUNNING: AtomicBool = AtomicBool::new(true);

static FINISHED: AtomicBool = AtomicBool::new(false);

#[cfg_attr(not(windows), allow(dead_code))]
const GRACE: Duration = Duration::from_secs(4);

pub fn running_flag() -> &'static AtomicBool {
    &RUNNING
}

pub fn mark_finished() {
    FINISHED.store(true, Ordering::SeqCst);
}

#[cfg_attr(not(windows), allow(dead_code))]
fn stop_and_wait() {
    RUNNING.store(false, Ordering::SeqCst);

    let deadline = Instant::now() + GRACE;
    while !FINISHED.load(Ordering::SeqCst) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(windows)]
pub fn install() {
    use windows_sys::Win32::Foundation::BOOL;
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;

    unsafe extern "system" fn handler(_event: u32) -> BOOL {
        stop_and_wait();
        1
    }

    unsafe {
        SetConsoleCtrlHandler(Some(handler), 1);
    }
}

#[cfg(not(windows))]
pub fn install() {}
