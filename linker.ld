SECTIONS {
	. = 1M;

	.text : {
		KEEP(*(.multiboot_header))
		*(.text .text.*)
	}

	.rodata : {
		*(.rodata .rodata.*)
	}

	.data : {
		*(.data .data.*)
	}

	.bss : {
		*(COMMON)
		*(.bss .bss.*)
	}
}
