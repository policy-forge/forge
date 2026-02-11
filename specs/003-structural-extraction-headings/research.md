# Research: Structural Extraction — Headings

**Feature Branch**: `003-structural-extraction-headings`
**Date**: 2026-02-11

## Research Summary

All technical unknowns have been resolved. No NEEDS CLARIFICATION items remain.

---

## R-1: pulldown-cmark Event API for Heading Extraction

**Decision**: Use `pulldown_cmark::Parser::new(content).into_offset_iter()` to iterate events with byte offset ranges.

**Rationale**: `into_offset_iter()` returns `Iterator<Item = (Event<'a>, Range<usize>)>` where the `Range<usize>` represents byte offsets into the source content. This is exactly what we need to compute source line numbers. The regular iterator (without `into_offset_iter()`) provides no position information.

**Alternatives considered**:
- `Parser::new(content)` without offset iter — rejected because we need byte offsets for line number computation (PRD M-2).
- `Parser::new_ext(content, Options::ENABLE_HEADING_ATTRIBUTES)` — not needed; we only need heading text and level, not id/classes/attrs.

---

## R-2: pulldown-cmark Heading Event Types (v0.13.x)

**Decision**: Use `Event::Start(Tag::Heading { level, .. })` for heading detection and `Event::End(TagEnd::Heading(_))` for heading end markers.

**Rationale**: In pulldown-cmark 0.13.x (latest stable, MIT licensed):

```rust
// Start event — struct variant with named fields
Event::Start(Tag::Heading { level, id, classes, attrs })

// End event — tuple variant with just level
Event::End(TagEnd::Heading(level))

// Text inside heading
Event::Text(cow_str)
```

Where `level` is `HeadingLevel` enum (`H1` through `H6`), not a raw integer.

**Alternatives considered**:
- pulldown-cmark 0.9.x — rejected; uses different API (`Tag::Heading(HeadingLevel, Option<&str>, Vec<&str>)` tuple variant). Version 0.13.x is current stable and required by constitution principle XI.

---

## R-3: HeadingLevel to u8 Conversion

**Decision**: Use a match expression to convert `HeadingLevel` enum to `u8` (1-6).

**Rationale**: `HeadingLevel` is an enum with variants `H1` through `H6`. It has no `#[repr]` attribute, so casting with `as u8` is not guaranteed. A simple match is the safe, idiomatic approach:

```rust
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}
```

`HeadingLevel` implements `Ord`/`PartialOrd`, so we can compare levels directly for stack operations.

**Alternatives considered**:
- Store `HeadingLevel` enum directly in `SectionNode` — rejected; the AR specifies `heading_level: u8` and downstream consumers should not need to depend on pulldown-cmark types.

---

## R-4: Byte Offset to Line Number Conversion

**Decision**: Pre-compute a line-starts table from the content, then binary-search the byte offset to find the line number.

**Rationale**: pulldown-cmark provides byte offsets via `Range<usize>` from `into_offset_iter()`. To convert to 1-based line numbers (PRD M-2), we need to count newlines before the offset. Pre-computing a `Vec<usize>` of byte positions where each line starts (by scanning for `\n`) enables O(log n) lookup per heading via `partition_point` (binary search). With typically dozens of headings and content up to 10MB, this is efficient.

```rust
fn build_line_starts(content: &str) -> Vec<usize> {
    let mut starts = vec![0]; // Line 1 starts at byte 0
    for (i, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

fn offset_to_line(offset: usize, line_starts: &[usize]) -> usize {
    // partition_point returns the first index where starts[i] > offset
    // Subtracting 1 would give 0-based; we return as-is for 1-based
    line_starts.partition_point(|&start| start <= offset)
}
```

**Alternatives considered**:
- Count newlines in `content[..offset]` for each heading — O(n * h) worst case where h = number of headings. Acceptable for small documents but the pre-computed table is simpler and faster.
- Use `IngestedDocument.lines[].number` — rejected; the ingested lines are separate from the raw content string that pulldown-cmark parses. The byte offsets from pulldown-cmark don't correspond to line indices directly.

