//! Outermost JSON bracket boundary extractor.

/// Slices the outermost JSON object between the first `{` and last `}`.
///
/// Returns `None` if no matching pair of braces exists.
pub fn slice_outermost_json(input: &str) -> Option<&str> {
    let first = input.find('{')?;
    let last = input.rfind('}')?;

    if first <= last {
        Some(&input[first..=last])
    } else {
        None
    }
}
