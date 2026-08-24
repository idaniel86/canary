fn main() {
    let mut generator = micropb_gen::Generator::new();
    // Compile example.proto into a Rust module
    generator
        .use_container_heapless()
        .configure(
            ".readings.SensorReading.gas_resistance",
            micropb_gen::Config::new().max_len(10),
        )
        .configure(
            ".readings.SensorReadings.readings",
            micropb_gen::Config::new().max_len(5),
        )
        .add_protoc_arg("--proto_path=../proto");

    generator
        .compile_protos(
            &["readings.proto"],
            std::env::var("OUT_DIR").unwrap() + "/readings.rs",
        )
        .unwrap();
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
