//! oom_service — Test fixture allocating memory aggressively
//!
//! Allocates memory dirtying pages to force real physical page allocation
//! until terminated by cgroup v2 memory limit or Linux kernel OOM killer (SIGKILL).

use std::env;
use std::thread;
use std::time::Duration;

fn parse_args() -> (usize, u64) {
    let args: Vec<String> = env::args().collect();
    let mut chunk_mb = 1usize;
    let mut interval_ms = 5u64;

    for i in 0..args.len() {
        if args[i] == "--chunk-mb" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<usize>() {
                chunk_mb = val;
            }
        } else if args[i] == "--interval-ms" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<u64>() {
                interval_ms = val;
            }
        }
    }
    (chunk_mb, interval_ms)
}

fn dirty_allocate_loop(chunk_mb: usize, interval_ms: Duration) -> ! {
    let chunk_bytes = chunk_mb * 1024 * 1024;
    let mut pool: Vec<Vec<u8>> = Vec::new();
    let mut total_allocated_mb: usize = 0;

    println!("[oom_service] Commencing aggressive memory allocation...");
    loop {
        let mut buffer = vec![0u8; chunk_bytes];
        // Dirty every 4096-byte page to ensure kernel allocates physical frames
        for page_offset in (0..chunk_bytes).step_by(4096) {
            buffer[page_offset] = 0xAA;
        }
        pool.push(buffer);
        total_allocated_mb += chunk_mb;

        if total_allocated_mb % 10 == 0 {
            println!("[oom_service] Allocated and dirtied {} MB", total_allocated_mb);
        }

        if interval_ms.as_millis() > 0 {
            thread::sleep(interval_ms);
        }
    }
}

fn main() {
    println!("[oom_service] Starting fixture process PID {}", std::process::id());
    let (chunk_mb, interval_ms) = parse_args();
    dirty_allocate_loop(chunk_mb, Duration::from_millis(interval_ms));
}
