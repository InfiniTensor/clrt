use crate::{kernel::Kernel, AsRaw, Context};
use cl3::{
    kernel::{
        cl_kernel, create_kernels_in_program, get_kernel_info, release_kernel, retain_kernel,
        CL_KERNEL_FUNCTION_NAME,
    },
    program::{build_program, cl_program, create_program_with_source, release_program},
};
use std::{collections::HashMap, ffi::CStr, mem::take, ptr::null_mut};

pub struct Program {
    program: cl_program,
    kernels: HashMap<String, cl_kernel>,
}

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

        let kernels = create_kernels_in_program(program)
            .unwrap()
            .into_iter()
            .map(|kernel| {
                (
                    get_kernel_info(kernel, CL_KERNEL_FUNCTION_NAME)
                        .unwrap()
                        .into(),
                    kernel,
                )
            })
            .collect();

        Program { program, kernels }
    }
}

impl AsRaw for Program {
    type Raw = cl_program;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.program
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe { release_program(self.program).unwrap() };
        for (_, kernel) in take(&mut self.kernels) {
            unsafe { release_kernel(kernel).unwrap() }
        }
    }
}

impl Program {
    pub fn get_kernel(&self, name: &str) -> Option<Kernel> {
        self.kernels.get(name).map(|&kernel| {
            unsafe { retain_kernel(kernel).unwrap() };
            Kernel(kernel)
        })
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
            let kernels = program
                .kernels
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<_>>();
            assert_eq!(kernels, ["saxpy_float"])
        }
    }
}
