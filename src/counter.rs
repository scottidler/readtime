use eyre::{Context, Result};
use std::fs;
use std::path::Path;

/// Count words in a text file
pub fn count_words<P: AsRef<Path>>(path: P) -> Result<usize> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read file: {}", path.display()))?;

    // Detect if this looks like markdown and strip formatting if so
    let is_markdown = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "md" || ext == "markdown")
        .unwrap_or(false);

    if is_markdown {
        Ok(count_markdown_words(&content))
    } else {
        Ok(count_plain_words(&content))
    }
}

/// Count words in plain text (simple whitespace split)
fn count_plain_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Count words in markdown, stripping formatting
fn count_markdown_words(text: &str) -> usize {
    let mut cleaned = String::with_capacity(text.len());
    let mut in_code_block = false;
    let mut in_inline_code = false;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        // Handle code blocks (``` or ~~~)
        if ch == '`' || ch == '~' {
            let mut count = 1;
            let marker = ch;
            while chars.peek() == Some(&marker) {
                chars.next();
                count += 1;
            }

            if count >= 3 {
                // Toggle code block state
                in_code_block = !in_code_block;
                continue;
            } else if count == 1 && marker == '`' {
                // Toggle inline code state
                in_inline_code = !in_inline_code;
                continue;
            }
        }

        // Skip content in code blocks or inline code
        if in_code_block || in_inline_code {
            continue;
        }

        // Handle markdown links [text](url) -> keep text
        if ch == '[' {
            let mut link_text = String::new();
            let mut found_closing = false;

            // Collect the link text
            for c in chars.by_ref() {
                if c == ']' {
                    found_closing = true;
                    break;
                }
                link_text.push(c);
            }

            // Check if it's followed by (url)
            if found_closing && chars.peek() == Some(&'(') {
                chars.next(); // consume '('
                // Skip until closing ')'
                for c in chars.by_ref() {
                    if c == ')' {
                        break;
                    }
                }
                // Keep the link text
                cleaned.push_str(&link_text);
                cleaned.push(' ');
                continue;
            } else {
                // Not a link, restore the bracket and text
                cleaned.push('[');
                cleaned.push_str(&link_text);
                if found_closing {
                    cleaned.push(']');
                }
                continue;
            }
        }

        // Handle heading markers at start of line
        if ch == '#' {
            // Check if we're at the start of a line (or after whitespace)
            let prev_is_newline = cleaned.is_empty() || cleaned.ends_with('\n');
            if prev_is_newline {
                // Skip all # characters and the following space
                while chars.peek() == Some(&'#') {
                    chars.next();
                }
                if chars.peek() == Some(&' ') {
                    chars.next();
                }
                continue;
            }
        }

        // Keep everything else
        cleaned.push(ch);
    }

    count_plain_words(&cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_words() {
        assert_eq!(count_plain_words("hello world"), 2);
        assert_eq!(count_plain_words("  hello   world  "), 2);
        assert_eq!(count_plain_words(""), 0);
        assert_eq!(count_plain_words("one two three four"), 4);
    }

    #[test]
    fn test_markdown_headings() {
        assert_eq!(count_markdown_words("# Hello World"), 2);
        assert_eq!(count_markdown_words("## Hello World"), 2);
        assert_eq!(count_markdown_words("### Hello World"), 2);
    }

    #[test]
    fn test_markdown_code_blocks() {
        let text = "Some text\n```\ncode here\n```\nmore text";
        assert_eq!(count_markdown_words(text), 4); // "Some text more text"
    }

    #[test]
    fn test_markdown_inline_code() {
        assert_eq!(count_markdown_words("Some `code` here"), 2); // "Some here"
    }

    #[test]
    fn test_markdown_links() {
        assert_eq!(count_markdown_words("[click here](http://example.com)"), 2); // "click here"
        assert_eq!(count_markdown_words("Visit [my site](url) today"), 4); // "Visit my site today"
    }

    #[test]
    fn test_markdown_mixed() {
        let text = r#"# Title

Some introduction text.

## Section

Here is [a link](http://example.com) and some `code`.

```rust
fn main() {}
```

Final paragraph."#;

        let word_count = count_markdown_words(text);
        // Should count: Title, Some, introduction, text, Section, Here, is, a, link, and, some, Final, paragraph
        // = 13 words (excluding code block and inline code)
        assert!(
            (10..=15).contains(&word_count),
            "Word count was {}",
            word_count
        );
    }
}
