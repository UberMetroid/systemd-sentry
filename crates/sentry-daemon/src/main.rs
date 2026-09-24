//! Main executable entrypoint for `systemd-sentry`.

use sentry_daemon::entrypoint;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let exit_code = entrypoint(&args).await;
    std::process::exit(exit_code);
}
