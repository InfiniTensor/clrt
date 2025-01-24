use super::SvmByte;
use crate::{
    bindings::{CL_MAP_READ, CL_MAP_WRITE, CL_MAP_WRITE_INVALIDATE_REGION},
    node::{destruct, EventNode, NodeParts},
    AsRaw, CommandQueue,
};
use std::{
    ffi::c_void,
    mem::forget,
    ops::{Deref, DerefMut},
    ptr::{null, null_mut},
    slice::from_raw_parts_mut,
};

#[repr(transparent)]
pub struct SvmMap<'a, const R: bool, const W: bool>(&'a mut [u8]);

impl<const R_: bool, const W_: bool> Drop for SvmMap<'_, R_, W_> {
    fn drop(&mut self) {
        panic!("SvmMap should not be dropped manually")
    }
}

#[derive(Clone, Copy)]
pub struct Valid;
#[derive(Clone, Copy)]
pub struct Invalid;
pub trait Flag<const R: bool>: Copy {}
impl Flag<true> for Valid {}
impl Flag<false> for Invalid {}

impl CommandQueue {
    pub fn map<'a>(&self, mem: &'a [SvmByte]) -> SvmMap<'a, true, false> {
        let flags = CL_MAP_READ;

        let ptr = mem.as_ptr().cast_mut();
        let len = mem.len();
        self.map_(ptr.cast(), len, flags, None);
        self.finish();
        SvmMap(unsafe { from_raw_parts_mut(ptr.cast(), len) })
    }

    pub fn map_mut<'a, const R: bool>(
        &self,
        mem: &'a mut [SvmByte],
        _content: impl Flag<R>,
    ) -> SvmMap<'a, R, true> {
        let flags = if R {
            CL_MAP_READ | CL_MAP_WRITE
        } else {
            CL_MAP_WRITE_INVALIDATE_REGION
        };

        let ptr = mem.as_mut_ptr();
        let len = mem.len();
        self.map_(ptr.cast(), len, flags, None);
        self.finish();
        SvmMap(unsafe { from_raw_parts_mut(ptr.cast(), len) })
    }

    pub(super) fn map_(
        &self,
        ptr: *mut c_void,
        len: usize,
        flags: u32,
        node: Option<&mut EventNode>,
    ) {
        if !self.fine_grain_svm() && len > 0 {
            let NodeParts {
                num_events_in_wait_list,
                event_wait_list,
                event,
                ..
            } = destruct(node);
            cl!(clEnqueueSVMMap(
                self.as_raw(),
                CL_FALSE,
                flags as _,
                ptr,
                len,
                num_events_in_wait_list,
                event_wait_list,
                event,
            ))
        } else if let Some(node) = node {
            self.wait_raw(node.to_wait())
        }
    }

    pub fn unmap<const R_: bool, const W_: bool>(&self, mem: SvmMap<'_, R_, W_>) {
        if !self.fine_grain_svm() && !mem.0.is_empty() {
            cl!(clEnqueueSVMUnmap(
                self.as_raw(),
                mem.0.as_mut_ptr().cast(),
                0,
                null(),
                null_mut()
            ))
        }
        forget(mem)
    }
}

impl<const W_: bool> Deref for SvmMap<'_, true, W_> {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for SvmMap<'_, true, true> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl SvmMap<'_, false, true> {
    /// Returns a slice of the mapped memory that can be written to.
    ///
    /// # Safety
    ///
    /// The returned slice is not able to read.
    #[inline]
    pub unsafe fn write_only_slice(&mut self) -> &mut [u8] {
        self.0
    }
}

#[test]
fn test_map() {
    for platform in crate::Platform::all() {
        for device in platform.devices() {
            if !device.svm_capabilities().coarse_grain_buffer() {
                continue;
            }

            let n = 1 << 10;

            let context = device.context();
            let queue = context.queue();
            let mut svm = context.malloc::<u32>(n);
            let mut host = (0..n).map(|i| i as u32).collect::<Vec<_>>();
            queue.memcpy_from_host(&mut svm, &host, None);
            let mut map = queue.map_mut(&mut svm, Valid);
            {
                let mem = unsafe {
                    from_raw_parts_mut(map.as_mut_ptr().cast::<u32>(), map.len() / size_of::<u32>())
                };
                for x in mem {
                    *x *= 2;
                }
            }
            queue.unmap(map);
            queue.memcpy_to_host(&mut host, &svm, None);
            queue.finish();

            assert!(host
                .into_iter()
                .enumerate()
                .all(|(i, x)| x as usize == 2 * i));
        }
    }
}
