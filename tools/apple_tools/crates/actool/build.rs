fn main() {
    let mut build = cc::Build::new();
    build.file("csrc/deflate_parts.c");
    if let Ok(include) = std::env::var("DEP_Z_INCLUDE") {
        build.include(include);
    }
    build.compile("deflate_parts");
    println!("cargo:rerun-if-changed=csrc/deflate_parts.c");
}
