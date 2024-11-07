use std::ptr::null;

use crate::{device::SvmCapabilities, AsRaw, Context};
use cl3::command_queue::{
    cl_command_queue, create_command_queue_with_properties, finish, release_command_queue,
};

pub struct CommandQueue {
    raw: cl_command_queue,
    svm: SvmCapabilities,
}

impl Context {
    #[inline]
    pub fn queue(&self) -> CommandQueue {
        unsafe { create_command_queue_with_properties(self.as_raw(), self.device(), null()) }
            .map(|raw| CommandQueue {
                raw,
                svm: self.svm_capabilities(),
            })
            .unwrap()
    }
}

impl Drop for CommandQueue {
    fn drop(&mut self) {
        unsafe { release_command_queue(self.raw) }.unwrap()
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
        finish(self.raw).unwrap()
    }

    #[inline]
    pub fn fine_grain_svm(&self) -> bool {
        self.svm.fine_grain_buffer()
    }
}
