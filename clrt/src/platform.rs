use crate::AsRaw;
use cl3::platform::{cl_platform_id, get_platform_ids, get_platform_info, CL_PLATFORM_NAME};

#[repr(transparent)]
pub struct Platform(cl_platform_id);

impl AsRaw for Platform {
    type Raw = cl_platform_id;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Platform {
    #[inline]
    pub fn all() -> Vec<Self> {
        get_platform_ids().unwrap().into_iter().map(Self).collect()
    }
    #[inline]
    pub fn name(&self) -> String {
        get_platform_info(self.0, CL_PLATFORM_NAME).unwrap().into()
    }
}

#[test]
fn test() {
    for platform in Platform::all() {
        println!("{}", platform.name());
    }
}
