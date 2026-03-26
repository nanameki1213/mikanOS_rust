#![no_std]
#![no_main]

use console::print;
use core::arch::asm;
use core::ffi::c_void;
use core::panic::PanicInfo;
use spin::Once;
use uefi::*;

mod uefi;

pub static BOOT_SERVICE: Once<&'static EfiBootServices> = Once::new();
pub static CON_OUT: Once<&'static EfiSimpleTextOutputProtocol> = Once::new();

struct MemoryMap<'a> {
    buffer: &'a mut [u8],
    map_size: usize,
    map_key: usize,
    descriptor_size: usize,
    descriptor_version: u32,
}

fn boot_services() -> &'static EfiBootServices {
    *BOOT_SERVICE.get().expect("Boot Services not initialized")
}

fn con_out() -> &'static EfiSimpleTextOutputProtocol {
    *CON_OUT.get().expect("ConOut not initialized")
}

fn get_memory_map(map: &mut MemoryMap) -> EfiStatus {
    if map.buffer.len() == 0 {
        return EfiStatus::BufferTooSmall;
    }

    let bs = boot_services();

    unsafe {
        (bs.get_memory_map)(
            &mut map.map_size,
            map.buffer.as_ptr() as *mut EfiMemoryDescriptor,
            &mut map.map_key,
            &mut map.descriptor_size,
            &mut map.descriptor_version,
        )
    }
}

/// UEFI コンソール向け write_byte。
/// `\n` を `\r\n` に変換し、1 バイトを UTF-16 に変換して output_string で出力する。
fn uefi_putc(b: u8) {
    let out = con_out();
    if b == b'\n' {
        let buf = [b'\r' as u16, b'\n' as u16, 0u16];
        unsafe { (out.output_string)(out as *const _ as *mut _, buf.as_ptr()) };
    } else {
        let buf = [b as u16, 0u16];
        unsafe { (out.output_string)(out as *const _ as *mut _, buf.as_ptr()) };
    }
}

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(image_handle: EfiHandle, system_table: *mut EfiSystemTable) -> EfiStatus {
    let table = unsafe { &*system_table };

    let boot_service = unsafe { &*table.boot_services };
    BOOT_SERVICE.call_once(|| boot_service);
    let bs = boot_services();

    let con_out_proto = unsafe { &*table.con_out };
    CON_OUT.call_once(|| con_out_proto);
    *console::CONSOLE.lock() = Some(console::Console::new(uefi_putc));

    print!("Hello, World!\r\n");

    // 二段階呼び出し
    let mut temp_buf = [0u8; 1];
    let mut memmap = MemoryMap {
        buffer: &mut temp_buf,
        map_size: 0,
        map_key: 0,
        descriptor_size: 0,
        descriptor_version: 0,
    };
    let mut status = get_memory_map(&mut memmap);
    // statusはBufferTooSmallのはず
    assert_eq!(status, EfiStatus::BufferTooSmall);

    memmap.map_size += memmap.descriptor_size * 4;
    let mut memmap_buf_ptr: *mut c_void = core::ptr::null_mut();
    unsafe {
        status = (bs.allocate_pool)(
            EfiMemoryType::LoaderData,
            memmap.map_size,
            &mut memmap_buf_ptr as *mut *mut c_void,
        );
    }
    if EfiStatus::is_error(status) {
        print!("failed to allocate memory: {:?}\r\n", status);
        halt_loop();
    }
    let mut memmap_buf = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(
            memmap_buf_ptr as *mut u8,
            memmap.map_size / memmap.descriptor_size,
        )
    };

    memmap.buffer = &mut memmap_buf;
    let status = get_memory_map(&mut memmap);
    if EfiStatus::is_error(status) {
        print!("failed to get memory map: {:?}\r\n", status);
        halt_loop();
    }

    EfiStatus::Success
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
