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
