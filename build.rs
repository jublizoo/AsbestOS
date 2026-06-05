fn main() {
    // Grub enters in 32-bit mode, so we compile with m32
    cc::Build::new()
        .file("src/boot.s")
        .compile("boot");
}
