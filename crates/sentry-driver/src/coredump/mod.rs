//! Pure Rust systemd-coredump crash and backtrace extraction.

pub mod backtrace_extractor;
pub mod dir_scanner;
pub mod elf_extractor;
pub mod elf_header_parser;
pub mod elf_note_parser;
pub mod journal_matcher;
pub mod stream_reader;
pub mod xattr_reader;

pub use backtrace_extractor::extract_backtrace;
pub use dir_scanner::find_latest_coredump;
pub use elf_extractor::{extract_elf_crash_headers, extract_elf_crash_headers_from_file};
pub use elf_header_parser::{locate_note_segments, parse_elf_header, ElfHeaderInfo};
pub use elf_note_parser::{parse_elf_notes, ElfNotesInfo};
pub use journal_matcher::match_coredump_record;
pub use stream_reader::{
    lz4_flex, open_bounded_decompressed_reader, read_bounded_coredump_bytes, BoundedStreamReader,
    MAX_DECOMPRESSED_BYTES,
};
pub use xattr_reader::read_coredump_xattrs;
pub use sentry_core::models::coredump::{CoredumpRecord, CoredumpXattrs, ElfCrashHeader};
