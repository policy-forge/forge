/// Strip control characters (C0 0x00-0x1F excluding 0x0A newline and 0x09 tab,
/// DEL 0x7F, and the C1 block U+0080-U+009F) from a string. Used to prevent
/// terminal escape injection (SEC-5), including C1 CSI/OSC sequences.
///
/// Tab and newline are deliberately preserved. Callers feeding untrusted input into single-line
/// surfaces (log entries or status bars) must additionally collapse tabs and newlines, because
/// embedded line feeds can forge extra plausible output lines.
#[must_use]
pub fn strip_control_chars(s: &str) -> String {
    s.chars().filter(|&c| !c.is_control() || c == '\t' || c == '\n').collect()
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
    fn strip_del_and_c1_controls() {
        let input = "Hello\x7F\u{009B}31m\u{0085}World";
        let result = strip_control_chars(input);
        assert_eq!(result, "Hello31mWorld");
    }

    #[test]
    fn strip_control_character_boundaries() {
        assert_eq!(strip_control_chars("a\u{0}\r\u{b}\u{c}\u{1}b\u{1f} c"), "ab c");
    }

    #[test]
    fn strip_preserves_non_ascii_text() {
        assert_eq!(strip_control_chars("中文 café"), "中文 café");
    }
}
