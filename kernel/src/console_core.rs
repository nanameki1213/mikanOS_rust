use crate::{
    font::{write_ascii, write_string},
    graphics::{PixelColor, PixelWriter},
};
use core::char;

pub const ROWS: usize = 25;
pub const COLUMNS: usize = 80;

pub struct ConsoleCore<'a, T: PixelWriter> {
    writer: &'a T,
    fg_color: PixelColor,
    bg_color: PixelColor,
    buffer: [[char; COLUMNS]; ROWS],
    cursor_row: u32,
    cursor_column: u32,
}

impl<'a, T: PixelWriter> ConsoleCore<'a, T> {
    pub const fn new(writer: &'a T, fg_color: PixelColor, bg_color: PixelColor) -> Self {
        Self {
            writer,
            fg_color,
            bg_color,
            buffer: [[0u8 as char; COLUMNS]; ROWS],
            cursor_row: 0,
            cursor_column: 0,
        }
    }
    pub fn put_char(&mut self, c: char) {
        if c == '\n' {
            self.new_line();
        } else if self.cursor_column < COLUMNS as u32 - 1 {
            write_ascii(
                self.writer,
                8 * self.cursor_column,
                16 * self.cursor_row,
                c,
                self.fg_color,
            );
            self.buffer[self.cursor_row as usize][self.cursor_column as usize] = c;
            self.cursor_column += 1;
        }
    }
    pub fn put_string(&mut self, s: &'static str) {
        for c in s.chars() {
            self.put_char(c);
        }
    }
    fn new_line(&mut self) {
        self.cursor_column = 0;
        if self.cursor_row < ROWS as u32 - 1 {
            self.cursor_row += 1;
        } else {
            for y in 0..(16 * ROWS) {
                for x in 0..(8 * COLUMNS) {
                    self.writer.write(x as u32, y as u32, self.bg_color);
                }
            }
            for row in 0..(ROWS - 1) {
                let src = (self.buffer.as_ptr() as usize + (COLUMNS * (row + 1))) as *mut u8;
                let dst = (self.buffer.as_ptr() as usize + (COLUMNS * row)) as *mut u8;
                unsafe {
                    core::ptr::copy(src, dst, COLUMNS);
                }
                let s = self.buffer[row];
                let mut i = 0;
                for c in s {
                    write_ascii(self.writer, 8 * i, 16 * row as u32, c, self.fg_color);
                    i += 1;
                }
            }
            let dst = (self.buffer.as_ptr() as usize + (COLUMNS * (ROWS - 1))) as *mut u8;
            unsafe {
                core::ptr::write_bytes(dst, 0, COLUMNS + 1);
            }
        }
    }
}
