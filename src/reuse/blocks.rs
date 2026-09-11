//! Deterministic segment boundaries over one candidate document.
//!
//! A block is a maximal run of non-blank lines inside one heading scope. The
//! caller receives exact UTF-8 byte offsets so every candidate can be quoted
//! verbatim from the source it came from.
//!
//! The rules are deliberately narrow so that the same bytes always produce the
//! same spans:
//!
//! - a heading or code fence starts at the first byte of a line; an indented
//!   one is ordinary text;
//! - a line is blank when it is empty or holds only spaces, tabs and carriage
//!   returns, and a blank line always ends the current block;
//! - a span covers whole lines, leading and trailing whitespace included, minus
//!   the line break;
//! - nothing here allocates in proportion to the input: the block list is the
//!   only output.

use serde::{Deserialize, Serialize};

use crate::ForgeError;

/// Maximum number of blocks one document may yield.
pub const MAX_BLOCKS: usize = 4096;

/// One maximal run of non-blank lines inside a heading scope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    /// Byte offset of the first character of the first non-blank line.
    pub start: usize,
    /// Byte offset one past the last character of the last non-blank line.
    ///
    /// A trailing line break is never part of the span, so
    /// `&document[start..end]` is exactly the block text.
    pub end: usize,
    /// Heading text of the enclosing scope, with its leading `#` run, the
    /// following space, trailing spaces and an optional closing `#` run
    /// removed; `None` before the first heading of the document.
    pub scope_title: Option<String>,
}

/// Segment a document into blocks with exact UTF-8 byte spans.
///
/// Blocks are returned in document order. A heading (one to six `#` followed by
/// a space or the end of the line) opens a scope that runs until the next ATX
/// heading of any level; the heading line itself is never a block. Headings
/// inside a fenced code block (```` ``` ```` or `~~~` fences of at least three
/// characters) are ordinary text.
///
/// # Errors
///
/// Returns [`ForgeError::Authoring`] when `document` is not valid UTF-8 — the
/// corpus contract already pins candidate documents, so this is defence in
/// depth — or when the document yields more than [`MAX_BLOCKS`] blocks.
pub fn blocks(document: &[u8]) -> Result<Vec<Block>, ForgeError> {
    let text = std::str::from_utf8(document).map_err(|invalid| {
        error(format!("document is not valid UTF-8 at byte {}", invalid.valid_up_to()))
    })?;

    let mut result: Vec<Block> = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let mut scope: Option<String> = None;
    let mut fence: Option<(u8, usize)> = None;

    let mut position = 0;
    while position < text.len() {
        let line_start = position;
        let line_end = if let Some(offset) = text[position..].find('\n') {
            let newline = position + offset;
            position = newline + 1;
            if newline > line_start && text.as_bytes()[newline - 1] == b'\r' {
                newline - 1
            } else {
                newline
            }
        } else {
            position = text.len();
            text.len()
        };
        let line = &text[line_start..line_end];

        if let Some((character, length)) = fence {
            if closes_fence(line, character, length) {
                fence = None;
            }
        } else if let Some(title) = heading(line) {
            flush(&mut result, &mut open, scope.as_deref())?;
            scope = Some(title);
            continue;
        } else if let Some(opened) = opens_fence(line) {
            fence = Some(opened);
        }

        if is_blank(line) {
            flush(&mut result, &mut open, scope.as_deref())?;
        } else {
            match &mut open {
                Some((_, end)) => *end = line_end,
                None => open = Some((line_start, line_end)),
            }
        }
    }
    flush(&mut result, &mut open, scope.as_deref())?;
    Ok(result)
}

fn error(message: impl Into<String>) -> ForgeError {
    ForgeError::Authoring(message.into())
}

/// Close the current block, if any, at the given scope.
fn flush(
    out: &mut Vec<Block>,
    open: &mut Option<(usize, usize)>,
    scope: Option<&str>,
) -> Result<(), ForgeError> {
    let Some((start, end)) = open.take() else {
        return Ok(());
    };
    if out.len() >= MAX_BLOCKS {
        return Err(error(format!("document has more than {MAX_BLOCKS} blocks")));
    }
    out.push(Block { start, end, scope_title: scope.map(str::to_owned) });
    Ok(())
}

