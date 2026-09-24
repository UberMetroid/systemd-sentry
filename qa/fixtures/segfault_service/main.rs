//! segfault_service — Test fixture intentionally triggering SIGSEGV
//!
//! Used by systemd-sentry E2E tests to verify coredump extraction,
//! signal detection, and administrative escalation triage.

use std::env;
use std::thread;
use std::time::Duration;

fn parse_delay_ms() -> u64 {
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--delay-ms" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<u64>() {
                return val;
            }
        }
    }
    0
}

fn trigger_segfault() -> ! {
    eprintln!("[segfault_service] Intentionally dereferencing null pointer...");
    unsafe {
        let null_ptr = std::ptr::null_mut::<i32>();
        std::ptr::write_volatile(null_ptr, 42);
    }
    // Fallback if OS didn't catch the write
    std::process::abort();
}

fn main() {
    println!("[segfault_service] Starting fixture process PID {}", std::process::id());
    let delay_ms = parse_delay_ms();
    if delay_ms > 0 {
        println!("[segfault_service] Sleeping for {} ms before crash", delay_ms);
        thread::sleep(Duration::from_millis(delay_ms));
    }
    trigger_segfault();
}
