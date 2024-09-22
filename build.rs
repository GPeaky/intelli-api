fn main() {
    let mut config = prost_build::Config::new();

    config
        .compile_protos(&["protos/f1.proto"], &["protos/"])
        .unwrap();
}
