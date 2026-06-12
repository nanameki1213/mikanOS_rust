use crate::graphics::{PixelColor, PixelWriter};
use core::char;

pub fn get_font(c: char) -> *const u8 {
    unsafe extern "C" {
        static _binary_hankaku_bin_start: u8;
        static _binary_hankaku_bin_end: u8;
        static _binary_hankaku_bin_size: u8;
    }

    let index = 16 * (c as usize);
    unsafe {
        if index >= &_binary_hankaku_bin_size as *const _ as usize {
            return core::ptr::null();
        }
        return (&_binary_hankaku_bin_start as *const _ as usize + index) as *const u8;
    }
}

pub fn write_ascii<T: PixelWriter>(writer: &T, x: u32, y: u32, c: char, color: PixelColor) {
    let font_ptr = get_font(c);
    if font_ptr == core::ptr::null() {
        return;
    }
    let font = unsafe { &*core::ptr::slice_from_raw_parts(font_ptr, 16) };
    for dy in 0..16 {
        for dx in 0..8 {
            if (font[dy] << dx) & 0x80 != 0 {
                writer.write(x + dx, y + dy as u32, color);
            }
        }
    }
}

pub fn write_string<T: PixelWriter>(
    writer: &T,
    x: u32,
    y: u32,
    s: &'static str,
    color: PixelColor,
) {
    let mut i = 0;
    for c in s.chars() {
        write_ascii(writer, x + 8 * i, y, c, color);
        i += 1;
    }
}
