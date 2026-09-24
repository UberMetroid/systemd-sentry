//! Systemd-style drop-in policy loader and merger (`/etc/systemd-sentry/policy.d/*.toml`).

use crate::policy::file_loader::load_policy_file;
use crate::policy::model::PolicyConfig;
use std::fs;
use std::path::Path;
use tracing::warn;

/// Loads base policy and lexicographically overlays all valid `.toml` drop-ins from directory.
pub fn load_policy_with_dropins(base_path: &Path, dropin_dir: &Path) -> PolicyConfig {
    let mut policy = if base_path.exists() {
        match load_policy_file(base_path) {
            Ok(p) => p,
            Err(e) => {
                warn!("Base policy file at {} is invalid: {e}; using defaults", base_path.display());
                PolicyConfig::default()
            }
        }
    } else {
        PolicyConfig::default()
    };

    if !dropin_dir.is_dir() {
        return policy;
    }

    let read_dir = match fs::read_dir(dropin_dir) {
        Ok(rd) => rd,
        Err(e) => {
            warn!("Failed reading drop-in directory {}: {e}", dropin_dir.display());
            return policy;
        }
    };

    let mut dropin_paths = Vec::new();
    for entry in read_dir.flatten() {
        let path = entry.path();
        if is_valid_dropin(&path) {
            dropin_paths.push(path);
        }
    }

    // Sort paths in standard systemd alphanumeric order (e.g. 10-base.toml, 20-override.toml)
    dropin_paths.sort();

    for path in dropin_paths {
        match load_policy_file(&path) {
            Ok(dropin) => merge_policy(&mut policy, dropin),
            Err(e) => warn!("Skipping invalid drop-in policy at {}: {e}", path.display()),
        }
    }

    policy
}

fn is_valid_dropin(path: &Path) -> bool {
    let filename = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };

    // Ignore dotfiles, swap files, editor backup files, and non-toml extensions
    if filename.starts_with('.') || filename.ends_with('~') || filename.ends_with(".swp") {
        return false;
    }

    path.extension().and_then(|ext| ext.to_str()) == Some("toml")
}

fn merge_policy(target: &mut PolicyConfig, source: PolicyConfig) {
    // Append any newly protected units without duplicating
    for unit in source.global.protected_units {
        if !target.global.protected_units.contains(&unit) {
            target.global.protected_units.push(unit);
        }
    }

    // Merge allowed actions if explicitly provided in drop-in
    if !source.global.allowed_actions.is_empty() {
        for action in source.global.allowed_actions {
            if !target.global.allowed_actions.contains(&action) {
                target.global.allowed_actions.push(action);
            }
        }
    }

    // Override per-unit policies
    for (unit, override_policy) in source.units {
        target.units.insert(unit, override_policy);
    }
}
