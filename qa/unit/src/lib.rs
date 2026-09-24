//! 1:1 Unit QA test suite for systemd-sentry.

#[cfg(test)]
pub mod activation;
#[cfg(test)]
pub mod cgroup;
#[cfg(test)]
pub mod coredump;
#[cfg(test)]
pub mod dbus;
#[cfg(test)]
pub mod journal;
#[cfg(test)]
pub mod logind;
#[cfg(test)]
pub mod notify;
#[cfg(test)]
pub mod psi;
#[cfg(test)]
pub mod diagnostic;
#[cfg(test)]
pub mod mcp;
#[cfg(test)]
pub mod safety;
#[cfg(test)]
pub mod daemon;
#[cfg(test)]
pub mod inhibit;
#[cfg(test)]
pub mod networkd;
#[cfg(test)]
pub mod pstore;
#[cfg(test)]
pub mod resolved;
#[cfg(test)]
pub mod timesync;
#[cfg(test)]
pub mod user_session;
