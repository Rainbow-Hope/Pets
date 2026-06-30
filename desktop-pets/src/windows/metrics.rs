use crate::behavior::mood::ResourceSample;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuTimes {
    pub idle: u64,
    pub kernel: u64,
    pub user: u64,
}

pub fn cpu_percent(previous: CpuTimes, current: CpuTimes) -> Option<f32> {
    let idle = current.idle.checked_sub(previous.idle)?;
    let kernel = current.kernel.checked_sub(previous.kernel)?;
    let user = current.user.checked_sub(previous.user)?;
    let total = kernel.checked_add(user)?;
    if total == 0 || idle > total {
        return None;
    }
    Some(((total - idle) as f64 * 100.0 / total as f64) as f32)
}

#[cfg(windows)]
#[derive(Debug, Default)]
pub struct SystemMetrics {
    previous: Option<CpuTimes>,
}

#[cfg(windows)]
impl SystemMetrics {
    pub fn sample(&mut self) -> Option<ResourceSample> {
        use std::mem::size_of;
        use windows_sys::Win32::Foundation::FILETIME;
        use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        use windows_sys::Win32::System::Threading::GetSystemTimes;

        let mut idle = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        // SAFETY: all three pointers refer to initialized writable FILETIME values.
        if unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) } == 0 {
            return None;
        }
        let current = CpuTimes {
            idle: filetime_to_u64(idle),
            kernel: filetime_to_u64(kernel),
            user: filetime_to_u64(user),
        };
        let cpu = self
            .previous
            .and_then(|previous| cpu_percent(previous, current))
            .unwrap_or(0.0);
        self.previous = Some(current);

        let mut memory = MEMORYSTATUSEX {
            dwLength: size_of::<MEMORYSTATUSEX>() as u32,
            ..MEMORYSTATUSEX::default()
        };
        // SAFETY: memory has the required dwLength and points to writable storage.
        if unsafe { GlobalMemoryStatusEx(&mut memory) } == 0 {
            return None;
        }
        Some(ResourceSample {
            cpu_percent: cpu,
            memory_percent: memory.dwMemoryLoad as f32,
        })
    }
}

#[cfg(windows)]
fn filetime_to_u64(value: windows_sys::Win32::Foundation::FILETIME) -> u64 {
    (u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)
}