---

## R-5: Stack-Based Tree Construction Algorithm

**Decision**: Use an explicit `Vec<(u8, SectionNode)>` stack as specified in the AR.

**Rationale**: The AR (Option 1) specifies:
1. For each `Start(Heading { level, .. })` event: pop stack until `stack.last().level < new_level`, attaching popped nodes as children of their parent (or to root list). Then push the new node.
2. Between headings: accumulate text events into `stack.last().body_text`.
3. At end: drain stack, attaching remaining nodes to parents or root list.

This handles all irregular nesting naturally:
- H1 → H3 skip: H3 is pushed with H1 still on stack, becomes child of H1
- Multiple H1s: previous H1 subtree is popped to root list when new H1 arrives
- H3 first (no preceding H1): H3 becomes a root-level node
- No headings: empty stack, empty root list

**Alternatives considered**: Options 2 (two-pass) and 3 (recursive descent) were evaluated and rejected in the AR for increased complexity without benefit.

---

## R-6: Body Text Accumulation Strategy

**Decision**: Accumulate `Event::Text`, `Event::Code`, `Event::SoftBreak`, and `Event::HardBreak` events between headings into the current section's `body_text` field.

**Rationale**: Per PRD S-1, body text between headings should be captured. The events between `End(Heading)` and the next `Start(Heading)` (or end of document) represent the section body. We track a boolean `in_heading` state to distinguish heading title text from body text.

Text accumulation:
- `Event::Text(s)` → append `s` to body
- `Event::Code(s)` → append `` `s` `` to body (preserve inline code)
- `Event::SoftBreak` → append `\n` to body
- `Event::HardBreak` → append `\n` to body
- Other events (list items, emphasis, etc.) → skip (body captures raw text, not structure)

Per assumption A-4 in the spec: text before the first heading is discarded.

**Alternatives considered**:
- Capture raw Markdown source using byte ranges — rejected; would include formatting syntax (`**bold**`, `- list`), which is harder to work with downstream. Text events give clean text content.
- Capture all events including structure (lists, emphasis) — rejected; WI-4 handles clause/list extraction separately. Over-capturing here adds complexity.

---

## R-7: pulldown-cmark Dependency Version and Safety

**Decision**: Add `pulldown-cmark = "0.13"` to `Cargo.toml`.

**Rationale**:
- Version 0.13.0 is the latest stable release (February 2025)
- MIT licensed — compatible with project MIT license
- Pure Rust, no unsafe code blocks
- Well-maintained: active development, responsive maintainers
- Already conceptually part of the project (AR references it; WI-2 AR selected it)
- Must pass `cargo audit` and `cargo deny` checks per constitution principle XI

**Alternatives considered**:
- comrak — larger, more features than needed. pulldown-cmark is the lighter, focused choice.
- markdown-rs — less mature, smaller ecosystem.

---

## R-8: Security Requirements Implementation

**Decision**: Implement SEC-1 through SEC-4 from the security review.

| SEC ID | Requirement | Implementation Approach |
|--------|-------------|------------------------|
| SEC-1 | Handle all heading level combinations without panicking | Stack-based algorithm naturally handles all combinations. Comprehensive unit tests for all edge cases. |
| SEC-2 | Empty documents return empty Vec (not error/panic) | If no heading events found, return `Ok(vec![])`. |
| SEC-3 | Use explicit stack (not call-stack recursion) | `Vec<(u8, SectionNode)>` is the stack — heap-allocated, no recursion. |
| SEC-4 | Tree depth bounded by heading levels (max 6) | `HeadingLevel` enum only has H1-H6 variants. No additional enforcement needed. |

Security finding F1 (iterative tree traversal for Debug/display): The `#[derive(Debug)]` implementation uses the standard recursive `Debug` trait. Since tree depth is bounded to 6 (SEC-4), this is safe. Any custom display implementations should use iterative traversal per the security review recommendation.
