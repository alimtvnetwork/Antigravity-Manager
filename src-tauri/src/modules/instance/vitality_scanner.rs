//! Instance Process Vitality Scanner.
//! Conscious Win32 / POSIX PID verification distinguishing live Antigravity IDE processes from dead PIDs.

#[cfg(windows)]
extern "system" {
    fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut std::ffi::c_void;
    fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
    fn GetExitCodeProcess(hProcess: *mut std::ffi::c_void, lpExitCode: *mut u32) -> i32;
}

#[cfg(windows)]
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
#[cfg(windows)]
const STILL_ACTIVE: u32 = 259;

/// Checks whether a given PID is currently active and running.
pub fn is_process_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }

    #[cfg(windows)]
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }

        let mut exit_code: u32 = 0;
        let res = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);

        res != 0 && exit_code == STILL_ACTIVE
    }

    #[cfg(not(windows))]
    unsafe {
        libc::kill(pid as i32, 0) == 0
    }
}

/// Scans a list of candidate PIDs, returning their vitality statuses.
pub fn verify_instance_pids(pids: &[u32]) -> Vec<(u32, bool)> {
    pids.iter().map(|&pid| (pid, is_process_alive(pid))).collect()
}
