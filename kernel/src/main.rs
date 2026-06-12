#![no_std]
#![no_main]

pub mod console_core;
pub mod font;
pub mod graphics;

use common::frame_buffer::*;
use console_core::ConsoleCore;
use console::*;
use core::arch::asm;
use core::panic::PanicInfo;
use graphics::*;

pub fn write_pixel<T: PixelWriter>(writer: &T) {
    let config = writer.get_config();
    for x in 0..config.horizontal_resolution {
        for y in 0..config.vertical_resolution {
            let pixel_color = PixelColor { r: 0, g: 0, b: 100 };
            writer.write(x, y, pixel_color);
        }
    }
}

#[unsafe(no_mangle)]
extern "win64" fn main(frame_buffer_config: &FrameBufferConfig) {
    match frame_buffer_config.pixel_format {
        PixelFormat::PixelRGBResv8BitPerColor => {
            let writer = RGBResv8BitPerColorPixelWriter::new(frame_buffer_config);
            write_pixel(&writer);
            let fg_color = PixelColor {
                r: 255,
                g: 255,
                b: 255,
            };
            let bg_color = PixelColor { r: 0, g: 0, b: 0 };
            let mut console_core = ConsoleCore::new(&writer, fg_color, bg_color);
            for _ in 0..27 {
                console_core.put_string("hello, world!\n");
            }
        }
        PixelFormat::PixelBGRResv8BitPerColor => {
            let writer = BGRResv8BitPerColorPixelWriter::new(frame_buffer_config);
            write_pixel(&writer);
            let fg_color = PixelColor {
                r: 255,
                g: 255,
                b: 255,
            };
            let bg_color = PixelColor { r: 0, g: 0, b: 0 };
            let mut console = ConsoleCore::new(&writer, fg_color, bg_color);
            for _ in 0..27 {
                console.put_string("hello, world!\n");
            }
        }
    }
    halt_loop();
}

fn halt_loop() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    halt_loop();
}
