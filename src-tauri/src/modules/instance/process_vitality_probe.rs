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
            use windows_sys::Win32::Foundation::CloseHandle;
            use windows_sys::Win32::System::Threading::{
                GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
            };

            unsafe {
                let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
                if handle == 0 {
                    return false;
                }

                let mut exit_code: u32 = 0;
                let success = GetExitCodeProcess(handle, &mut exit_code);
                CloseHandle(handle);

                // STILL_ACTIVE is 259
                success != 0 && exit_code == 259
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
