use crate::{bindings::cl_context, AsRaw, Device, SvmCapabilities};
use std::ptr::{null, null_mut};

pub struct Context {
    raw: cl_context,
    dev: Device,
    svm: SvmCapabilities,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        Context {
            raw: cl!(err => clCreateContext(null(), 1, &self.as_raw(), None, null_mut(), &mut err)),
            dev: self.clone(),
            svm: self.svm_capabilities(),
        }
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl Clone for Context {
    fn clone(&self) -> Self {
        cl!(clRetainContext(self.raw));
        Self {
            raw: self.raw,
            dev: self.dev.clone(),
            svm: self.svm,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        cl!(clReleaseContext(self.raw))
    }
}

impl AsRaw for Context {
    type Raw = cl_context;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.raw
    }
}

impl Context {
    #[inline]
    pub(crate) fn device(&self) -> &Device {
        &self.dev
    }

    #[inline]
    pub(crate) fn svm_capabilities(&self) -> SvmCapabilities {
        self.svm
    }
}

#[test]
fn test() {
    use crate::{
        bindings::{clCreateContext, cl_int, CL_SUCCESS},
        Platform,
    };

    let mut nplatform = 0;
    let mut devices = Vec::new();
    for platform in Platform::all() {
        let mut vec = platform.devices();
        if !vec.is_empty() {
            nplatform += 1;
            devices.append(&mut vec);
        }
    }

    let raws = devices
        .iter()
        .map(|d| unsafe { d.as_raw() })
        .collect::<Vec<_>>();

    let mut err = 0;
    for &raw in &raws {
        unsafe { clCreateContext(null(), 1, &raw, None, null_mut(), &mut err) };
        assert_eq!(err, CL_SUCCESS as cl_int)
    }
    if nplatform > 1 {
        unsafe {
            clCreateContext(
                null(),
                raws.len() as _,
                raws.as_ptr(),
                None,
                null_mut(),
                &mut err,
            )
        };
        assert_ne!(err, CL_SUCCESS as cl_int)
    }
}
