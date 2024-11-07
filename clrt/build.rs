fn main() {
    use build_script_cfg::Cfg;
    use search_cl_tools::opencl_exists;

    println!("cargo:rerun-if-changed=build.rs");

    if opencl_exists() {
        Cfg::new("detected_opencl").define()
    }
}