/// Whether a line carries no content: empty, or only spaces, tabs and carriage
/// returns.
fn is_blank(line: &str) -> bool {
    line.bytes().all(|byte| matches!(byte, b' ' | b'\t' | b'\r'))
}

/// Heading text of `line`, or `None` when `line` is not an ATX heading.
///
/// One to six `#` must be followed by a space or the end of the line. The title
/// loses that space, its trailing spaces, and a closing `#` run that follows a
/// space or fills the whole title.
fn heading(line: &str) -> Option<String> {
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = line.get(level..)?;
    let title = if rest.is_empty() { rest } else { rest.strip_prefix(' ')? };
    let trimmed = title.trim_end_matches(' ');
    let closing = trimmed.trim_end_matches('#');
    let title = if closing.len() == trimmed.len() {
        trimmed
    } else if closing.is_empty() || closing.ends_with(' ') {
        closing.trim_end_matches(' ')
    } else {
        trimmed
    };
    Some(title.to_owned())
}

/// The fence character and length of an opening code fence, if `line` is one.
fn opens_fence(line: &str) -> Option<(u8, usize)> {
    let character = *line.as_bytes().first()?;
    if character != b'`' && character != b'~' {
        return None;
    }
    let length = line.bytes().take_while(|byte| *byte == character).count();
    (length >= 3).then_some((character, length))
}

