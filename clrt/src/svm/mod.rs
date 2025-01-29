mod capabilities;
mod map;

use crate::{
    bindings::{clSVMAlloc, clSVMFree, CL_MAP_READ, CL_MAP_WRITE, CL_MEM_READ_WRITE},
    node::{destruct, NodeParts},
    AsRaw, CommandQueue, Context, EventNode,
};
use std::{
    alloc::Layout,
    ffi::c_void,
    mem::forget,
    ops::{Deref, DerefMut},
    ptr::{null, null_mut, NonNull},
    slice::{from_raw_parts, from_raw_parts_mut},
};

pub use capabilities::SvmCapabilities;
pub use map::SvmMap;

#[repr(transparent)]
pub struct SvmByte(u8);

pub struct SvmBlob {
    ctx: Context,
    ptr: NonNull<SvmByte>,
    len: usize,
}

unsafe impl Send for SvmBlob {}
unsafe impl Sync for SvmBlob {}

impl Context {
    pub fn malloc<T: Copy>(&self, len: usize) -> SvmBlob {
        let layout = Layout::array::<T>(len).unwrap();
        let len = layout.size();

        SvmBlob {
            ctx: self.clone(),
            ptr: if len == 0 {
                NonNull::dangling()
            } else {
                let ptr = unsafe {
                    clSVMAlloc(
                        self.as_raw(),
                        CL_MEM_READ_WRITE as _,
                        len,
                        layout.align() as _,
                    )
                };
                NonNull::new(ptr).unwrap().cast()
            },
            len,
        }
    }
}

impl Drop for SvmBlob {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe { clSVMFree(self.ctx.as_raw(), self.ptr.as_ptr().cast()) }
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

#[repr(transparent)]
pub struct SvmBlobMapped(SvmBlob);

impl AsRaw for SvmBlobMapped {
    type Raw = *mut u8;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0.ptr.as_ptr().cast()
    }
}

impl Deref for SvmBlobMapped {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        let len = self.0.len;
        if len == 0 {
            &[]
        } else {
            unsafe { from_raw_parts(self.0.ptr.as_ptr().cast(), len) }
        }
    }
}

impl DerefMut for SvmBlobMapped {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let len = self.0.len;
        if len == 0 {
            &mut []
        } else {
            unsafe { from_raw_parts_mut(self.0.ptr.as_ptr().cast(), len) }
        }
    }
}

impl CommandQueue {
    pub fn map_blob(&self, mut blob: SvmBlob) -> SvmBlobMapped {
        self.map_(
            blob.as_mut_ptr().cast(),
            blob.len(),
            CL_MAP_READ | CL_MAP_WRITE,
            None,
        );
        self.finish();
        SvmBlobMapped(blob)
    }

    pub fn unmap_blob(&self, mut blob: SvmBlobMapped) -> SvmBlob {
        if !self.fine_grain_svm() {
            cl!(clEnqueueSVMUnmap(
                self.as_raw(),
                blob.0.as_mut_ptr().cast(),
                0,
                null(),
                null_mut()
            ))
        }
        blob.0
    }

    pub fn free(&self, blob: SvmBlob, event: Option<&mut EventNode>) {
        let mut ptr = blob.ptr.as_ptr().cast();
        forget(blob);

        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(event);
        cl!(clEnqueueSVMFree(
            self.as_raw(),
            1,
            &mut ptr,
            None,
            null_mut(),
            num_events_in_wait_list,
            event_wait_list,
            event,
        ))
    }

    pub fn free_mapped(&self, blob: SvmBlobMapped, event: Option<&mut EventNode>) {
        self.free(blob.0, event)
    }

    pub fn memcpy(&self, dst: &mut [SvmByte], src: &[SvmByte], event: Option<&mut EventNode>) {
        assert_eq!(size_of_val(dst), size_of_val(src));
        self.memcpy_any(
            dst.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            size_of_val(src),
            event,
        )
    }

    pub fn memcpy_from_host<T: Copy>(
        &self,
        dst: &mut [SvmByte],
        src: &[T],
        event: Option<&mut EventNode>,
    ) {
        assert_eq!(size_of_val(dst), size_of_val(src));
        self.memcpy_any(
            dst.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            size_of_val(src),
            event,
        )
    }

    pub fn memcpy_to_host<T: Copy>(
        &self,
        dst: &mut [T],
        src: &[SvmByte],
        event: Option<&mut EventNode>,
    ) {
        assert_eq!(size_of_val(dst), size_of_val(src));
        self.memcpy_any(
            dst.as_mut_ptr().cast(),
            src.as_ptr().cast(),
            size_of_val(src),
            event,
        )
    }

    fn memcpy_any(
        &self,
        dst: *mut c_void,
        src: *const c_void,
        len: usize,
        event: Option<&mut EventNode>,
    ) {
        let NodeParts {
            num_events_in_wait_list,
            event_wait_list,
            event,
            ..
        } = destruct(event);
        cl!(clEnqueueSVMMemcpy(
            self.as_raw(),
            CL_FALSE,
            dst,
            src,
            len,
            num_events_in_wait_list,
            event_wait_list,
            event,
        ))
    }
}

#[test]
fn test() {
    for platform in crate::Platform::all() {
        println!("- {}", platform.name());
        for device in platform.devices() {
            let capabilities = device.svm_capabilities();
            println!("  - {}: {capabilities}", device.name());
            if capabilities.coarse_grain_buffer() {
                let ctx = device.context();
                let _blob = ctx.malloc::<u8>(1 << 20);
                let _blob = ctx.malloc::<u8>(0);
            }
        }
    }
}
