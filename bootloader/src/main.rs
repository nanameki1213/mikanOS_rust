#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uefi::*;

mod uefi;

#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main(
    image_handle: Handle,
    system_table: *mut SystemTable,
) -> Status {
    let table = unsafe {
        &*system_table
    };

    let con_out = unsafe {
        &*table.con_out
    };

    let hello: &[u16] = &[
      'H' as u16, 'e' as u16, 'l' as u16, 'l' as u16, 'o' as u16,
      '\r' as u16, '\n' as u16, 0
    ];
    
    unsafe {
        (con_out.output_string)(table.con_out, hello.as_ptr());
    }

    0
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}
