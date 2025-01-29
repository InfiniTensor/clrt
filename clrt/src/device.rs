use crate::{
    bindings::{clGetDeviceIDs, cl_device_id, cl_uint, CL_DEVICE_NAME, CL_DEVICE_TYPE_ALL},
    AsRaw, Platform, SvmCapabilities,
};
use std::{ffi::c_void, ptr::null_mut};

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

    #[inline]
    fn query(&self, key: cl_uint, val_size: usize, val: *mut c_void, size_ret: &mut usize) {
        cl!(clGetDeviceInfo(self.as_raw(), key, val_size, val, size_ret))
    }
}

impl Device {
    #[inline]
    pub fn name(&self) -> String {
        self.query_string(CL_DEVICE_NAME)
    }

    #[inline]
    pub fn svm_capabilities(&self) -> SvmCapabilities {
        use crate::bindings::{cl_device_svm_capabilities, CL_DEVICE_SVM_CAPABILITIES};
        self.query_value::<cl_device_svm_capabilities>(CL_DEVICE_SVM_CAPABILITIES)
            .into()
    }

    #[inline]
    pub fn max_work_dim(&self) -> usize {
        use crate::bindings::CL_DEVICE_MAX_WORK_ITEM_DIMENSIONS;
        self.query_value::<cl_uint>(CL_DEVICE_MAX_WORK_ITEM_DIMENSIONS) as _
    }

    #[inline]
    pub fn max_group_size(&self) -> usize {
        use crate::bindings::CL_DEVICE_MAX_WORK_GROUP_SIZE;
        self.query_value(CL_DEVICE_MAX_WORK_GROUP_SIZE)
    }
}

#[test]
fn probe_devices() {
    for platform in crate::Platform::all() {
        println!("{} ({})", platform.name(), platform.version());
        for device in platform.devices() {
            println!("  - {}", device.name());
            println!("    - SVM: {}", device.svm_capabilities());
            println!("    - max work dim: {}", device.max_work_dim());
            println!("    - max group size: {}", device.max_group_size());
        }
    }
}
