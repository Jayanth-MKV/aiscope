//! Layer 1 — Markdown → `Statement` extraction.
//!
//! Replaces the line-based ad-hoc parser in [`crate::scanner`] with a real
//! CommonMark AST traversal. Output: one [`Statement`] per atomic instruction
//! (bullet list item or paragraph), carrying byte offsets and line numbers
//! for compiler-grade diagnostics.
//!
//! Skipped: headings, code blocks, HTML, YAML front-matter, blockquotes,
//! and link/image targets (only their visible text is kept).

use crate::model::Statement;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

/// Parse a markdown document into atomic statements.
///
/// `source_index` is propagated into every produced statement so they can be
/// joined back to their `Source` in the bundle.
pub fn parse(source_index: usize, text: &str) -> Vec<Statement> {
    let stripped = strip_frontmatter(text);
    let frontmatter_offset = text.len() - stripped.len();

    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(stripped, opts).into_offset_iter();

    let mut out = Vec::new();
    let mut buf = String::new();
    let mut current_span: Option<(usize, usize)> = None;
    let mut depth_skip: u32 = 0; // suppress code/html/heading content
    let mut in_collectable = false;
    let line_index = LineIndex::new(text);

    for (event, range) in parser {
        let abs_start = range.start + frontmatter_offset;
        let abs_end = range.end + frontmatter_offset;

        match event {
            Event::Start(Tag::Item) | Event::Start(Tag::Paragraph) => {
                // Flush any pending buffer (handles nested list items where the
                // outer item's text would otherwise be clobbered when the inner
                // item starts).
                if in_collectable && depth_skip == 0 {
                    let trimmed = buf.trim();
                    if !trimmed.is_empty() {
                        let (start, end) = current_span.unwrap_or((abs_start, abs_end));
                        let line = line_index.line_of(start);
                        out.push(Statement {
                            source_index,
                            text: trimmed.to_string(),
                            byte_start: start,
                            byte_end: end,
                            line,
                        });
                    }
                }
                in_collectable = true;
                buf.clear();
                current_span = Some((abs_start, abs_end));
            }
            Event::End(TagEnd::Item) | Event::End(TagEnd::Paragraph) => {
                if in_collectable && depth_skip == 0 {
                    let trimmed = buf.trim();
                    if !trimmed.is_empty() {
                        let (start, _) = current_span.unwrap_or((abs_start, abs_end));
                        let line = line_index.line_of(start);
                        out.push(Statement {
                            source_index,
                            text: trimmed.to_string(),
                            byte_start: start,
                            byte_end: abs_end,
                            line,
                        });
                    }
                }
                in_collectable = false;
                buf.clear();
                current_span = None;
            }

            // Skip code/html/heading/blockquote bodies entirely.
            Event::Start(Tag::CodeBlock(_))
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            })
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            })
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H3,
                ..
            })
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H4,
                ..
            })
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H5,
                ..
            })
            | Event::Start(Tag::Heading {
                level: HeadingLevel::H6,
                ..
            })
            | Event::Start(Tag::HtmlBlock)
            | Event::Start(Tag::BlockQuote(_)) => {
                depth_skip += 1;
            }
            Event::End(TagEnd::CodeBlock)
            | Event::End(TagEnd::Heading(_))
            | Event::End(TagEnd::HtmlBlock)
            | Event::End(TagEnd::BlockQuote(_)) => {
                depth_skip = depth_skip.saturating_sub(1);
            }

            // Inline code: keep its text (it's often part of a rule like `tabs`).
            Event::Code(text) if in_collectable && depth_skip == 0 => {
                if !buf.is_empty() && !buf.ends_with(' ') {
                    buf.push(' ');
                }
                buf.push_str(&text);
            }

            // Plain text content.
            Event::Text(text) if in_collectable && depth_skip == 0 => {
                buf.push_str(&text);
            }

            // Soft and hard line breaks inside a paragraph collapse to a space.
            Event::SoftBreak | Event::HardBreak
                if in_collectable && depth_skip == 0 && !buf.ends_with(' ') =>
            {
                buf.push(' ');
            }

            _ => {}
        }
    }

    out
}

/// Remove a leading YAML front-matter block (`---\n...\n---\n`) and return
/// the rest. Byte offsets in the returned slice refer to the trimmed content;
/// callers add the offset back to get absolute positions.
fn strip_frontmatter(text: &str) -> &str {
    if !text.starts_with("---") {
        return text;
    }
    let after_first = &text[3..];
    let nl = match after_first.find('\n') {
        Some(n) => n + 1,
        None => return text,
    };
    let body = &after_first[nl..];
    let close = match body.find("\n---") {
        Some(n) => n,
        None => return text,
    };
    let after_close = &body[close + 4..];
    let consume = match after_close.find('\n') {
        Some(n) => n + 1,
        None => after_close.len(),
    };
    &after_close[consume..]
}

/// Pre-built map from byte offset → 1-based line number.
struct LineIndex {
    starts: Vec<usize>,
}

impl LineIndex {
    fn new(text: &str) -> Self {
        let mut starts = vec![0];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { starts }
    }

    fn line_of(&self, byte_offset: usize) -> usize {
        match self.starts.binary_search(&byte_offset) {
            Ok(i) => i + 1,
            Err(i) => i.max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_bullets_and_skips_headings_and_code() {
        let md = "# Title\n\n- Use camelCase for variables.\n- Co-locate tests as `*.test.ts`.\n\n```ts\nlet x = 1;\n```\n\nA standalone paragraph rule.\n";
        let s = parse(0, md);
        assert_eq!(s.len(), 3);
        assert_eq!(s[0].text, "Use camelCase for variables.");
        assert_eq!(s[1].text, "Co-locate tests as *.test.ts.");
        assert_eq!(s[2].text, "A standalone paragraph rule.");
    }

    #[test]
    fn skips_yaml_frontmatter() {
        let md = "---\ntitle: Rules\n---\n\n- Use snake_case.\n";
        let s = parse(0, md);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].text, "Use snake_case.");
    }

    #[test]
    fn line_numbers_are_one_based() {
        let md = "# Title\n\n- First\n- Second\n";
        let s = parse(0, md);
        assert_eq!(s[0].line, 3);
        assert_eq!(s[1].line, 4);
    }

    #[test]
    fn inline_code_is_preserved() {
        let md = "- Prefer `pnpm` over `npm`.\n";
        let s = parse(0, md);
        assert!(s[0].text.contains("pnpm"));
        assert!(s[0].text.contains("npm"));
    }

    #[test]
    fn handles_nested_lists() {
        let md = "- Outer rule\n  - Inner detail\n  - Inner two\n- Second outer\n";
        let s = parse(0, md);
        assert!(s.len() >= 4);
    }
}
