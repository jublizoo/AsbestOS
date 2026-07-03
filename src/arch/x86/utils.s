# fn outb(port: u16, data: u8);
.global _outb
_outb:
	mov %di, %dx
	mov %sil, %al
	out %al, %dx
	ret

# fn inb(port: u16) -> u8;
.global _inb
_inb:
	mov %di, %dx
	inb %dx, %al
	ret

_ltr:
	ltr %ax
	ret

_str:
	str %di
	ret

_lgdt:
	lgdt (%rax)
	ret

_sgdt:
	sgdt (%rdi)
	ret
