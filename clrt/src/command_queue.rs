use crate::{bindings::cl_command_queue, AsRaw, Context, SvmCapabilities};

pub struct CommandQueue {
    raw: cl_command_queue,
    svm: SvmCapabilities,
}

impl Context {
    #[inline]
    pub fn queue(&self) -> CommandQueue {
        CommandQueue {
            raw: cl!(err => clCreateCommandQueue(self.as_raw(), self.device().as_raw(), 0, &mut err)),
            svm: self.svm_capabilities(),
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
    pub fn finish(&self) {
        cl!(clFinish(self.raw))
    }

    #[inline]
    pub fn fine_grain_svm(&self) -> bool {
        self.svm.fine_grain_buffer()
    }
}
