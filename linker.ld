SECTIONS {
	. = 1M;

	.multiboot_header : {
		/* Keep section even if there are no references */
		KEEP(*(.multiboot_header))
	}

	.text : {
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
