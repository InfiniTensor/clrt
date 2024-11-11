#![deny(warnings)]

use std::path::PathBuf;

pub struct OpenclPath {
    pub inc: PathBuf,
    pub lib: PathBuf,
}

pub fn find_opencl() -> Option<OpenclPath> {
    fn env_path(key: &str) -> Option<PathBuf> {
        println!("cargo:rerun-if-env-changed={key}");
        std::env::var(key).ok().map(PathBuf::from)
    }

    Some(OpenclPath {
        inc: env_path("OPENCL_HEADERS")?,
        lib: env_path("OPENCL_LIB")?,
    })
}
