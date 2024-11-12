use crate::{
    bindings::{clCreateKernel, cl_int, cl_program, CL_INVALID_KERNEL_NAME, CL_SUCCESS},
    kernel::Kernel,
    AsRaw, Context,
};
use std::{ffi::CStr, ptr::null_mut};

#[repr(transparent)]
pub struct Program(cl_program);

impl Context {
    pub fn build_from_source(&self, source: &str, options: impl AsRef<CStr>) -> Program {
        let mut str = source.as_ptr().cast();
        let len = source.len();
        let program =
            cl!(err => clCreateProgramWithSource(self.as_raw(), 1, &mut str, &len, &mut err));

        let [device] = self.devices() else {
            panic!("multi-device context is not supported")
        };
        let device = unsafe { device.as_raw() };
        cl!(clBuildProgram(
            program,
            1,
            &device,
            options.as_ref().as_ptr(),
            None,
            null_mut()
        ));

        Program(program)
    }
}

unsafe impl Send for Program {}
unsafe impl Sync for Program {}

impl Clone for Program {
    fn clone(&self) -> Self {
        cl!(clRetainProgram(self.0));
        Self(self.0)
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        cl!(clReleaseProgram(self.0))
    }
}

impl AsRaw for Program {
    type Raw = cl_program;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Program {
    #[inline]
    pub fn get_kernel(&self, name: impl AsRef<CStr>) -> Option<Kernel> {
        const OK: cl_int = CL_SUCCESS as _;

        let mut err = 0;
        let kernel = unsafe { clCreateKernel(self.0, name.as_ref().as_ptr(), &mut err) };
        match err {
            OK => Some(Kernel(kernel)),
            CL_INVALID_KERNEL_NAME => None,
            _ => panic!("clCreateKernel failed with error code {err}"),
        }
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
