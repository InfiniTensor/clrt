#![deny(warnings)]

/// See <https://github.com/kenba/opencl-sys-rs/blob/main/build.rs>.
pub fn opencl_exists() -> bool {
    [
        "OPENCL_PATH",
        "OPENCL_ROOT",
        "INTELOCLSDKROOT",
        "AMDAPPSDKROOT",
        "CUDA_PATH",
    ]
    .iter()
    .any(|var| std::env::var_os(var).is_some())
}
