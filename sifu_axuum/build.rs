fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(
            std::path::PathBuf::from(
                std::env::var("OUT_DIR").unwrap()
            ).join("clipboard_descriptor.bin")
        )
        .compile_protos(&["proto/clipboard.proto"], &["proto"])?;

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(
            std::path::PathBuf::from(
                std::env::var("OUT_DIR").unwrap()
            ).join("hello_descriptor.bin")
        )
        .compile_protos(&["proto/hello.proto"], &["proto"])?;

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(
            std::path::PathBuf::from(
                std::env::var("OUT_DIR").unwrap()
            ).join("clipboard_sync_descriptor.bin")
        )
        .compile_protos(&["proto/clipboard_sync.proto"], &["proto"])?;
    Ok(())
}
