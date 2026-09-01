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
            &["readings.proto", "scores.proto"],
            std::env::var("OUT_DIR").unwrap() + "/proto.rs",
        )
        .unwrap();

    let bindings = bindgen::Builder::default()
        .headers([
            "../vendor/bsec2/src/inc/bsec_datatypes.h",
            "../vendor/bsec2/src/inc/bsec_interface.h",
        ])
        .use_core()
        .generate()
        .unwrap();

    bindings
        .write_to_file(std::env::var("OUT_DIR").unwrap() + "/bindings.rs")
        .unwrap();

    println!("cargo:rustc-link-search=native=vendor/bsec2/src/cortex-m33/fpv5-sp-d16-hard");
    println!("cargo:rustc-link-lib=static=algobsec");
    println!("cargo:rerun-if-changed=proto");
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
