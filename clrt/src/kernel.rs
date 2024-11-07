use crate::AsRaw;
use cl3::kernel::{cl_kernel, release_kernel};

#[repr(transparent)]
pub struct Kernel(pub(crate) cl_kernel);

impl AsRaw for Kernel {
    type Raw = cl_kernel;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Drop for Kernel {
    fn drop(&mut self) {
        unsafe { release_kernel(self.0).unwrap() }
    }
}
