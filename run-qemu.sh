cp target/x86_64-target/debug/kernel iso/boot/kernel.elf
grub-mkrescue -o kernel.iso iso > /dev/null
grub-file --is-x86-multiboot2 iso/boot/kernel.elf > /dev/null
if [ "$?" -ne 0 ]; then
	echo "ELF is invalid for multiboot2"
	exit 1
fi

extra=()
if [ "$GDB" = "1" ]; then
	extra=(-S -s)
fi

qemu-system-x86_64 -cdrom kernel.iso -display gtk,zoom-to-fit=on "${extra[@]}"
