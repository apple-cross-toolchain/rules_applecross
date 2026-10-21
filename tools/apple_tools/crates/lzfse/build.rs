fn main() {
    let files = [
        "lzfse_decode.c",
        "lzfse_decode_base.c",
        "lzfse_encode.c",
        "lzfse_encode_base.c",
        "lzfse_encode_stream.c",
        "lzfse_fse.c",
        "lzvn_decode_base.c",
        "lzvn_encode_base.c",
    ];
    let mut build = cc::Build::new();
    for file in files {
        build.file(format!("vendor/{file}"));
        println!("cargo:rerun-if-changed=vendor/{file}");
    }
    build.include("vendor").opt_level(2).warnings(false).compile("lzfse");
}
