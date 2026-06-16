fn main() {
    println!("cargo:rerun-if-changed=src/boot.s");
    println!("cargo:rerun-if-changed=src/linker.ld");

    // Grub enters in 32-bit mode, so we compile with m32
    cc::Build::new()
        .file("src/boot.s")
        .compile("boot");
}
