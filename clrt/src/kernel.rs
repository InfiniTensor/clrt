﻿use crate::{
    bindings::{cl_kernel, cl_uint},
    node::{destruct, NodeParts},
    AsRaw, CommandQueue, EventNode, SvmByte,
};
use half::{bf16, f16};
use std::ffi::c_void;

#[repr(transparent)]
pub struct Kernel(pub(crate) cl_kernel);

impl Drop for Kernel {
    fn drop(&mut self) {
        cl!(clReleaseKernel(self.0))
    }
}

impl AsRaw for Kernel {
    type Raw = cl_kernel;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }

    #[inline]
    fn query(&self, key: cl_uint, val_size: usize, val: *mut c_void, size_ret: &mut usize) {
        cl!(clGetKernelInfo(self.as_raw(), key, val_size, val, size_ret))
    }
}

impl Kernel {
    #[inline]
    pub fn name(&self) -> String {
        use crate::bindings::CL_KERNEL_FUNCTION_NAME;
        self.query_string(CL_KERNEL_FUNCTION_NAME)
    }

    #[inline]
    pub fn max_group_size(&self) -> usize {
        use crate::bindings::CL_KERNEL_WORK_GROUP_SIZE;
        self.query_value(CL_KERNEL_WORK_GROUP_SIZE)
    }

    #[inline]
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
        event: Option<&mut EventNode>,
    ) {
        let work_dim = local_work_size.len();
        assert_eq!(work_dim, global_work_offset.len());
        assert_eq!(work_dim, global_work_size.len());

        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(event);
        cl!(clEnqueueNDRangeKernel(
            queue.as_raw(),
            self.0,
            work_dim as _,
            global_work_offset.as_ptr(),
            global_work_size.as_ptr(),
            local_work_size.as_ptr(),
            num_events_in_wait_list,
            event_wait_list,
            event,
        ))
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
                     cl!(clSetKernelArg(kernel.0, index as _, size_of::<Self>(), (self as *const Self).cast()))
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
        cl!(clSetKernelArgSVMPointer(kernel.0, index as _, self.cast()))
    }
}

impl Argument for *mut SvmByte {
    #[inline]
    fn set_to(&self, kernel: &mut Kernel, index: usize) {
        cl!(clSetKernelArgSVMPointer(kernel.0, index as _, self.cast()))
    }
}
