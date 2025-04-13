fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir("src/generated") // Set out_dir to a shared location
        .compile(&["proto/corebridge.proto"], &["proto"])
        .unwrap();
}
