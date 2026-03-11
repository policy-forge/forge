/// Strip ASCII control characters (0x00-0x1F, excluding 0x0A newline and 0x09 tab)
/// and DEL (0x7F) from a string. Used to prevent terminal escape injection (SEC-5).
#[must_use]
pub fn strip_control_chars(s: &str) -> String {
    s.chars()
        .filter(|&c| {
            let b = c as u32;
            // Keep everything except control chars 0x00-0x1F and DEL 0x7F, but preserve tab (0x09) and newline (0x0A)
            (b >= 0x20 && b != 0x7F) || b == 0x09 || b == 0x0A
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_escape() {
        let input = "Hello\x1b[31m World\x1b[0m";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello[31m World[0m");
    }

    #[test]
    fn strip_preserves_tab_and_newline() {
        let input = "Hello\tWorld\nNext line";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello\tWorld\nNext line");
    }

    #[test]
    fn strip_clean_string_unchanged() {
        let input = "Hello World";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello World");
    }

    #[test]
    fn strip_empty_string() {
        let result = strip_control_chars("");
        assert_eq!(result, "");
    }

    #[test]
    fn strip_del_character() {
        let input = "Hello\x7FWorld";
        let result = strip_control_chars(input);
        assert_eq!(result, "HelloWorld");
    }
}
