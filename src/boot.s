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

	# tag list is terminated by tag of type 0, size 8
	.short 0	# type
	.short 0	# flags
	.long 8 	# size
multiboot_header_end:



bootstrap_reserve_start:



#  Allocate small stack 
.global stack_base
.global stack_end
.section .bss
.align 16
stack_base:
	.skip 16384 # 16KB
stack_end:



.set PRESENT, 	1 << 7	# Not an empty GDT entry
.set NOT_SYS, 	1 << 4	# Code or data (rather than system) segment
.set EXEC, 		1 << 3	# Executable segment
.set DC, 		1 << 2	# Downward/conforming segment
.set RW, 		1 << 1	# Read+writable
.set ACCESSED, 	1 << 0	# Segment has been accessed
.set GRAN_4K, 	1 << 7	# Segment limit has 4KB units
.set SZ_32, 	1 << 6 	# 32-bit default operation size 
.set LONG_MODE, 1 << 5	# 64-bit code segment

.global gdt_base
.global gdt_end
.section .data
.align 8
gdt_base:
.quad 0
gdt_code:
.word 0xffff							# Limit lo
.word 0									# Base lo
.byte 0									# Base mid
.byte PRESENT | NOT_SYS | EXEC | RW 	# Access
.byte GRAN_4K | LONG_MODE | 0xF			# Flags & limit hi
.byte 0									# Base hi
gdt_data:
.word 0xffff							# Limit lo
.word 0									# Base lo
.byte 0									# Base mid
.byte PRESENT | NOT_SYS | RW 			# Access
.byte GRAN_4K | SZ_32 | 0xF				# Flags & limit hi
.byte 0									# Base hi
gdt_end:
# Used for lgdt
gdt_base_limit:
	.word . - gdt_base - 1	# GDT size
	.long gdt_base			# GDT base

.set GDT_CODE_OFFSET, gdt_code - gdt_base
.set GDT_DATA_OFFSET, gdt_data - gdt_base



bootstrap_reserve_end:

.set PML4T_ADDR, 0x1000 	# Page Map Level 4 Table
.set PDPT_ADDR, 0x2000 		# Page directory pointer table
.set PD_ADDR, 0x3000 		# Page directory
.set PT_ADDR, 0x4000 		# First page table
.set PT_SIZE, 4096			# In bytes, for all levels
.set NUM_PTS, 16
.set PT_PRESENT, 1 << 0
.set PT_READABLE, 1 << 1
.set CR0_PE_ENABLE, 1 << 0
.set CR0_PG_ENABLE, 1 << 31
.set CR4_PAE_ENABLE, 1 << 5
.set SIZEOF_PT_ENTRY, 8
.set ENTRIES_PER_PT, 512
.set ENTRIES, ENTRIES_PER_PT * NUM_PTS
.set PAGE_SIZE, 4096
.set EFER_MSR, 0xC0000080
.set EFER_LM_ENABLE, 1 << 8

.section .text
.global _start
.extern rust_main
_start:
	# 32-bit protected mode, interrupts disabled, paging disabled

	cli
	mov $stack_end, %esp
	push %eax 	# saved_magic
	push %ebx 	# saved_mbi

	# Clear all PTS (stosd writes 4 bytes, so this clears all 4 tables)

	mov $PML4T_ADDR, %edi
	mov %edi, %cr3

	xor %eax, %eax
	mov $PT_SIZE, %ecx
	# `rep` termination is %rcx=0, decrements %rcx
	# `stos` writes %eax to addr stored in %edi, decrements %edi
	rep stosl

	# Link PML4T with PDPT
	movl $PML4T_ADDR, %edi
	movl $(PDPT_ADDR | PT_PRESENT | PT_READABLE), (%edi)

	# Link PDPT with PD
	movl $PDPT_ADDR, %edi
	movl $(PD_ADDR | PT_PRESENT | PT_READABLE), (%edi)

	# Link PD with all PTs
	movl $(PT_ADDR | PT_PRESENT | PT_READABLE), %ebx 	# PD entry data
	movl $PD_ADDR, %edi 								# PD entry ptr
	movl $NUM_PTS, %ecx
_link_pd_loop:
	movl %ebx, (%edi)
	add $PAGE_SIZE, %ebx
	add $SIZEOF_PT_ENTRY, %edi
	loop _link_pd_loop

	# FILL PTS

	movl $PT_ADDR, %edi 					# Current entry
	movl $(PT_PRESENT | PT_READABLE), %ebx 	# Phys addr (starting at 0)
	movl $ENTRIES, %ecx 					# Counter
_fill_pt_loop:
	movl %ebx, (%edi)
	add $PAGE_SIZE, %ebx
	add $SIZEOF_PT_ENTRY, %edi
	loop _fill_pt_loop

	# Set PAE
	mov %cr4, %eax
	or $CR4_PAE_ENABLE, %eax
	mov %eax, %cr4

	# Set LME
	movl $EFER_MSR, %ecx
	rdmsr
	# MSR data stored in edx, eax
	or $EFER_LM_ENABLE, %eax
	wrmsr

	# Set PG, PE (enable paging)
	movl %cr0, %eax
	or $(CR0_PG_ENABLE | CR0_PE_ENABLE), %eax
	movl %eax, %cr0

	lgdt gdt_base_limit
	mov $GDT_DATA_OFFSET, %ax
	mov %ax, %ds
	mov %ax, %es
	mov %ax, %ss

	pop %esi 	# mbi pointer
	pop %edi 	# magic

	ljmp $GDT_CODE_OFFSET, $_enter_main

.code64
_enter_main:
	call rust_main

	# Disable interrupts, enter hlt loop
	cli
hang:
	hlt
	jmp hang
