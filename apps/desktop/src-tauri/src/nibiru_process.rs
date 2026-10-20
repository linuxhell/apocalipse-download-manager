//! Lifetime-bound supervision for native CLI operations.
//!
//! Unix children inherit a dedicated process group. Windows children inherit a
//! kill-on-close Job Object, assigned before their initial thread can execute.
//! The guard remains valid after the immediate child exits or is reaped.
use tokio::process::{Child, Command};

#[cfg(unix)]
pub(super) struct ProcessGroup {
    pgid: libc::pid_t,
}

#[cfg(unix)]
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        // SAFETY: spawn configured a new group whose positive ID is the child
        // PID. Negating it addresses that group, never the caller's group.
        // ESRCH simply means all members already exited. SIGKILL also covers a
        // descendant ignoring SIGTERM and requires no external executable.
        unsafe {
            libc::kill(-self.pgid, libc::SIGKILL);
        }
    }
}

#[cfg(unix)]
pub(super) fn spawn(command: &mut Command) -> Result<(Child, ProcessGroup), String> {
    command.process_group(0);
    let child = command
        .spawn()
        .map_err(|_| "nibiru_executable_unavailable".to_owned())?;
    let pgid = child
        .id()
        .and_then(|pid| libc::pid_t::try_from(pid).ok())
        .filter(|pid| *pid > 0)
        .ok_or("nibiru_process_supervision_failed")?;
    Ok((child, ProcessGroup { pgid }))
}

#[cfg(windows)]
mod windows {
    use super::{Child, Command};
    use std::{
        io,
        mem::size_of,
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD,
                THREADENTRY32,
            },
            JobObjects::{
                AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
                SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
                JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            },
            Threading::{
                OpenThread, ResumeThread, CREATE_NO_WINDOW, CREATE_SUSPENDED, THREAD_SUSPEND_RESUME,
            },
        },
    };

    pub(in super::super) struct ProcessGroup {
        // Closing the last handle synchronously requests termination of all
        // job members, including descendants whose parent has already exited.
        job: OwnedHandle,
    }

    impl ProcessGroup {
        fn new() -> io::Result<Self> {
            // SAFETY: null security attributes create a non-inheritable handle;
            // null name creates a fresh, anonymous job owned by this operation.
            let raw = unsafe { CreateJobObjectW(ptr::null(), ptr::null()) };
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            // SAFETY: CreateJobObjectW returned a new valid handle, owned once.
            let job = unsafe { OwnedHandle::from_raw_handle(raw) };
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            // SAFETY: the handle is live and the pointer/length describe the
            // exact information class. No breakaway flags are enabled.
            let configured = unsafe {
                SetInformationJobObject(
                    job.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )
            };
            if configured == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Self { job })
        }

        fn attach(&self, child: &Child) -> io::Result<()> {
            let process = child
                .raw_handle()
                .ok_or_else(|| io::Error::other("child process handle missing"))?;
            // SAFETY: both owned handles are live. The child was created
            // suspended, so it cannot launch descendants before assignment.
            if unsafe { AssignProcessToJobObject(self.job.as_raw_handle(), process) } == 0 {
                return Err(io::Error::last_os_error());
            }
            resume_initial_thread(
                child
                    .id()
                    .ok_or_else(|| io::Error::other("child PID missing"))?,
            )
        }
    }

    fn resume_initial_thread(pid: u32) -> io::Result<()> {
        // Stable std/Tokio Child exposes the process handle, not its initial
        // thread handle. ToolHelp is the documented Win32 way to enumerate a
        // process's threads. CREATE_SUSPENDED keeps this process at one thread
        // until its job membership has been established.
        // https://learn.microsoft.com/windows/win32/toolhelp/traversing-the-thread-list
        // https://learn.microsoft.com/windows/win32/procthread/process-creation-flags
        // SAFETY: a thread snapshot requires no pointers or caller buffers.
        let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if raw == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: this is a new valid snapshot handle with a single owner.
        let snapshot = unsafe { OwnedHandle::from_raw_handle(raw) };
        let mut entry = THREADENTRY32 {
            dwSize: size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };
        let mut thread_id = None;
        // SAFETY: the entry's size is initialized and its storage is writable.
        let mut found = unsafe { Thread32First(snapshot.as_raw_handle(), &mut entry) };
        while found != 0 {
            if entry.th32OwnerProcessID == pid && thread_id.replace(entry.th32ThreadID).is_some() {
                return Err(io::Error::other("suspended child has multiple threads"));
            }
            entry.dwSize = size_of::<THREADENTRY32>() as u32;
            // SAFETY: the snapshot and correctly sized output buffer are live.
            found = unsafe { Thread32Next(snapshot.as_raw_handle(), &mut entry) };
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
            return Err(error);
        }
        let thread_id = thread_id.ok_or_else(|| io::Error::other("initial thread missing"))?;
        // SAFETY: access is limited to resuming this child's initial thread;
        // the handle is deliberately not inheritable.
        let raw = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, thread_id) };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: OpenThread returned a newly owned valid handle.
        let thread = unsafe { OwnedHandle::from_raw_handle(raw) };
        // SAFETY: only this supervised, suspended child's thread is resumed.
        let previous_count = unsafe { ResumeThread(thread.as_raw_handle()) };
        if previous_count == u32::MAX {
            return Err(io::Error::last_os_error());
        }
        if previous_count != 1 {
            return Err(io::Error::other("unexpected initial suspend count"));
        }
        Ok(())
    }

    pub(in super::super) fn spawn(command: &mut Command) -> Result<(Child, ProcessGroup), String> {
        let supervision_error = |error: io::Error| {
            format!(
                "nibiru_process_supervision_failed:{}",
                error.raw_os_error().unwrap_or(-1)
            )
        };
        let group = ProcessGroup::new().map_err(supervision_error)?;
        command.creation_flags(CREATE_NO_WINDOW | CREATE_SUSPENDED);
        let mut child = command
            .spawn()
            .map_err(|_| "nibiru_executable_unavailable".to_owned())?;
        if let Err(error) = group.attach(&child) {
            // Never run a child after failed job setup. kill_on_drop plus the
            // job guard also cover failures after assignment but before resume.
            let _ = child.start_kill();
            return Err(supervision_error(error));
        }
        Ok((child, group))
    }
}

#[cfg(windows)]
pub(super) use windows::spawn;
