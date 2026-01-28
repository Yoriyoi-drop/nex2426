use std::fs;
use std::time::Instant;

/// Basic Anti-Debugging Checks.
/// NOTE: In a real scenario, these are just hurdles, not absolute protection.

pub fn check_tracer() -> bool {
    // Check for TracerPid in /proc/self/status (Linux specific)
    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("TracerPid:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(pid_str) = parts.get(1) {
                    if let Ok(pid) = pid_str.parse::<i32>() {
                        if pid != 0 {
                            return true; // Being traced!
                        }
                    }
                }
            }
        }
    }
    false
}

/// Timing check to detect single-stepping (Debugger)
pub fn check_timing_anomaly() -> bool {
    let start = Instant::now();
    
    // Perform some trivial operation
    let mut x = 0;
    for i in 0..10_000 {
        x += i;
    }
    std::hint::black_box(x);
    
    let duration = start.elapsed();
    
    // If 10,000 trivial additions take more than 10ms, something is likely hooking or stepping
    // Adjusted threshold for general purpose
    duration.as_millis() > 10
}
