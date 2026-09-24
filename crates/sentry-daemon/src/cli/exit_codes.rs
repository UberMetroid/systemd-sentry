//! Standard Unix sysexits exit codes.
//!
//! Provides deterministic process exit statuses conforming to BSD sysexits(3).

/// Successful termination.
pub const EX_OK: i32 = 0;

/// The command was used incorrectly (e.g. invalid flag or argument count).
pub const EX_USAGE: i32 = 64;

/// The input data was incorrect in some way (e.g. incident not found).
pub const EX_DATAERR: i32 = 65;

/// A service or resource was unavailable (e.g. daemon not running).
pub const EX_UNAVAILABLE: i32 = 69;

/// An internal software error was detected.
pub const EX_SOFTWARE: i32 = 70;

/// An input/output error occurred while reading or writing files or sockets.
pub const EX_IOERR: i32 = 74;

/// Permission denied (e.g. SO_PEERCRED check failed for non-root caller).
pub const EX_NOPERM: i32 = 77;

/// Configuration error (e.g. invalid TOML syntax or policy violation).
pub const EX_CONFIG: i32 = 78;
