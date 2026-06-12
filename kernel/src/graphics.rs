use common::frame_buffer::*;

pub trait PixelWriter {
    fn write(&self, x: u32, y: u32, c: PixelColor);
    fn get_config(&self) -> &FrameBufferConfig;
    fn pixel_at(&self, x: u32, y: u32) -> *mut u8 {
        let config = self.get_config();
        (config.frame_buffer as u32 + 4 * (config.pixels_per_scan_line * y + x)) as *mut u8
    }
}

pub struct RGBResv8BitPerColorPixelWriter<'a> {
    config: &'a FrameBufferConfig,
}

impl<'a> RGBResv8BitPerColorPixelWriter<'a> {
    pub const fn new(config: &'a FrameBufferConfig) -> Self {
        Self { config }
    }
}

impl<'a> PixelWriter for RGBResv8BitPerColorPixelWriter<'a> {
    fn write(&self, x: u32, y: u32, c: PixelColor) {
        let p = self.pixel_at(x, y);
        let pixel_buffer = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(p, 4) };
        pixel_buffer[0] = c.r;
        pixel_buffer[1] = c.g;
        pixel_buffer[2] = c.b;
    }
    fn get_config(&self) -> &FrameBufferConfig {
        self.config
    }
}

pub struct BGRResv8BitPerColorPixelWriter<'a> {
    config: &'a FrameBufferConfig,
}

impl<'a> BGRResv8BitPerColorPixelWriter<'a> {
    pub const fn new(config: &'a FrameBufferConfig) -> Self {
        Self { config }
    }
}

impl<'a> PixelWriter for BGRResv8BitPerColorPixelWriter<'a> {
    fn write(&self, x: u32, y: u32, c: PixelColor) {
        let p = self.pixel_at(x, y);
        let pixel_buffer = unsafe { &mut *core::ptr::slice_from_raw_parts_mut(p, 4) };
        pixel_buffer[0] = c.b;
        pixel_buffer[1] = c.g;
        pixel_buffer[2] = c.r;
    }
    fn get_config(&self) -> &FrameBufferConfig {
        self.config
    }
}

#[derive(Clone, Copy)]
pub struct PixelColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
