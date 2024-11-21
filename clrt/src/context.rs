use crate::{
    bindings::{cl_context, cl_device_id},
    AsRaw, Device,
};
use smallvec::{smallvec, SmallVec};
use std::ptr::{null, null_mut};

pub struct Context {
    raw: cl_context,
    dev: SmallVec<[Device; 1]>,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        Context {
            raw: cl!(err => clCreateContext(null(), 1, &self.as_raw(), None, null_mut(), &mut err)),
            dev: [self.clone()].into(),
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
    /// Create a new context from a raw context handle.
    ///
    /// # Safety
    ///
    /// The raw context handle must be a valid  and retained OpenCL context handle.
    pub unsafe fn from_raw(raw: cl_context) -> Self {
        let mut size = 0;
        cl!(clGetContextInfo(
            raw,
            CL_CONTEXT_DEVICES,
            0,
            null_mut(),
            &mut size
        ));
        assert_eq!(size % size_of::<cl_device_id>(), 0);

        let mut dev: SmallVec<[cl_device_id; 1]> =
            smallvec![null_mut(); size / size_of::<cl_device_id>()];
        cl!(clGetContextInfo(
            raw,
            CL_CONTEXT_DEVICES,
            size,
            dev.as_mut_ptr().cast(),
            &mut size
        ));
        assert_eq!(size, size_of_val(dev.as_slice()));

        Self {
            raw,
            dev: dev.into_iter().map(Device).collect(),
        }
    }

    #[inline]
    pub(crate) fn devices(&self) -> &[Device] {
        &self.dev
    }
}

#[test]
fn test() {
    use crate::{
        bindings::{clCreateContext, NO_ERR},
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
        assert_eq!(err, NO_ERR)
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
        assert_ne!(err, NO_ERR)
    }
}

#[test]
fn test_context_from_raw() {
    for platform in crate::Platform::all() {
        for dev in platform.devices() {
            let ctx = dev.context();
            let raw = unsafe { ctx.as_raw() };
            cl!(clRetainContext(raw));
            let _ctx2 = unsafe { Context::from_raw(raw) };
        }
    }
}
