#![no_std]
#![no_main]

mod elf;
mod uefi;

use console::print;
use core::arch::asm;
use core::ffi::c_void;
use core::panic::PanicInfo;
use elf::*;
use spin::Once;
use uefi::*;

pub static BOOT_SERVICE: Once<&'static EfiBootServices> = Once::new();
pub static CON_OUT: Once<&'static EfiSimpleTextOutputProtocol> = Once::new();

pub struct MemoryMap {
    pub buffer: *mut u8,
    pub map_size: usize,
    pub map_key: usize,
    pub descriptor_size: usize,
    pub descriptor_version: u32,
}

#[derive(Debug)]
pub struct UefiError(pub EfiStatus);

pub struct MemoryMapSizeInfo {
    pub required_size: usize,
    pub descriptor_size: usize,
}

pub fn efi_status_to_result(status: EfiStatus) -> Result<(), UefiError> {
    if EfiStatus::is_error(status) {
        Err(UefiError(status))
    } else if EfiStatus::is_warning(status) {
        Err(UefiError(status))
    } else {
        Ok(())
    }
}

pub struct BootServices {
    bs: &'static EfiBootServices,
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl BootServices {
    pub const fn new(bs: &'static EfiBootServices) -> Self {
        Self { bs }
    }

    pub fn allocate_any_pages(
        &self,
        memory_type: EfiMemoryType,
        pages: usize,
    ) -> Result<EfiPhysicalAddress, UefiError> {
        let mut addr: EfiPhysicalAddress = 0;
        let status = unsafe {
            (self.bs.allocate_pages)(
                EfiAllocateType::AllocateAnyPages,
                memory_type,
                pages,
                &mut addr,
            )
        };
        efi_status_to_result(status)?;
        Ok(addr)
    }

    pub fn allocate_pages_below(
        &self,
        memory_type: EfiMemoryType,
        pages: usize,
        max_addr: EfiPhysicalAddress,
    ) -> Result<EfiPhysicalAddress, UefiError> {
        let mut addr: EfiPhysicalAddress = max_addr;
        let status = unsafe {
            (self.bs.allocate_pages)(
                EfiAllocateType::AllocateMaxAddress,
                memory_type,
                pages,
                &mut addr,
            )
        };
        efi_status_to_result(status)?;
        Ok(addr)
    }

    pub fn allocate_pages_at(
        &self,
        memory_type: EfiMemoryType,
        pages: usize,
        addr: EfiPhysicalAddress,
    ) -> Result<(), UefiError> {
        let mut address = addr;
        let status = unsafe {
            (self.bs.allocate_pages)(
                EfiAllocateType::AllocateAddress,
                memory_type,
                pages,
                &mut address,
            )
        };
        efi_status_to_result(status)?;
        Ok(())
    }

    pub fn get_memory_map<'a>(&self) -> Result<MemoryMap, UefiError> {
        let mut map_size = 0;
        let mut map_key = 0;
        let mut descriptor_size = 0;
        let mut descriptor_version = 0;
        unsafe {
            (self.bs.get_memory_map)(
                &mut map_size,
                core::ptr::null_mut(),
                &mut map_key,
                &mut descriptor_size,
                &mut descriptor_version,
            )
        };

        map_size += descriptor_size * 2;
        let memmap_buf_ptr = self.allocate_pool(EfiMemoryType::LoaderData, map_size)?;

        let status = unsafe {
            (self.bs.get_memory_map)(
                &mut map_size,
                memmap_buf_ptr as *mut EfiMemoryDescriptor,
                &mut map_key,
                &mut descriptor_size,
                &mut descriptor_version,
            )
        };
        efi_status_to_result(status)?;

        Ok(MemoryMap {
            buffer: memmap_buf_ptr,
            map_size,
            map_key,
            descriptor_size,
            descriptor_version,
        })
    }

    pub fn get_memory_map_with_buf<'a>(&self, buffer: &'a [u8]) -> Result<MemoryMap, UefiError> {
        let mut map_size = buffer.len();
        let mut map_key = 0;
        let mut descriptor_size = 0;
        let mut descriptor_version = 0;

        let status = unsafe {
            (self.bs.get_memory_map)(
                &mut map_size,
                buffer.as_ptr() as *mut EfiMemoryDescriptor,
                &mut map_key,
                &mut descriptor_size,
                &mut descriptor_version,
            )
        };
        efi_status_to_result(status)?;
        Ok(MemoryMap {
            map_size,
            buffer: buffer.as_ptr() as *mut u8,
            map_key,
            descriptor_size,
            descriptor_version,
        })
    }

    pub fn allocate_pool(
        &self,
        memory_type: EfiMemoryType,
        size: usize,
    ) -> Result<*mut u8, UefiError> {
        let mut buf: *mut c_void = core::ptr::null_mut();
        let status = unsafe { (self.bs.allocate_pool)(memory_type, size, &mut buf) };
        efi_status_to_result(status)?;
        Ok(buf as *mut u8)
    }

    pub fn open_protocol(
        &self,
        image_handle: EfiHandle,
        protocol: *const EfiGuid,
        agent_handle: EfiHandle,
        controller_handle: EfiHandle,
        attributes: u32,
    ) -> Result<*mut c_void, UefiError> {
        let mut interface: *mut c_void = core::ptr::null_mut();
        let status = unsafe {
            (self.bs.open_protocol)(
                image_handle,
                protocol,
                &raw mut interface,
                agent_handle,
                controller_handle,
                attributes,
            )
        };
        efi_status_to_result(status)?;
        Ok(interface)
    }

    pub fn exit_boot_services(
        &self,
        image_handle: EfiHandle,
        map_key: usize,
    ) -> Result<(), UefiError> {
        let status = unsafe { (self.bs.exit_boot_services)(image_handle, map_key) };
        efi_status_to_result(status)?;
        Ok(())
    }
}

