//! Supervisor daemon implementation, in-memory state, and incident processing.

pub mod event_multiplexer;
pub mod incident_manager;
pub mod state;
pub mod supervisor;

pub use event_multiplexer::run_event_loop;
pub use incident_manager::IncidentManager;
pub use state::DaemonState;
pub use supervisor::run_supervisor;
