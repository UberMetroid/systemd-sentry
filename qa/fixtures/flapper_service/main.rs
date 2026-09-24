//! flapper_service — Test fixture simulating an unstable flapping service
//!
//! Sends READY=1 heartbeat to $NOTIFY_SOCKET (if present), remains active for
//! a brief duration (--live-ms, default 50ms), and exits with failure status 1.

use std::env;
use std::os::unix::net::UnixDatagram;
use std::thread;
use std::time::Duration;

fn parse_live_ms() -> u64 {
    let args: Vec<String> = env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--live-ms" && i + 1 < args.len() {
            if let Ok(val) = args[i + 1].parse::<u64>() {
                return val;
            }
        }
    }
    50
}

fn emit_ready_notification() {
    if let Ok(socket_path) = env::var("NOTIFY_SOCKET") {
        if let Ok(socket) = UnixDatagram::unbound() {
            let msg = b"READY=1\nSTATUS=Flapper initialized\n";
            if let Some(abstract_name) = socket_path.strip_prefix('@') {
                use std::os::linux::net::SocketAddrExt;
                if let Ok(addr) = std::os::unix::net::SocketAddr::from_abstract_name(abstract_name) {
                    let _ = socket.send_to_addr(msg, &addr);
                }
            } else {
                let _ = socket.send_to(msg, socket_path);
            }
        }
    }
}

fn main() {
    println!("[flapper_service] PID {} starting up...", std::process::id());
    emit_ready_notification();
    let live_ms = parse_live_ms();
    println!("[flapper_service] Alive for {} ms, then exiting with failure", live_ms);
    thread::sleep(Duration::from_millis(live_ms));
    println!("[flapper_service] Intentional failure exit(1)");
    std::process::exit(1);
}
