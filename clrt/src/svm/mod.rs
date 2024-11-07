mod map;

use crate::{
    node::{destruct, NodeParts},
    AsRaw, CommandQueue, Context, EventNode,
};
use cl3::{
    context::{cl_context, release_context, retain_context},
    ext::{
        clEnqueueSVMFree, clEnqueueSVMMap, clEnqueueSVMMemcpy, clSVMAlloc, clSVMFree, CL_FALSE,
        CL_MAP_READ, CL_MEM_READ_WRITE, CL_TRUE,
    },
    gl::CL_SUCCESS,
};
use std::{
    alloc::Layout,
    ffi::c_void,
    mem::forget,
    ops::{Deref, DerefMut},
    ptr::{null_mut, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};

#[repr(transparent)]
pub struct SvmByte(u8);

pub struct SvmBlob {
    ctx: cl_context,
    ptr: NonNull<SvmByte>,
    len: usize,
}

unsafe impl Send for SvmBlob {}
unsafe impl Sync for SvmBlob {}

impl Context {
    pub fn malloc<T: Copy>(&self, len: usize) -> SvmBlob {
        let layout = Layout::array::<T>(len).unwrap();
        let context = unsafe {
            let raw = self.as_raw();
            retain_context(raw).unwrap();
            raw
        };
        let ptr = unsafe {
            clSVMAlloc(
                context,
                CL_MEM_READ_WRITE,
                layout.size(),
                layout.align() as _,
            )
        };
        SvmBlob {
            ctx: context,
            ptr: NonNull::new(ptr).unwrap().cast(),
            len: layout.size(),
        }
    }
}

impl Drop for SvmBlob {
    fn drop(&mut self) {
        unsafe {
            clSVMFree(self.ctx, self.ptr.as_ptr().cast());
            release_context(self.ctx).unwrap()
        }
    }
}

impl AsRaw for SvmBlob {
    type Raw = *mut SvmByte;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.ptr.as_ptr()
    }
}

impl Deref for SvmBlob {
    type Target = [SvmByte];
    #[inline]
    fn deref(&self) -> &Self::Target {
        let len = self.len;
        if len == 0 {
            &[]
        } else {
            unsafe { from_raw_parts(self.ptr.as_ptr(), len) }
        }
    }
}

impl DerefMut for SvmBlob {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.len;
        if len == 0 {
            &mut []
        } else {
            unsafe { from_raw_parts_mut(self.ptr.as_ptr(), len) }
        }
    }
}

impl CommandQueue {
    pub fn free(&self, blob: SvmBlob, node: Option<&mut EventNode>) {
        let ptr = blob.ptr.as_ptr().cast_const().cast();
        forget(blob);

        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(node);
        let result = unsafe {
            clEnqueueSVMFree(
                self.as_raw(),
                1,
                &ptr,
                None,
                null_mut(),
                num_events_in_wait_list,
                event_wait_list,
                event,
            )
        };
        assert_eq!(result, CL_SUCCESS)
    }

    pub fn memcpy(&self, dst: &mut [SvmByte], src: &[SvmByte], node: Option<&mut EventNode>) {
        assert_eq!(size_of_val(dst), size_of_val(src));
        self.memcpy_any(
            dst.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            size_of_val(src),
            node,
        )
    }

    pub fn memcpy_from_host<T: Copy>(
        &self,
        dst: &mut [SvmByte],
        src: &[T],
        node: Option<&mut EventNode>,
    ) {
        assert_eq!(size_of_val(dst), size_of_val(src));
        self.memcpy_any(
            dst.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            size_of_val(src),
            node,
        )
    }

    pub fn memcpy_to_host<T: Copy>(
        &self,
        dst: &mut [T],
        src: &[SvmByte],
        node: Option<&mut EventNode>,
    ) {
        assert_eq!(size_of_val(dst), size_of_val(src));
        self.memcpy_any(
            dst.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            size_of_val(src),
            node,
        )
    }

    fn memcpy_any(
        &self,
        dst: *mut c_void,
        src: *const c_void,
        len: usize,
        node: Option<&mut EventNode>,
    ) {
        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(node);
        assert_eq!(CL_SUCCESS, unsafe {
            clEnqueueSVMMemcpy(
                self.as_raw(),
                CL_FALSE,
                dst,
                src,
                len,
                num_events_in_wait_list,
                event_wait_list,
                event,
            )
        })
    }

    pub fn map<'r>(&self, slice: &'r [SvmByte], node: Option<&mut EventNode>) -> &'r [u8] {
        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(node);
        assert_eq!(CL_SUCCESS, unsafe {
            clEnqueueSVMMap(
                self.as_raw(),
                CL_TRUE,
                CL_MAP_READ,
                slice.as_ptr().cast_mut().cast(),
                slice.len(),
                num_events_in_wait_list,
                event_wait_list,
                event,
            )
        });
        unsafe { from_raw_parts(slice.as_ptr().cast(), slice.len()) }
    }
}

#[test]
fn test() {
    for platform in crate::Platform::list() {
        println!("- {}", platform.name());
        for device in platform.devices() {
            let capabilities = device.svm_capabilities();
            println!("  - {}: {capabilities}", device.name());
            if capabilities.coarse_grain_buffer() {
                let ctx = device.context();
                let _blob = ctx.malloc::<u8>(1 << 20);
            }
        }
    }
}
