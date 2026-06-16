cargo build
cp target/x86_64-target/debug/kernel iso/boot/kernel.elf
grub-mkrescue -o kernel.iso iso
grub-file --is-x86-multiboot2 iso/boot/kernel.elf
echo "0 on success:"
echo $?
