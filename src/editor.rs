/*
 * editor.rs contains the source code for the editor representation in jot
 */

use crate::{Document, Row, Terminal};
use std::env;
use termion::color;
use termion::event::Key;

const EDITOR_NAME: &str = "Jot";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "by @aneekm";
const SCROLLOFF: usize = 5;
const STATUS_FG_COLOR: color::Rgb = color::Rgb(136, 0, 26);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(230, 233, 236);

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Editor {
    quit: bool,
    mode: Mode,
    terminal: Terminal,
    document: Document,
    cursor: Position,
    scroll_offset: Position,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.refresh_screen() {
                die(e);
            }
            if self.quit {
                break;
            }
            if let Err(e) = self.handle_keypress() {
                die(e);
            }
        }
    }

    pub fn default() -> Self {
        let args: Vec<String> = env::args().collect();
        let document = if let Some(filename) = args.get(1) {
            Document::open(filename)
        } else {
            Document::default()
        };

        Self {
            quit: false,
            mode: Mode::Normal,
            terminal: Terminal::default().expect("Failed to initialize terminal."),
            document,
            cursor: Position::default(),
            scroll_offset: Position::default(),
        }
    }

    fn refresh_screen(&self) -> Result<(), std::io::Error> {
        Terminal::cursor_visible(false);
        Terminal::cursor_position(&Position::default());

        if self.quit {
            Terminal::clear_screen();
            println!("Thanks for using jot!\r");
        } else {
            self.draw_lines();
            self.draw_status_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor.x.saturating_sub(self.scroll_offset.x),
                y: self.cursor.y.saturating_sub(self.scroll_offset.y),
            });
        }

        Terminal::cursor_visible(true);
        Terminal::flush()
    }

    fn handle_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Char('\n') => {
                self.document.insert_newline(&self.cursor);
            }
            Key::Char(c) => match self.mode {
                Mode::Normal => match c {
                    'i' => self.mode = Mode::Insert,
                    ':' => self.mode = Mode::Command,
                    _ => (),
                },
                Mode::Insert => {
                    self.document.insert(&self.cursor, c);
                    self.move_cursor(Key::Right);
                    if c == '\t' {
                        self.move_cursor(Key::Right);
                        self.move_cursor(Key::Right);
                        self.move_cursor(Key::Right);
                    }
                }
                _ => (),
            },
            Key::Ctrl('q') => {
                self.quit = true; // TODO: replace this with real :w :q command mode ops
            }
            Key::Delete => self.document.delete(&self.cursor),
            Key::Backspace => {
                if self.cursor.x > 0 || self.cursor.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        self.scroll();
        Ok(())
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.size().height as usize;
        let Position { mut x, mut y } = self.cursor;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.line(y) {
            row.len()
        } else {
            0
        };

        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = if let Some(row) = self.document.line(y) {
                        row.len()
                    } else {
                        0
                    };
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.line(y) {
            row.len()
        } else {
            0
        };
        if x > width {
            x = width;
        }

        self.cursor = Position { x, y }
    }

    fn scroll(&mut self) {
        // creating descriptive vars for term dimensions & cursor and offset pos
        let Position { x, y } = self.cursor;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let offset = &mut self.scroll_offset;
        let mut window_start_x = offset.x;
        let window_end_x = offset.x.saturating_add(width);
        let mut window_start_y = offset.y;
        let window_end_y = offset.y.saturating_add(height);

        // Scrolloff determines how many lines Jot always keeps before/after
        // cursor in the frame
        if y < window_start_y.saturating_add(SCROLLOFF) {
            window_start_y = y.saturating_sub(SCROLLOFF);
        } else if y >= window_end_y.saturating_sub(SCROLLOFF) {
            window_start_y = y.saturating_sub(height).saturating_add(SCROLLOFF + 1);
        }
        if x < window_start_x {
            window_start_x = x;
        } else if x >= window_end_x {
            window_start_x = x.saturating_sub(width).saturating_add(1);
        }

        self.scroll_offset = Position {
            x: window_start_x,
            y: window_start_y,
        };
    }

    fn draw_line(&self, line: &Row, line_num: usize) {
        let width = self.terminal.size().width as usize;
        let line_num_width = self.document.len().to_string().len();
        let start = self.scroll_offset.x;
        let end = start.saturating_add(width - line_num_width);
        let line = line.render(start, end);
        let mut line_num = line_num.saturating_add(1).to_string();
        while line_num.len() != line_num_width {
            line_num.insert(0, ' ');
        }
        println!("{}{}\r", line_num, line);
    }

    fn draw_lines(&self) {
        let height = self.terminal.size().height as usize;
        if self.document.is_empty() {
            self.draw_homepage();
            return;
        }
        for terminal_line in 0..height {
            Terminal::clear_current_line();
            let line_num = self.scroll_offset.y.saturating_add(terminal_line);
            if let Some(line) = self.document.line(line_num) {
                self.draw_line(line, line_num);
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_status_bar(&self) {
        let mut status: String;
        let width = self.terminal.size().width as usize;

        let mode_indicator = match self.mode {
            Mode::Normal => " N ",
            Mode::Insert => " I ",
            Mode::Command => " C ",
        };
        let modified_indicator = if self.document.is_dirty() { " [!]" } else { "" };
        let mut filename = "".to_string();
        if let Some(name) = &self.document.get_filename() {
            filename = name.clone();
        }
        status = format!("{}{}{}", mode_indicator, filename, modified_indicator);

        let line_num_width = self.document.len().to_string().len();
        let mut line_num = self.cursor.y.to_string();
        while line_num.len() != line_num_width {
            line_num.insert(0, ' ');
        }
        let line_indicator = format!(
            "Text | {:>3},{}/{}",
            self.cursor.y.saturating_add(1),
            self.cursor.x.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();
        status.push_str(&" ".repeat(width.saturating_sub(len)));
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_homepage(&self) {
        let mut title = format!("{}", EDITOR_NAME);
        let mut version = format!("{}", VERSION);
        let mut author = format!("{}", AUTHOR);
        let num_lines = 4; // title + blank line + version + author

        let width = self.terminal.size().width as usize;

        title = format!("~{}\r", center_text(title, width.saturating_sub(1)));
        version = format!("~{}\r", center_text(version, width.saturating_sub(1)));
        author = format!("~{}\r", center_text(author, width.saturating_sub(1)));

        let height = self.terminal.size().height as usize;
        let message_start_line = (height / 2).saturating_sub(num_lines / 2);

        let mut terminal_line = 0;
        while terminal_line < height {
            let line_num = self.scroll_offset.y.saturating_add(terminal_line);
            if let Some(line) = self.document.line(line_num) {
                self.draw_line(line, line_num);
            } else if terminal_line == message_start_line {
                println!("{}\r", title);
                println!("~\r");
                println!("{}\r", version);
                println!("{}\r", author);
                terminal_line += 3; // 3 extra lines vs the other arms of the if
            } else {
                println!("~\r");
            }
            terminal_line += 1;
        }
    }
}

fn center_text(text: String, width_to_pad_to: usize) -> String {
    let len = text.len();
    let padding = width_to_pad_to.saturating_sub(len) / 2;
    let spaces = " ".repeat(padding.saturating_sub(0)); // TODO: confirm this doesn't cause issues.
    return format!("{}{}", spaces, text);
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!(e);
}
