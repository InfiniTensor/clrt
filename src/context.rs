use crate::{device::SvmCapabilities, AsRaw, Device};
use cl3::{
    context::{cl_context, create_context, release_context},
    device::cl_device_id,
};
use std::ptr::{null, null_mut};

pub struct Context {
    raw: cl_context,
    dev: cl_device_id,
    svm: SvmCapabilities,
}

impl Device {
    #[inline]
    pub fn context(&self) -> Context {
        let device = unsafe { self.as_raw() };
        let context = create_context(&[device], null(), None, null_mut()).unwrap();
        Context {
            raw: context,
            dev: device,
            svm: self.svm_capabilities(),
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { release_context(self.raw) }.unwrap()
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl AsRaw for Context {
    type Raw = cl_context;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.raw
    }
}

impl Context {
    #[inline]
    pub(crate) unsafe fn device(&self) -> cl_device_id {
        self.dev
    }

    #[inline]
    pub(crate) fn svm_capabilities(&self) -> SvmCapabilities {
        self.svm
    }
}

#[test]
fn test() {
    use crate::Platform;

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

    for &raw in &raws {
        assert!(create_context(&[raw], null(), None, null_mut()).is_ok());
    }
    if nplatform > 1 {
        assert!(create_context(&raws, null(), None, null_mut()).is_err());
    }
}
