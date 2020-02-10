fn main() {
    prost_build::Config::new()
        .out_dir("src/proto/")
        .compile_protos(&["proto/service.proto"], &["proto/"])
        .unwrap();
    println!("cargo:rerun-if-changed=proto/service.proto")
}
