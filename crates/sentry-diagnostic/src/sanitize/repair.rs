//! Heuristic repair for truncated or malformed JSON payloads.

/// Repairs truncated or slightly malformed JSON text.
///
/// Handles:
/// 1. Unclosed string literals by appending a closing quote.
/// 2. Trailing commas before closing braces/brackets or EOF.
/// 3. Missing closing braces (`}`) and brackets (`]`) caused by token limits.
pub fn repair_json(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return "{}".to_string();
    }

    let mut result = String::with_capacity(trimmed.len() + 32);
    let mut in_string = false;
    let mut is_escaped = false;
    let mut delimiter_stack = Vec::new();

    for ch in trimmed.chars() {
        if in_string {
            result.push(ch);
            if is_escaped {
                is_escaped = false;
            } else if ch == '\\' {
                is_escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else {
            match ch {
                '"' => {
                    in_string = true;
                    is_escaped = false;
                    result.push(ch);
                }
                '{' => {
                    delimiter_stack.push('}');
                    result.push(ch);
                }
                '[' => {
                    delimiter_stack.push(']');
                    result.push(ch);
                }
                '}' => {
                    clean_trailing_comma(&mut result);
                    if let Some(pos) = delimiter_stack.iter().rposition(|&c| c == '}') {
                        delimiter_stack.remove(pos);
                    }
                    result.push(ch);
                }
                ']' => {
                    clean_trailing_comma(&mut result);
                    if let Some(pos) = delimiter_stack.iter().rposition(|&c| c == ']') {
                        delimiter_stack.remove(pos);
                    }
                    result.push(ch);
                }
                _ => {
                    result.push(ch);
                }
            }
        }
    }

    // If string was truncated mid-quote, close it
    if in_string {
        result.push('"');
    }

    // Clean trailing comma at end
    clean_trailing_comma(&mut result);

    // Close remaining open brackets in LIFO order
    while let Some(closing_char) = delimiter_stack.pop() {
        result.push(closing_char);
    }

    result
}

fn clean_trailing_comma(buf: &mut String) {
    while let Some(last_char) = buf.chars().next_back() {
        if last_char.is_whitespace() || last_char == ',' {
            buf.pop();
        } else {
            break;
        }
    }
}
