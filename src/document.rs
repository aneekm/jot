/*
 * document.rs contains the source code for the document representation in jot.
 *
 * Document currently uses a row-based implementation to represent text. The
 * API is presented such that an editor using cursor position to interact will
 * not need to know the underlying implementation if it ever changes (eg. to
 * ropes or rrb-trees).
 */

use crate::{Position, Row};
use std::fs;
use std::io::Write;

pub struct Document {
    rows: Vec<Row>,
    filename: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn default() -> Self {
        Self {
            rows: vec![Row::default()],
            filename: None,
            dirty: false,
        }
    }

    pub fn open(filename: &str) -> Self {
        let contents = fs::read_to_string(filename);

        if contents.is_err() {
            return Self {
                rows: Vec::new(),
                filename: Some(filename.to_string()),
                dirty: false,
            };
        }

        let mut rows = Vec::new();
        for value in contents.unwrap().lines() {
            let row = Row::from(value);
            rows.push(row);
        }
        Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        // document could be truly empty, or it could be new document with no
        // data but a single empty row (default Document)
        let true_empty = self.rows.is_empty();
        let almost_empty = self.rows.len() == 1 && self.rows.get(0).unwrap().is_empty();
        true_empty || almost_empty
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn get_filename(&self) -> Option<String> {
        self.filename.clone()
    }

    pub fn line(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn save(&mut self) -> Result<(), std::io::Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;
        if at.y == self.len() {
            self.rows.push(Row::default());
        } else {
            let row = &mut self.rows[at.y];
            let new_row = row.split(at.x);
            self.rows.insert(at.y.saturating_add(1), new_row);
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y.saturating_add(1));
            let row = &mut self.rows[at.y];
            row.append(&next_row);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
        }
    }
}