/// Whether `line` closes the fence it was opened with: the same character,
/// at least as many times, and nothing but spaces or tabs after it.
fn closes_fence(line: &str, character: u8, opened_with: usize) -> bool {
    let length = line.bytes().take_while(|byte| *byte == character).count();
    length >= opened_with && line[length..].bytes().all(|byte| matches!(byte, b' ' | b'\t'))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Segment `document` and assert every span by slicing it, which also
    /// proves each offset lands on a UTF-8 character boundary.
    fn texts(document: &str) -> Vec<(String, Option<String>)> {
        blocks(document.as_bytes())
            .expect("document segments")
            .into_iter()
            .map(|block| {
                assert!(
                    document.is_char_boundary(block.start),
                    "start {} is not a character boundary",
                    block.start
                );
                assert!(
                    document.is_char_boundary(block.end),
                    "end {} is not a character boundary",
                    block.end
                );
                (document[block.start..block.end].to_owned(), block.scope_title)
            })
            .collect()
    }

    fn expected(pairs: &[(&str, Option<&str>)]) -> Vec<(String, Option<String>)> {
        pairs.iter().map(|(text, scope)| ((*text).to_owned(), scope.map(str::to_owned))).collect()
    }

    #[test]
    fn heading_scopes_carry_the_stripped_heading_text() {
        let document = "# Policy\n\nintro\n\n## Access control ##\n\nalpha\n\n### Deep   \n\nbeta\n\n#### C#\n\ngamma\n";
        assert_eq!(
            texts(document),
            expected(&[
                ("intro", Some("Policy")),
                ("alpha", Some("Access control")),
                ("beta", Some("Deep")),
                ("gamma", Some("C#")),
            ])
        );
    }

    #[test]
    fn only_one_to_six_hashes_followed_by_space_are_headings() {
        let document = "\
####### seven

#no-space

#

content
";
        assert_eq!(
            texts(document),
            expected(&[("####### seven", None), ("#no-space", None), ("content", Some("")),])
        );
    }

    #[test]
    fn blocks_before_the_first_heading_have_no_scope() {
        let document = "\
preamble

before any heading

# Later

after
";
        assert_eq!(
            texts(document),
            expected(
                &[("preamble", None), ("before any heading", None), ("after", Some("Later")),]
            )
        );
    }

    #[test]
    fn a_heading_line_is_never_a_block() {
        let document = "intro\n# A\nafter\n";
        assert_eq!(texts(document), expected(&[("intro", None), ("after", Some("A"))]));
    }

    #[test]
    fn the_last_line_needs_no_trailing_newline() {
        assert_eq!(texts("# A"), Vec::new());
        assert_eq!(texts("# A\n\nfirst\nsecond"), expected(&[("first\nsecond", Some("A"))]));
    }

    #[test]
    fn fenced_code_hides_headings() {
        let document = "\
# Doc

```
# not a heading

# still code
```

tail
";
        assert_eq!(
            texts(document),
            expected(&[
                ("```\n# not a heading", Some("Doc")),
                ("# still code\n```", Some("Doc")),
                ("tail", Some("Doc")),
            ])
        );
    }

    #[test]
    fn fences_close_only_on_the_same_character_and_enough_length() {
        let document = "\
# A

~~~
# inside
```
~~~~

after

```
# open
``
";
        assert_eq!(
            texts(document),
            expected(&[
                ("~~~\n# inside\n```\n~~~~", Some("A")),
                ("after", Some("A")),
                ("```\n# open\n``", Some("A")),
            ])
        );
    }

    #[test]
    fn carriage_returns_are_not_part_of_the_span() {
        let document = "# Spec\r\n\r\ncrlf body\r\n\r\nsecond line  \r\n";
        assert_eq!(
            texts(document),
            expected(&[("crlf body", Some("Spec")), ("second line  ", Some("Spec")),])
        );
    }

    #[test]
    fn multi_byte_utf8_spans_land_on_character_boundaries() {
        let document = "# ポリシー\n\n暗号化は必須です。\n\n🔐 鍵のローテーション – täglich\n";
        let parsed = blocks(document.as_bytes()).unwrap();
        assert_eq!(parsed[0].start, "# ポリシー\n\n".len());
        assert_eq!(parsed[0].end, parsed[0].start + "暗号化は必須です。".len());
        assert!(document.is_char_boundary(parsed[0].start));
        assert!(document.is_char_boundary(parsed[0].end));
        assert_eq!(
            texts(document),
            expected(&[
                ("暗号化は必須です。", Some("ポリシー")),
                ("🔐 鍵のローテーション – täglich", Some("ポリシー")),
            ])
        );
    }

    #[test]
    fn consecutive_blank_lines_separate_blocks() {
        let document = "para one\n\n\n\npara two\n";
        assert_eq!(texts(document), expected(&[("para one", None), ("para two", None)]));
    }

    #[test]
    fn leading_and_trailing_blank_lines_yield_no_empty_blocks() {
        let document = "\n\n\n";
        assert!(texts(document).is_empty());
        assert_eq!(texts("\n\n\npara\n\n\n"), expected(&[("para", None)]));
    }

    #[test]
    fn a_document_without_headings_has_no_scope() {
        let document = "alpha\nbeta\n\n gamma\n";
        assert_eq!(texts(document), expected(&[("alpha\nbeta", None), (" gamma", None)]));
    }

    #[test]
    fn an_empty_document_has_no_blocks() {
        assert_eq!(blocks(b"").unwrap(), Vec::new());
    }

    #[test]
    fn invalid_utf8_is_rejected() {
        let rejected = blocks(b"# A\n\n\xff\xfe\n").unwrap_err();
        assert!(matches!(rejected, ForgeError::Authoring(_)), "{rejected}");
        assert!(rejected.to_string().contains("UTF-8"), "{rejected}");

        let truncated = blocks(b"# A\n\n\xe6\x97\n").unwrap_err();
        assert!(truncated.to_string().contains("UTF-8"), "{truncated}");
    }

    #[test]
    fn block_count_is_bounded() {
        let within = "a\n\n".repeat(MAX_BLOCKS);
        assert_eq!(blocks(within.as_bytes()).unwrap().len(), MAX_BLOCKS);

        let over = "a\n\n".repeat(MAX_BLOCKS + 1);
        let rejected = blocks(over.as_bytes()).unwrap_err();
        assert!(rejected.to_string().contains("4096"), "{rejected}");
    }
}
