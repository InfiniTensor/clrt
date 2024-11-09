use build_script_cfg::Cfg;

fn main() {
    use search_cl_tools::{find_opencl, OpenclPath};
    use std::{env, path::PathBuf};

    let cfg = Cfg::new("cl");
    let Some(OpenclPath { inc, lib }) = find_opencl() else {
        return;
    };

    cfg.define();
    println!("cargo:rustc-link-search={}", lib.display());
    println!("cargo:rustc-link-lib=OpenCL");

    // The bindgen::Builder is the main entry point to bindgen,
    // and lets you build up options for the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}", inc.display()))
        // Only generate bindings for the functions in these namespaces.
        .allowlist_item("CL.*")
        .allowlist_item("cl.*")
        // Annotate the given type with the #[must_use] attribute.
        // Nothing...
        // Generate rust style enums.
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // Use core instead of std in the generated bindings.
        .use_core()
        // Tell cargo to invalidate the built crate whenever any of the included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
