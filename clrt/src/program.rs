use crate::{
    bindings::{
        clBuildProgram, clCreateKernel, cl_int, cl_program, CL_BUILD_PROGRAM_FAILURE,
        CL_INVALID_KERNEL_NAME, NO_ERR,
    },
    kernel::Kernel,
    AsRaw, Context,
};
use std::{ffi::CStr, ptr::null_mut};

#[repr(transparent)]
pub struct Program(cl_program);

#[derive(Clone, Debug)]
pub enum BuildError {
    BuildFailed(String),
    Others(cl_int),
}

impl Context {
    pub fn build_from_source(
        &self,
        source: &str,
        options: impl AsRef<CStr>,
    ) -> Result<Program, BuildError> {
        let mut str = source.as_ptr().cast();
        let len = source.len();
        let program =
            cl!(err => clCreateProgramWithSource(self.as_raw(), 1, &mut str, &len, &mut err));

        let [device] = self.devices() else {
            panic!("multi-device context is not supported")
        };
        let device = unsafe { device.as_raw() };
        match unsafe {
            clBuildProgram(
                program,
                1,
                &device,
                options.as_ref().as_ptr(),
                None,
                null_mut(),
            )
        } {
            NO_ERR => Ok(Program(program)),
            CL_BUILD_PROGRAM_FAILURE => {
                let mut size = 0;
                cl!(clGetProgramBuildInfo(
                    program,
                    device,
                    CL_PROGRAM_BUILD_LOG,
                    0,
                    null_mut(),
                    &mut size
                ));
                let mut log = vec![0u8; size];
                cl!(clGetProgramBuildInfo(
                    program,
                    device,
                    CL_PROGRAM_BUILD_LOG,
                    size,
                    log.as_mut_ptr().cast(),
                    &mut size
                ));
                assert_eq!(size, log.len());
                cl!(clReleaseProgram(program));

                Err(BuildError::BuildFailed(String::from_utf8(log).unwrap()))
            }
            err => Err(BuildError::Others(err)),
        }
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
    pub fn kernels(&self) -> Vec<Kernel> {
        let mut num = 0;
        cl!(clCreateKernelsInProgram(self.0, 0, null_mut(), &mut num));

        let mut kernels = vec![null_mut(); num as usize];
        cl!(clCreateKernelsInProgram(
            self.0,
            num,
            kernels.as_mut_ptr(),
            &mut num
        ));
        assert_eq!(kernels.len(), num as _);

        kernels.into_iter().map(Kernel).collect()
    }

    pub fn get_kernel(&self, name: impl AsRef<CStr>) -> Option<Kernel> {
        let mut err = 0;
        let kernel = unsafe { clCreateKernel(self.0, name.as_ref().as_ptr(), &mut err) };
        match err {
            NO_ERR => Some(Kernel(kernel)),
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
    const WRONG_SOURCE: &str = "#error Error in source code";

    for platform in crate::Platform::all() {
        for device in platform.devices() {
            let context = device.context();
            let program = context
                .build_from_source(PROGRAM_SOURCE, CString::default())
                .unwrap();
            assert!(program
                .get_kernel(CString::new("saxpy_float").unwrap())
                .is_some());
            assert!(program
                .get_kernel(CString::new("saxpy_double").unwrap())
                .is_none());

            match context.build_from_source(WRONG_SOURCE, CString::default()) {
                Err(BuildError::BuildFailed(log)) => println!("Build log: {log}"),
                _ => panic!("Error in source code should be caught"),
            }
        }
    }
}
