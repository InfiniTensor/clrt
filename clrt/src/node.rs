use crate::{AsRaw, Event};
use cl3::{event::cl_event, ext::cl_uint};
use std::{
    marker::PhantomData,
    mem::{forget, take},
    ptr::null_mut,
};

pub struct EventNode {
    to_wait: Vec<cl_event>,
    to_record: Option<cl_event>,
}

unsafe impl Send for EventNode {}
unsafe impl Sync for EventNode {}

impl EventNode {
    pub fn new(events: impl IntoIterator<Item = Event>, record: bool) -> Self {
        Self {
            to_wait: events
                .into_iter()
                .map(|e| {
                    let ptr = unsafe { e.as_raw() };
                    forget(e);
                    ptr
                })
                .collect(),
            to_record: if record { Some(null_mut()) } else { None },
        }
    }
}

impl Drop for EventNode {
    fn drop(&mut self) {
        for ptr in take(&mut self.to_wait) {
            drop(Event(ptr))
        }
        if let Some(ptr) = self.to_record {
            if !ptr.is_null() {
                drop(Event(ptr))
            }
        }
    }
}

impl EventNode {
    pub fn take(mut self) -> Option<Event> {
        self.to_record
            .take()
            .filter(|ptr| !ptr.is_null())
            .map(Event)
    }
}

pub(crate) struct NodeParts<'a> {
    pub num_events_in_wait_list: cl_uint,
    pub event_wait_list: *const cl_event,
    pub event: *mut cl_event,
    _phantom: PhantomData<&'a ()>,
}

pub(crate) fn destruct(node: Option<&mut EventNode>) -> NodeParts {
    match node {
        Some(EventNode { to_wait, to_record }) => NodeParts {
            num_events_in_wait_list: to_wait.len() as _,
            event_wait_list: to_wait.as_ptr(),
            event: if to_record.is_some() {
                to_record.as_slice().as_ptr().cast_mut()
            } else {
                null_mut()
            },
            _phantom: PhantomData,
        },
        None => NodeParts {
            num_events_in_wait_list: 0,
            event_wait_list: null_mut(),
            event: null_mut(),
            _phantom: PhantomData,
        },
    }
}

#[test]
fn test_empty_node() {
    let node = EventNode::new([], true);
    assert!(node.take().is_none())
}
