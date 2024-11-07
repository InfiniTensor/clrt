use crate::{
    node::{destruct, NodeParts},
    AsRaw, CommandQueue, EventNode, SvmByte,
};
use cl3::{
    ext::clEnqueueNDRangeKernel,
    gl::CL_SUCCESS,
    kernel::{cl_kernel, release_kernel, set_kernel_arg, set_kernel_arg_svm_pointer},
};
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

impl Kernel {
    pub fn set_arg(&mut self, index: usize, value: impl Argument) -> &mut Self {
        value.set_to(self, index);
        self
    }

    pub fn launch(
        &self,
        global_work_offset: &[usize],
        global_work_size: &[usize],
        local_work_size: &[usize],
        queue: &CommandQueue,
        node: Option<&mut EventNode>,
    ) {
        let work_dim = local_work_size.len();
        assert_eq!(work_dim, global_work_offset.len());
        assert_eq!(work_dim, global_work_size.len());

        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(node);
        assert_eq!(CL_SUCCESS, unsafe {
            clEnqueueNDRangeKernel(
                queue.as_raw(),
                self.0,
                work_dim as _,
                global_work_offset.as_ptr(),
                global_work_size.as_ptr(),
                local_work_size.as_ptr(),
                num_events_in_wait_list,
                event_wait_list,
                event,
            )
        })
    }
}

pub trait Argument {
    fn set_to(&self, kernel: &mut Kernel, index: usize);
}

impl<T: Argument> Argument for &T {
    fn set_to(&self, kernel: &mut Kernel, index: usize) {
        T::set_to(*self, kernel, index)
    }
}

macro_rules! impl_for_num {
    ($($ty:ty)+) => {
        $(
            impl Argument for $ty {
                #[inline]
                fn set_to(&self, kernel: &mut Kernel, index: usize) {
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
    fn set_to(&self, kernel: &mut Kernel, index: usize) {
        unsafe { set_kernel_arg_svm_pointer(kernel.0, index as _, self.cast()) }.unwrap()
    }
}
