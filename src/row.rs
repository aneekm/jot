/*
 * row.rs contains the source code for a row of text in a document.
 *
 * TODO: coloring!! at minimum, every row needs a default background color and
 *       foreground color. Using terminal defaults is ok for now but eventually
 *       I need a theming module that parses a TOML file of colors at startup
 *       and use it in Row::render()
 */

use std::cmp;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    text: String,
    len: usize,
}

impl From<&str> for Row {
    fn from(line: &str) -> Self {
        Self {
            text: line.to_string(),
            len: line.len(),
        }
    }
}

impl Row {
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.text.as_bytes()
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len {
            self.text.push(c);
            self.len += 1;
            return;
        }

        let mut new_line = String::new();
        for (index, grapheme) in self.text[..].graphemes(true).enumerate() {
            if index == at {
                if c == '\t' {
                    new_line.push_str("    ");
                } else {
                    new_line.push(c)
                }
            }
            new_line.push_str(grapheme);
        }

        self.text = new_line;
        self.len += 1;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len {
            return;
        }

        let mut new_line = String::new();
        for (index, grapheme) in self.text[..].graphemes(true).enumerate() {
            if index != at {
                new_line.push_str(grapheme);
            }
        }

        self.text = new_line;
        self.len -= 1;
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut curr_line = String::new();
        let mut curr_len = 0;
        let mut new_line = String::new();
        let mut new_len = 0;

        for (index, grapheme) in self.text[..].graphemes(true).enumerate() {
            if index < at {
                curr_line.push_str(grapheme);
                curr_len += 1;
            } else {
                new_line.push_str(grapheme);
                new_len += 1;
            }
        }

        self.text = curr_line;
        self.len = curr_len;
        Self {
            text: new_line,
            len: new_len,
        }
    }

    pub fn append(&mut self, new_row: &Self) {
        self.text = format!("{}{}", self.text, new_row.text);
        self.len += new_row.len;
    }

    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.len);
        let start = cmp::min(start, end);
        let mut rendered_string = String::new();
        // TODO: index will be useful for highlighting
        for (__index, grapheme) in self.text[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                rendered_string.push(c);
            }
        }

        rendered_string
    }
}
