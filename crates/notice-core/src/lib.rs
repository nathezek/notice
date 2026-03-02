pub mod config;
pub mod error;
pub mod types;

pub use error::Error;

/// Truncate a string to at most `max_bytes` bytes at a valid UTF-8 char boundary.
/// Never panics.
pub fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    // Walk backwards from max_bytes to find a char boundary
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii() {
        assert_eq!(truncate_utf8("hello world", 5), "hello");
    }

    #[test]
    fn truncate_multibyte() {
        // 'è' is 2 bytes (0xC3 0xA8)
        let s = "cafè";
        // c=1, a=2, f=3, è=4,5
        assert_eq!(truncate_utf8(s, 4), "caf");
        assert_eq!(truncate_utf8(s, 5), "cafè");
    }

    #[test]
    fn truncate_cjk() {
        // Each CJK character is 3 bytes
        let s = "能力テスト";
        assert_eq!(truncate_utf8(s, 3), "能");
        assert_eq!(truncate_utf8(s, 4), "能");
        assert_eq!(truncate_utf8(s, 6), "能力");
    }

    #[test]
    fn truncate_no_op() {
        assert_eq!(truncate_utf8("short", 100), "short");
    }

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate_utf8("", 10), "");
    }
}
