//! Tier 1: Feature Coverage Integration Runner
//!
//! Executes all Tier 1 tests covering R1 through R7.

#[path = "harness_models.rs"]
pub mod harness_models;

#[path = "harness_circuit.rs"]
pub mod harness_circuit;

#[path = "harness_policy.rs"]
pub mod harness_policy;

#[path = "harness_notify.rs"]
pub mod harness_notify;

mod r1_architecture;
mod r2_integration;
mod r3_diagnostic;
mod r4_safety;
mod r5_ux;
mod r6_qa;
mod r7_packaging;
