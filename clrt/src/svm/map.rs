use super::SvmByte;
use crate::{AsRaw, CommandQueue};
use cl3::{
    ext::{clEnqueueSVMMap, clEnqueueSVMUnmap, CL_MAP_READ, CL_MAP_WRITE, CL_TRUE},
    gl::CL_SUCCESS,
};
use std::{
    ptr::{null, null_mut},
    slice::from_raw_parts_mut,
};

impl CommandQueue {
    pub fn use_on_host(&self, mem: &mut [SvmByte], f: impl FnOnce(&mut [u8])) {
        let ptr = mem.as_mut_ptr();
        let len = mem.len();
        let need_map = !self.fine_grain_svm();
        if need_map {
            assert_eq!(CL_SUCCESS, unsafe {
                clEnqueueSVMMap(
                    self.as_raw(),
                    CL_TRUE,
                    CL_MAP_READ | CL_MAP_WRITE,
                    ptr.cast(),
                    len,
                    0,
                    null(),
                    null_mut(),
                )
            })
        } else {
            self.finish();
        }
        f(unsafe { from_raw_parts_mut(ptr.cast(), len) });
        if need_map {
            assert_eq!(CL_SUCCESS, unsafe {
                clEnqueueSVMUnmap(self.as_raw(), ptr.cast(), 0, null(), null_mut())
            })
        }
    }
}

#[test]
fn test_map() {
    for platform in crate::Platform::list() {
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
            queue.use_on_host(&mut svm, |mem| {
                let mem = unsafe {
                    from_raw_parts_mut(mem.as_mut_ptr().cast::<u32>(), mem.len() / size_of::<u32>())
                };
                for x in mem {
                    *x *= 2;
                }
            });
            queue.memcpy_to_host(&mut host, &svm, None);
            queue.finish();

            assert!(host
                .into_iter()
                .enumerate()
                .all(|(i, x)| x as usize == 2 * i));
        }
    }
}
