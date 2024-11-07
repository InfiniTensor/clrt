use crate::{kernel::Kernel, AsRaw, Context};
use cl3::{
    kernel::create_kernel,
    program::{build_program, cl_program, create_program_with_source, release_program},
};
use std::{ffi::CStr, ptr::null_mut};

#[repr(transparent)]
pub struct Program(cl_program);

unsafe impl Send for Program {}
unsafe impl Sync for Program {}

impl Context {
    pub fn build_from_source(&self, source: &str, options: impl AsRef<CStr>) -> Program {
        let program = create_program_with_source(unsafe { self.as_raw() }, &[source]).unwrap();
        build_program(
            program,
            &[unsafe { self.device() }],
            options.as_ref(),
            None,
            null_mut(),
        )
        .unwrap();
        Program(program)
    }
}

impl AsRaw for Program {
    type Raw = cl_program;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { release_program(self.0).unwrap() };
    }
}

impl Program {
    #[inline]
    pub fn get_kernel(&self, name: impl AsRef<CStr>) -> Option<Kernel> {
        create_kernel(self.0, name.as_ref()).ok().map(Kernel)
    }
}

#[test]
fn test() {
    use std::ffi::CString;

    const PROGRAM_SOURCE: &str = r#"
kernel void saxpy_float (global float* z,
    global float const* x,
    global float const* y,
    float a)
{
    const size_t i = get_global_id(0);
    z[i] = a*x[i] + y[i];
}"#;

    for platform in crate::Platform::all() {
        for device in platform.devices() {
            let context = device.context();
            let program = context.build_from_source(PROGRAM_SOURCE, CString::default());
            assert!(program
                .get_kernel(CString::new("saxpy_float").unwrap())
                .is_some());
            assert!(program
                .get_kernel(CString::new("saxpy_double").unwrap())
                .is_none());
        }
    }
}
