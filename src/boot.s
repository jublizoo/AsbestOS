.code32

.set MULTIBOOT2_MAGIC, 0xE85250D6
.set ARCHITECTURE, 0 	# 32-bit protected
.set HEADER_LENGTH, multiboot_header_end - multiboot_header
.set CHECKSUM, -(MULTIBOOT2_MAGIC + ARCHITECTURE + HEADER_LENGTH)

.section .multiboot_header
.align 8
multiboot_header:
	.long MULTIBOOT2_MAGIC
	.long ARCHITECTURE
	.long HEADER_LENGTH
	.long CHECKSUM
multiboot_header_end:


# No tags, tag list is terminated by tag of type 0, size 8
; .short 0	# type
; .short 0	# flags
; .long 8 	# size

#  Allocate small stack 
.section .bss
.align 16
.stack_bottom:
	.skip 16284 # 16KB
stack_top:

.section .text
.global _start
.extern rust_main
_start:
	# 32-bit protected mode, interrupts disabled, paging disabled

	mov $stack_top, %esp
	push %ebx 	# mbi pointer
	push %eax 	# magic
	call rust_main
	# Disable interrupts, enter hlt loop
	cli
hang:
	hlt
	jmp hang

.code64
