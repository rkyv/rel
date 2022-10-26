//! Case conversion functions.

/// Converts a name from `PascalCase` to `snake_case`.
pub fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for c in s.chars() {
        if c.is_ascii_uppercase() && !result.is_empty() {
            result.push('_');
        }
        result.push(c.to_ascii_lowercase());
    }
    result
}
