fn main() {
    cc::Build::new()
        .file("src/libretro_log_shim.c")
        .warnings(false)
        .compile("phicade_libretro_log_shim");
}
