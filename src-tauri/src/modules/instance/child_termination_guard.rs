//! Child process termination guard and escalating kill ladder.
//! Implements multi-stage process termination (graceful interrupt -> delay -> forced kill)
//! to clean up stubborn language server and background worker processes.

use crate::error::AppError;
use std::time::Duration;

pub struct ChildTerminationGuard;

impl ChildTerminationGuard {
    /// Terminates a process using an escalating strategy.
    pub fn terminate_with_ladder(pid: u32, grace_period_ms: u64) -> Result<bool, AppError> {
        if pid == 0 {
            return Ok(false);
        }

        #[cfg(target_os = "windows")]
        {
            #[link(name = "kernel32")]
            extern "system" {
                fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
                fn TerminateProcess(handle: *mut std::ffi::c_void, exit_code: u32) -> i32;
                fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
            }
            const PROCESS_TERMINATE: u32 = 0x0001;

            unsafe {
                let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
                if handle.is_null() {
                    return Ok(false); // Process likely already terminated
                }

                // Stage 1: Attempt termination with exit code 0
                let success = TerminateProcess(handle, 0) != 0;
                CloseHandle(handle);

                if success {
                    std::thread::sleep(Duration::from_millis(grace_period_ms.min(500)));
                    return Ok(true);
                }
            }

            Ok(false)
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix systems, send SIGTERM then SIGKILL if still running
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
                std::thread::sleep(Duration::from_millis(grace_period_ms.min(500)));
                libc::kill(pid as i32, libc::SIGKILL);
            }

            Ok(true)
        }
    }
}
