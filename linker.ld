SECTIONS {
	. = 1M;

	.text : {
		__kernel_start = .;
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

	__kernel_end = .;
}
