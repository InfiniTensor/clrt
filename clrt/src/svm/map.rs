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
pub struct SvmMap<'a, const W: bool>(&'a mut [u8]);

impl<const W_: bool> Drop for SvmMap<'_, W_> {
    fn drop(&mut self) {
        panic!("SvmMap should not be dropped manually")
    }
}

impl CommandQueue {
    pub fn map<'a>(&self, mem: &'a [SvmByte]) -> SvmMap<'a, false> {
        let flags = CL_MAP_READ;

        let ptr = mem.as_ptr().cast_mut();
        let len = mem.len();
        self.map_(ptr.cast(), len, flags, None);
        self.finish();
        SvmMap(unsafe { from_raw_parts_mut(ptr.cast(), len) })
    }

    pub fn map_mut<'a>(&self, mem: &'a mut [SvmByte], readable: bool) -> SvmMap<'a, true> {
        let flags = if readable {
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
        event: Option<&mut EventNode>,
    ) {
        if !self.fine_grain_svm() && len > 0 {
            let NodeParts {
                num_events_in_wait_list,
                event_wait_list,
                event,
                ..
            } = destruct(event);
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
        } else if let Some(node) = event {
            self.wait_raw(node.to_wait())
        }
    }

    pub fn unmap<const W_: bool>(&self, mem: SvmMap<'_, W_>) {
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

impl<const W_: bool> Deref for SvmMap<'_, W_> {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DerefMut for SvmMap<'_, true> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
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
            let mut map = queue.map_mut(&mut svm, true);
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
