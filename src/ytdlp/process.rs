//! 자식 프로세스를 창 없이 띄우고, 앱이 끝나면 함께 끝나게 묶는다.

use std::io::{self, Read};
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Windows 작업 개체. 이 값을 버리면(앱이 끝나면) 안에 든 yt-dlp 와 그 아래 ffmpeg 가 모두 끝난다.
pub struct KillGroup {
    job: HANDLE,
}

// 작업 개체 핸들은 어느 스레드에서 써도 된다.
unsafe impl Send for KillGroup {}
unsafe impl Sync for KillGroup {}

impl KillGroup {
    pub fn new() -> io::Result<Self> {
        // SAFETY: 이름 없는 작업 개체를 만들고, 크기를 맞춘 구조체로 한도를 정한다.
        unsafe {
            let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if job.is_null() {
                return Err(io::Error::last_os_error());
            }
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            let ok = SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&info).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            );
            if ok == 0 {
                let err = io::Error::last_os_error();
                CloseHandle(job);
                return Err(err);
            }
            Ok(Self { job })
        }
    }

    /// 창 없이, 출력을 파이프로 받게 띄워 이 그룹에 넣는다.
    pub fn spawn(&self, cmd: &mut Command) -> io::Result<Child> {
        let child = cmd
            .creation_flags(CREATE_NO_WINDOW)
            // yt-dlp(파이썬)가 한글을 UTF-8 로 쓰게 한다
            .env("PYTHONUTF8", "1")
            .env("PYTHONIOENCODING", "utf-8")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // SAFETY: 방금 띄운 자식의 핸들은 child 가 살아 있는 동안 유효하다.
        // 실패해도(이미 끝났을 때 등) 받기는 계속한다. 그 프로세스만 앱보다 오래 남을 수 있다.
        unsafe {
            AssignProcessToJobObject(self.job, child.as_raw_handle() as HANDLE);
        }
        Ok(child)
    }
}

impl Drop for KillGroup {
    fn drop(&mut self) {
        // SAFETY: new 에서 만든 핸들이고 한 번만 닫는다.
        unsafe {
            CloseHandle(self.job);
        }
    }
}

/// 끝까지 돌린 결과.
pub struct Output {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// stderr 를 다른 스레드에서 모은다. 파이프가 차서 자식이 멈추지 않게.
pub(crate) fn drain_stderr(child: &mut Child) -> std::thread::JoinHandle<String> {
    let stderr = child.stderr.take();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stderr {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// 끝날 때까지 돌리고 출력을 모은다. `timeout` 을 넘기면 죽이고 `None`.
pub fn run(group: &KillGroup, cmd: &mut Command, timeout: Option<Duration>) -> io::Result<Option<Output>> {
    let mut child = group.spawn(cmd)?;
    let stderr = drain_stderr(&mut child);
    let stdout = child.stdout.take();
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = stdout {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    });
    let status = match timeout {
        None => child.wait()?,
        Some(limit) => {
            let started = Instant::now();
            loop {
                if let Some(status) = child.try_wait()? {
                    break status;
                }
                if started.elapsed() > limit {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Ok(None);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    };
    Ok(Some(Output {
        success: status.success(),
        stdout: reader.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    }))
}
