mod svm_capabilities;

use crate::{AsRaw, Platform};
use cl3::{
    device::{
        cl_device_id, cl_device_info, get_device_ids, get_device_info, CL_DEVICE_NAME,
        CL_DEVICE_SVM_CAPABILITIES, CL_DEVICE_TYPE_ALL,
    },
    info_type::InfoType,
};

pub use svm_capabilities::SvmCapabilities;

#[repr(transparent)]
pub struct Device(cl_device_id);

impl Platform {
    pub fn devices(&self) -> Vec<Device> {
        get_device_ids(unsafe { self.as_raw() }, CL_DEVICE_TYPE_ALL)
            .unwrap()
            .into_iter()
            .map(Device)
            .collect()
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
        self.get_info(CL_DEVICE_NAME).into()
    }

    #[inline]
    pub fn svm_capabilities(&self) -> SvmCapabilities {
        self.get_info(CL_DEVICE_SVM_CAPABILITIES).into()
    }

    #[inline(always)]
    fn get_info(&self, param_name: cl_device_info) -> InfoType {
        get_device_info(self.0, param_name).unwrap()
    }
}

#[test]
fn probe_devices() {
    for platform in crate::Platform::list() {
        println!("- {}", platform.name());
        for device in platform.devices() {
            println!("  - {}", device.name());
        }
    }
}
