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

fn allocate_poll(size: usize, buffer: *mut *mut c_void) -> EfiStatus {
    let bs = boot_services();

    unsafe {
        (bs.allocate_pool)(
            EfiMemoryType::LoaderData,
            size,
            buffer,
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
pub extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus {
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
    let status = get_memory_map(&mut memmap);
    // statusはBufferTooSmallのはず
    assert_eq!(status, EfiStatus::BufferTooSmall);

    memmap.map_size += memmap.descriptor_size * 4;
    let mut memmap_buf_ptr: *mut c_void = core::ptr::null_mut();
    let status = allocate_poll(memmap.map_size, &raw mut memmap_buf_ptr);
    if EfiStatus::is_error(status) {
        print!("failed to allocate memory: {:?}\r\n", status);
        halt_loop();
    }
    let memmap_buf = unsafe {
        &mut *core::ptr::slice_from_raw_parts_mut(
            memmap_buf_ptr as *mut u8,
            memmap.map_size / memmap.descriptor_size,
        )
    };

    memmap.buffer = memmap_buf;
    let status = get_memory_map(&mut memmap);
    if EfiStatus::is_error(status) {
        print!("failed to get memory map: {:?}\r\n", status);
        halt_loop();
    }

    let mut root_dir: *mut EfiFileProtocol = core::ptr::null_mut();
    let status = open_root_dir(image_handle, &raw mut root_dir);
    if EfiStatus::is_error(status) {
        print!("failed to open root directory: {:?}", status);
        halt_loop();
    }

    let mut kernel_file: *mut EfiFileProtocol = core::ptr::null_mut();

    // "kernel.elf" を UTF-16 に変換（NUL 終端込みで 7 要素）
    let mut kernel_name = [0u16; 11];
    for (i, c) in "kernel.elf".encode_utf16().enumerate() {
        kernel_name[i] = c;
    }
    let status = unsafe {
        ((*root_dir).open)(
            root_dir,
            &raw mut kernel_file,
            kernel_name.as_ptr(),
            open_mode::READ,
            0,
        )
    };
    if EfiStatus::is_error(status) {
        print!("failed to open kernel: {:?}\r\n", status);
        halt_loop();
    }

    let mut file_info_size = core::mem::size_of::<EfiFileInfo>() + core::mem::size_of::<u16>() * 11;
    let mut file_info_buffer_ptr: *mut c_void = core::ptr::null_mut();
    let status = allocate_poll(file_info_size, &raw mut file_info_buffer_ptr);
    if EfiStatus::is_error(status) {
        print!("failed to allocate memory: {:?}\r\n", status);
        halt_loop();
    }
    unsafe {
        ((*kernel_file).get_info)(
            kernel_file,
            &raw const EFI_FILE_INFO_ID,
            &raw mut file_info_size,
            file_info_buffer_ptr,
        );
    }

    let file_info = file_info_buffer_ptr as *mut EfiFileInfo;
    let kernel_file_size = unsafe {
        (*file_info).file_size
    };



    EfiStatus::Success
}

fn open_root_dir(image_handle: EfiHandle, root: *mut *mut EfiFileProtocol) -> EfiStatus {
    // ブートローダがロードされたストレージデバイスを調べる
    let mut loaded_image: *mut EfiLoadedImageProtocol = core::ptr::null_mut();
    let bs = boot_services();
    let status = unsafe {
        (bs.open_protocol)(
            image_handle,
            &raw const EFI_LOADED_IMAGE_PROTOCOL_GUID as *mut EfiGuid,
            &mut loaded_image as *mut _ as *mut *mut c_void,
            image_handle,
            core::ptr::null_mut(),
            open_protocol::BY_HANDLE_PROTOCOL,
        )
    };
    if EfiStatus::is_error(status) {
        return status;
    }

    let mut fs: *mut EfiSimpleFileSystemProtocol = core::ptr::null_mut();
    let status = unsafe {
        (bs.open_protocol)(
            (*loaded_image).device_handle,
            &raw const EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID as *mut EfiGuid,
            &mut fs as *mut _ as *mut *mut c_void,
            image_handle,
            core::ptr::null_mut(),
            open_protocol::BY_HANDLE_PROTOCOL,
        )
    };
    if EfiStatus::is_error(status) {
        return status;
    }

    unsafe {
        return ((*fs).open_volume)(fs, root);
    }
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
