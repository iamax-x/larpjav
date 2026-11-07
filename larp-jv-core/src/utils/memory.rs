//! Memory utilities

use crate::error::LarpResult;
use std::fs;
use std::path::Path;

/// Memory region info from /proc/self/maps
#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
    pub permissions: String,
    pub offset: usize,
    pub device: String,
    pub inode: usize,
    pub pathname: Option<String>,
}

/// Parse /proc/self/maps (Linux/Android)
pub fn parse_self_maps() -> LarpResult<Vec<MemoryRegion>> {
    let content = fs::read_to_string("/proc/self/maps")?;
    let mut regions = Vec::new();
    for line in content.lines() {
        if let Some(region) = parse_maps_line(line) {
            regions.push(region);
        }
    }
    Ok(regions)
}

fn parse_maps_line(line: &str) -> Option<MemoryRegion> {
    // Format: address perms offset dev inode pathname
    // Example: 7f8b8c000000-7f8b8c021000 rw-p 00000000 00:00 0 [heap]
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 { return None; }

    let addr_range = parts[0];
    let addrs: Vec<&str> = addr_range.split('-').collect();
    if addrs.len() != 2 { return None; }
    let start = usize::from_str_radix(addrs[0], 16).ok()?;
    let end = usize::from_str_radix(addrs[1], 16).ok()?;

    let permissions = parts[1].to_string();
    let offset = usize::from_str_radix(parts[2], 16).ok()?;
    let device = parts[3].to_string();
    let inode = parts[4].parse().ok()?;
    let pathname = if parts.len() > 5 { Some(parts[5..].join(" ")) } else { None };

    Some(MemoryRegion { start, end, permissions, offset, device, inode, pathname })
}

/// Find memory holes (gaps between regions) suitable for allocation
pub fn find_memory_holes(regions: &[MemoryRegion], min_size: usize) -> Vec<(usize, usize)> {
    let mut holes = Vec::new();
    let mut sorted = regions.to_vec();
    sorted.sort_by_key(|r| r.start);

    let mut prev_end = 0;
    for region in sorted {
        if region.start > prev_end + min_size {
            holes.push((prev_end, region.start));
        }
        prev_end = prev_end.max(region.end);
    }
    holes
}

/// Get total memory usage of current process
pub fn process_memory_usage() -> LarpResult<ProcessMemory> {
    #[cfg(target_os = "linux")]
    {
        let content = fs::read_to_string("/proc/self/status")?;
        let mut mem = ProcessMemory::default();
        for line in content.lines() {
            if line.starts_with("VmRSS:") {
                mem.rss_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("VmSize:") {
                mem.virtual_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("VmData:") {
                mem.data_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("VmStk:") {
                mem.stack_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("VmExe:") {
                mem.exe_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            } else if line.starts_with("VmLib:") {
                mem.lib_kb = line.split_whitespace().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            }
        }
        Ok(mem)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(ProcessMemory::default())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ProcessMemory {
    pub rss_kb: u64,
    pub virtual_kb: u64,
    pub data_kb: u64,
    pub stack_kb: u64,
    pub exe_kb: u64,
    pub lib_kb: u64,
}

/// Align address up to alignment
pub fn align_up(addr: usize, alignment: usize) -> usize {
    (addr + alignment - 1) & !(alignment - 1)
}

/// Align address down to alignment
pub fn align_down(addr: usize, alignment: usize) -> usize {
    addr & !(alignment - 1)
}

/// Check if address is aligned
pub fn is_aligned(addr: usize, alignment: usize) -> bool {
    addr & (alignment - 1) == 0
}

/// Page size
pub fn page_size() -> usize {
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
    }
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Memory::GetSystemInfo;
        let mut info = std::mem::zeroed();
        unsafe { GetSystemInfo(&mut info) };
        info.dwPageSize as usize
    }
    #[cfg(target_os = "macos")]
    {
        4096
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        4096
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align() {
        assert_eq!(align_up(100, 16), 112);
        assert_eq!(align_up(112, 16), 112);
        assert_eq!(align_down(100, 16), 96);
        assert_eq!(align_down(112, 16), 112);
        assert!(is_aligned(112, 16));
        assert!(!is_aligned(100, 16));
    }

    #[test]
    fn test_page_size() {
        assert!(page_size() > 0);
        assert!(page_size().is_power_of_two());
    }
}
