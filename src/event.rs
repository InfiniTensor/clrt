use crate::AsRaw;
use cl3::event::{cl_event, release_event, retain_event, wait_for_events};

#[repr(transparent)]
pub struct Event(pub(crate) cl_event);

unsafe impl Send for Event {}
unsafe impl Sync for Event {}

impl Drop for Event {
    fn drop(&mut self) {
        unsafe { release_event(self.0) }.unwrap()
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
        unsafe { retain_event(self.0) }.unwrap();
        Self(self.0)
    }
}

impl Event {
    #[inline]
    pub fn wait(&self) {
        wait_for_events(&[self.0]).unwrap()
    }
}
