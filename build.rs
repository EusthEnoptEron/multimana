use python3_dll_a::ImportLibraryGenerator;

fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let cross_lib_dir = std::env::var_os("PYO3_CROSS_LIB_DIR")
            .unwrap_or("target/python3-dll".into());

        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();

        let libdir = std::path::Path::new(&cross_lib_dir);
        ImportLibraryGenerator::new(arch.as_str(), env.as_str())
            .version(Some((3, 7)))
            .generate(libdir).expect("python3.dll import library generator failed");
    }
}