use crate::{bindings::cl_command_queue, AsRaw, Context, SvmCapabilities};
use std::ptr::null_mut;

pub struct CommandQueue {
    raw: cl_command_queue,
    svm: SvmCapabilities,
}

impl Context {
    #[inline]
    pub fn queue(&self) -> CommandQueue {
        let [device] = self.devices() else {
            panic!("multi-device context is not supported")
        };
        CommandQueue {
            raw: cl!(err => clCreateCommandQueue(self.as_raw(), device.as_raw(), 0, &mut err)),
            svm: device.svm_capabilities(),
        }
    }
}

impl Drop for CommandQueue {
    fn drop(&mut self) {
        cl!(clReleaseCommandQueue(self.raw))
    }
}

impl AsRaw for CommandQueue {
    type Raw = cl_command_queue;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.raw
    }
}

impl CommandQueue {
    #[inline]
    pub fn ctx(&self) -> Context {
        let mut raw = null_mut();
        let mut size = 0;
        cl!(clGetCommandQueueInfo(
            self.raw,
            CL_QUEUE_CONTEXT,
            size_of_val(&raw),
            &mut raw as *mut _ as _,
            &mut size,
        ));
        cl!(clRetainContext(raw));
        unsafe { Context::from_raw(raw) }
    }

    #[inline]
    pub fn finish(&self) {
        cl!(clFinish(self.raw))
    }

    #[inline]
    pub fn fine_grain_svm(&self) -> bool {
        self.svm.fine_grain_buffer()
    }
}

#[test]
fn test() {
    for platform in crate::Platform::all() {
        for device in platform.devices() {
            let ctx = device.context();
            let queue = ctx.queue();
            assert_eq!(unsafe { queue.ctx().as_raw() }, unsafe { ctx.as_raw() });
        }
    }
}
