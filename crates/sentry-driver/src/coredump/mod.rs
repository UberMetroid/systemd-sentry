//! Pure Rust systemd-coredump crash and backtrace extraction.

pub mod backtrace_extractor;
pub mod dir_scanner;
pub mod journal_matcher;
pub mod xattr_reader;

pub use backtrace_extractor::extract_backtrace;
pub use dir_scanner::find_latest_coredump;
pub use journal_matcher::match_coredump_record;
pub use xattr_reader::read_coredump_xattrs;
