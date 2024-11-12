use crate::{bindings::cl_event, AsRaw, Context};
use std::{borrow::Borrow, mem::transmute, ops::Deref};

#[repr(transparent)]
pub struct Event(pub(crate) cl_event);

impl Event {
    #[inline]
    pub(crate) fn from_ref(event: &cl_event) -> &Self {
        unsafe { transmute(event) }
    }
}

impl<'a> From<&'a cl_event> for &'a Event {
    #[inline]
    fn from(value: &'a cl_event) -> Self {
        Event::from_ref(value)
    }
}

impl From<UserEvent> for Event {
    #[inline]
    fn from(value: UserEvent) -> Self {
        value.0
    }
}

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

#[derive(Clone)]
#[repr(transparent)]
pub struct UserEvent(Event);

impl Context {
    #[inline]
    pub fn user_event(&self) -> UserEvent {
        UserEvent(Event(
            cl!(err => clCreateUserEvent(self.as_raw(), &mut err)),
        ))
    }
}

impl UserEvent {
    #[inline]
    pub fn complete(&self) {
        cl!(clSetUserEventStatus(self.0 .0, CL_COMPLETE as _))
    }
}

impl AsRaw for UserEvent {
    type Raw = cl_event;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0 .0
    }
}

impl AsRef<Event> for UserEvent {
    #[inline]
    fn as_ref(&self) -> &Event {
        &self.0
    }
}

impl Borrow<Event> for UserEvent {
    #[inline]
    fn borrow(&self) -> &Event {
        &self.0
    }
}

impl Deref for UserEvent {
    type Target = Event;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test]
fn test() {
    for platform in crate::Platform::all() {
        for device in platform.devices() {
            let ctx = device.context();
            let queue = ctx.queue();
            let user_event = ctx.user_event();

            queue.wait(&user_event);
            user_event.complete();
            queue.finish();
        }
    }
}
