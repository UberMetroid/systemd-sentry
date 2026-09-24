//! Default policies and critical protected unit baselines.

use crate::policy::model::{GlobalPolicy, PolicyConfig};
use std::collections::HashMap;

/// Builtin critical Linux system units that are protected by default from automated modification.
pub const DEFAULT_PROTECTED_UNITS: &[&str] = &[
    "systemd-journald.service",
    "systemd-logind.service",
    "systemd-udevd.service",
    "systemd-resolved.service",
    "dbus.service",
    "sshd.service",
    "ssh.service",
    "init.scope",
    "systemd-sentry.service",
];

impl Default for PolicyConfig {
    fn default() -> Self {
        let protected = DEFAULT_PROTECTED_UNITS
            .iter()
            .map(|&s| s.to_string())
            .collect();

        Self {
            global: GlobalPolicy {
                protected_units: protected,
                ..GlobalPolicy::default()
            },
            units: HashMap::new(),
        }
    }
}