fn boot_services() -> &'static EfiBootServices {
    *BOOT_SERVICE.get().expect("Boot Services not initialized")
}

fn con_out() -> &'static EfiSimpleTextOutputProtocol {
    *CON_OUT.get().expect("ConOut not initialized")
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
pub unsafe extern "efiapi" fn efi_main(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> EfiStatus {
    let table = unsafe { &*system_table };

    let boot_service = unsafe { &*table.boot_services };
    BOOT_SERVICE.call_once(|| boot_service);
    let bs = BootServices::new(boot_services());

    let con_out_proto = unsafe { &*table.con_out };
    CON_OUT.call_once(|| con_out_proto);
    *console::CONSOLE.lock() = Some(console::Console::new(uefi_putc));

    print!("Hello, World!\r\n");

    let mut memmap = match bs.get_memory_map() {
        Ok(map) => map,
        Err(err) => panic!("failed to get memory map: {:?}", err),
    };

    let root_dir = match open_root_dir(&bs, image_handle) {
        Ok(root) => root,
        Err(err) => panic!("failed to open root directory: {:?}", err),
    };

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
        panic!("failed to open kernel: {:?}", status);
    }

    let mut file_info_size = core::mem::size_of::<EfiFileInfo>() + core::mem::size_of::<u16>() * 11;
    let file_info_buffer_ptr = match bs.allocate_pool(EfiMemoryType::LoaderData, file_info_size) {
        Ok(ptr) => ptr as *mut c_void,
        Err(err) => panic!("failed to allocate memory: {:?}", err),
    };
    unsafe {
        ((*kernel_file).get_info)(
            kernel_file,
            &raw const EFI_FILE_INFO_ID,
            &raw mut file_info_size,
            file_info_buffer_ptr,
        );
    }

    let file_info = file_info_buffer_ptr as *mut EfiFileInfo;
    let mut kernel_file_size = unsafe { (*file_info).file_size };

    let kernel_base_addr: EfiPhysicalAddress = 0x100000;
    match bs.allocate_pages_at(
        EfiMemoryType::LoaderData,
        (kernel_file_size as usize + 0xfff) / 0x1000,
        kernel_base_addr,
    ) {
        Ok(_) => {}
        Err(err) => panic!("failed to allocate memory for kernel: {:?}", err),
    }
    let status = unsafe {
        ((*kernel_file).read)(
            kernel_file,
            &raw mut kernel_file_size as *mut usize,
            kernel_base_addr as *mut c_void,
        )
    };
    if EfiStatus::is_error(status) {
        panic!("failed to read kernel file: {:?}", status);
    }
    print!(
        "Kernel: {:#x} ({} bytes)\r\n",
        kernel_base_addr, kernel_file_size
    );

    let gop = match open_gop(&bs, image_handle) {
        Ok(gop) => gop,
        Err(err) => panic!("failed to get gop: {:?}", err),
    };
    unsafe {
        let mode = (*gop).mode;
        let frame_buffer_base = (*mode).frame_buffer_base as *mut u8;
        let frame_buffer_size = (*mode).frame_buffer_size;

        let frame_buffer =
            &mut *core::ptr::slice_from_raw_parts_mut(frame_buffer_base, frame_buffer_size);
        frame_buffer.fill(255);
    }

    let buffer = [0u8; 4096 * 4];
    memmap = match bs.get_memory_map_with_buf(&buffer) {
        Ok(map) => map,
        Err(err) => panic!("failed to get memory map: {:?}", err),
    };
    match bs.exit_boot_services(image_handle, memmap.map_key) {
        Ok(_) => {}
        Err(_) => halt_loop(),
    };

    let entry_address = load_elf(kernel_base_addr as *const u64);
    let entry_point: unsafe extern "C" fn() =
        unsafe { core::mem::transmute(entry_address as usize) };
    unsafe {
        entry_point();
    }

    EfiStatus::Success
}

fn open_root_dir(
    bs: &BootServices,
    image_handle: EfiHandle,
) -> Result<*mut EfiFileProtocol, UefiError> {
    // ブートローダがロードされたストレージデバイスを調べる
    let loaded_image = bs.open_protocol(
        image_handle,
        &raw const EFI_LOADED_IMAGE_PROTOCOL_GUID,
        image_handle,
        core::ptr::null_mut(),
        open_protocol::BY_HANDLE_PROTOCOL,
    )? as *mut EfiLoadedImageProtocol;
    let fs = unsafe {
        bs.open_protocol(
            (*loaded_image).device_handle,
            &raw const EFI_SIMPLE_FILE_SYSTEM_PROTOCOL_GUID,
            image_handle,
            core::ptr::null_mut(),
            open_protocol::BY_HANDLE_PROTOCOL,
        )? as *mut EfiSimpleFileSystemProtocol
    };

    let mut root: *mut EfiFileProtocol = core::ptr::null_mut();
    let status = unsafe { ((*fs).open_volume)(fs, &raw mut root) };
    efi_status_to_result(status)?;
    Ok(root)
}

fn halt_loop() -> ! {
    loop {
        unsafe { asm!("hlt") };
    }
}

fn open_gop(
    bs: &BootServices,
    image_handle: EfiHandle,
) -> Result<*mut EfiGraphicsOutputProtocol, UefiError> {
    let gop = bs.open_protocol(
        image_handle,
        &raw const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        image_handle,
        core::ptr::null_mut(),
        open_protocol::BY_HANDLE_PROTOCOL,
    )? as *mut EfiGraphicsOutputProtocol;
    Ok(gop)
}

#[panic_handler]
fn panic(info: &PanicInfo<'_>) -> ! {
    print!("{}\r\n", info);
    halt_loop();
}
