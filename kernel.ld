OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(main)
SECTIONS
{
  . = 0x100000;
  . = 0x100000 + SIZEOF_HEADERS;
  . = ALIGN(0x1000);

  .text : {
    *(.text)
    *(.text.*)
  }

  .rodata : ALIGN(0x1000) {
    *(.strings)
    *(.rodata)
    *(.rodata.*)
  }

  .data : ALIGN(0x1000) {
    *(.data)
    *(.data.*)
  }

  .bss : ALIGN(0x1000) {
    __bss_start = .;
    *(.bss)
    *(.bss.*)
    *(COMMON)
    __bss_end = .;
  }

  . = ALIGN(4096);
  .intr_stack (NOLOAD) : {
    _intr_stack_start = .;
    . = . + 0x4000;
    _intr_stack_end = .;
  }
  _free_area = .;
}
