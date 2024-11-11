use crate::{
    bindings::{
        clGetDeviceIDs, clGetDeviceInfo, cl_device_id, cl_device_svm_capabilities, CL_DEVICE_NAME,
        CL_DEVICE_TYPE_ALL,
    },
    utils::query_string,
    AsRaw, Platform, SvmCapabilities,
};
use std::ptr::null_mut;

#[repr(transparent)]
pub struct Device(pub(crate) cl_device_id);

impl Platform {
    pub fn devices(&self) -> Vec<Device> {
        let mut num = 0;
        unsafe {
            clGetDeviceIDs(
                self.as_raw(),
                CL_DEVICE_TYPE_ALL as _,
                0,
                null_mut(),
                &mut num,
            )
        };

        let mut ans = vec![null_mut(); num as _];
        unsafe {
            clGetDeviceIDs(
                self.as_raw(),
                CL_DEVICE_TYPE_ALL as _,
                ans.len() as _,
                ans.as_mut_ptr(),
                &mut num,
            )
        };
        assert_eq!(num, ans.len() as _);

        ans.into_iter().map(Device).collect()
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Clone for Device {
    fn clone(&self) -> Self {
        cl!(clRetainDevice(self.0));
        Self(self.0)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        cl!(clReleaseDevice(self.0))
    }
}

impl AsRaw for Device {
    type Raw = cl_device_id;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Device {
    #[inline]
    pub fn name(&self) -> String {
        query_string(clGetDeviceInfo, self.0, CL_DEVICE_NAME)
    }

    #[inline]
    pub fn svm_capabilities(&self) -> SvmCapabilities {
        let mut ans: cl_device_svm_capabilities = 0;
        let mut size = 0;
        cl!(clGetDeviceInfo(
            self.0,
            CL_DEVICE_SVM_CAPABILITIES,
            size_of_val(&ans),
            (&mut ans) as *mut _ as _,
            &mut size,
        ));
        assert_eq!(size, size_of_val(&ans));

        ans.into()
    }
}

#[test]
fn probe_devices() {
    for platform in crate::Platform::all() {
        println!("{} v{}", platform.name(), platform.version());
        for device in platform.devices() {
            println!("  - {}", device.name());
        }
    }
}
