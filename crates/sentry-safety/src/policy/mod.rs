//! Declarative policy subsystem and zero-trust gatekeeper.

pub mod defaults;
pub mod dropin_merger;
pub mod file_loader;
pub mod gatekeeper;
pub mod model;

pub use defaults::DEFAULT_PROTECTED_UNITS;
pub use dropin_merger::load_policy_with_dropins;
pub use file_loader::{load_policy_file, MAX_POLICY_FILE_SIZE};
pub use gatekeeper::PolicyGatekeeper;
pub use model::{GlobalPolicy, PolicyConfig, UnitPolicyOverride};
