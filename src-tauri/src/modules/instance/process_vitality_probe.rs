//! Lightweight Process Vitality Probe
//! Efficiently checks OS process table vitality for target PIDs
//! using direct kernel handles without triggering heavy system-wide sysinfo scans.

pub struct ProcessVitalityProbe;

impl ProcessVitalityProbe {
    /// Fast cross-platform process vitality check for a specific PID.
    pub fn is_pid_alive(pid: u32) -> bool {
        if pid == 0 {
            return false;
        }

        #[cfg(target_os = "windows")]
        {
            #[link(name = "kernel32")]
            extern "system" {
                fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
                fn GetExitCodeProcess(handle: *mut std::ffi::c_void, code: *mut u32) -> i32;
                fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
            }
            const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
            const STILL_ACTIVE: u32 = 259;

            unsafe {
                let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                if handle.is_null() {
                    return false;
                }

                let mut exit_code: u32 = 0;
                let success = GetExitCodeProcess(handle, &mut exit_code);
                CloseHandle(handle);

                // STILL_ACTIVE is 259
                success != 0 && exit_code == STILL_ACTIVE
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            unsafe {
                // kill(pid, 0) checks if process exists without sending a signal
                libc::kill(pid as libc::pid_t, 0) == 0
            }
        }
    }
}
