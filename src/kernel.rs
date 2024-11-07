use crate::{AsRaw, SvmByte};
use cl3::kernel::{cl_kernel, release_kernel, set_kernel_arg, set_kernel_arg_svm_pointer};
use half::{bf16, f16};

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

pub trait Argument {
    fn set(&self, kernel: &mut Kernel, index: usize);
}

macro_rules! impl_for_num {
    ($($ty:ty)+) => {
        $(
            impl Argument for $ty {
                #[inline]
                fn set(&self, kernel: &mut Kernel, index: usize) {
                    unsafe {
                        set_kernel_arg(
                            kernel.0,
                            index as _,
                            size_of::<Self>(),
                            (self as *const Self).cast(),
                        )
                    }
                    .unwrap()
                }
            }
        )+
    };
}

impl_for_num! {
    u8    i8
    u16   i16   f16 bf16
    u32   i32   f32
    u64   i64   f64
    u128  i128
    usize isize
}

impl Argument for *const SvmByte {
    #[inline]
    fn set(&self, kernel: &mut Kernel, index: usize) {
        unsafe { set_kernel_arg_svm_pointer(kernel.0, index as _, self.cast()) }.unwrap()
    }
}
