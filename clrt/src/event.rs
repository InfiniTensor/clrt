use crate::{bindings::cl_event, AsRaw};

#[repr(transparent)]
pub struct Event(pub(crate) cl_event);

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl Drop for Event {
    fn drop(&mut self) {
        cl!(clReleaseEvent(self.0))
    }
}

impl AsRaw for Event {
    type Raw = cl_event;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Clone for Event {
    fn clone(&self) -> Self {
        cl!(clRetainEvent(self.0));
        Self(self.0)
    }
}

impl Event {
    #[inline]
    pub fn wait(&self) {
        cl!(clWaitForEvents(1, &self.0))
    }
}
