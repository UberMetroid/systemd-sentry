//! Pure Rust systemd D-Bus listener and client via `zbus`.

pub mod error;
pub mod event;
pub mod listener;
pub mod manager_client;
pub mod match_rules;
pub mod path_escape;
pub mod path_unescape;
pub mod property_decoder;

pub use error::DbusDriverError;
pub use event::{DbusEvent, UnitFailedEvent, UnitStateUpdate};
pub use listener::SystemdDbusListener;
pub use manager_client::{
    call_systemd_unit_method, get_service_properties, get_unit_properties, subscribe_manager,
};
pub use match_rules::{build_manager_match_rule, build_unit_properties_match_rule};
pub use path_escape::{escape_unit_name, unit_name_to_object_path};
pub use path_unescape::{object_path_to_unit_name, unescape_unit_name};
pub use property_decoder::{decode_unit_properties, extract_unit_failed_event};
