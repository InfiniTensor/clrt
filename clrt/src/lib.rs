#![cfg(cl)]
#![deny(warnings)]

#[macro_use]
#[allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::approx_constant
)]
pub mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    #[macro_export]
    macro_rules! cl {
        ($f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;
            #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
            let err = unsafe { $f };
            assert_eq!(err, CL_SUCCESS as cl_int);
        }};

        ($err: ident => $f:expr) => {{
            #[allow(unused_imports)]
            use $crate::bindings::*;

            let mut $err = 0;
            #[allow(unused_unsafe, clippy::macro_metavars_in_unsafe)]
            let ans = unsafe { $f };
            assert_eq!($err, CL_SUCCESS as cl_int);

            ans
        }};
    }
}

mod command_queue;
mod context;
mod device;
mod event;
mod kernel;
mod node;
mod platform;
mod program;
mod svm;

pub use command_queue::CommandQueue;
pub use context::Context;
pub use device::Device;
pub use event::Event;
pub use kernel::{Argument, Kernel};
pub use node::EventNode;
pub use platform::Platform;
pub use program::Program;
pub use svm::{SvmBlob, SvmByte, SvmCapabilities};

/// 资源的原始形式的表示。通常来自底层库的定义。
pub trait AsRaw {
    /// 原始形式的类型。
    type Raw: Unpin + 'static;
    /// # Safety
    ///
    /// The caller must ensure that the returned item is dropped before the original item.
    unsafe fn as_raw(&self) -> Self::Raw;
}

mod utils {
    use crate::bindings::{cl_int, cl_uint};
    use std::{ffi::c_void, ptr::null_mut};

    pub fn query_string<T: Copy>(
        f: unsafe extern "C" fn(T, cl_uint, usize, *mut c_void, *mut usize) -> cl_int,
        t: T,
        key: cl_uint,
    ) -> String {
        let mut size = 0;
        cl!(f(t, key, 0, null_mut(), &mut size));

        let mut ans = vec![0u8; size];
        cl!(f(t, key, ans.len(), ans.as_mut_ptr().cast(), &mut size));
        assert_eq!(size, ans.len());
        assert_eq!(ans.pop(), Some(0));

        String::from_utf8(ans).unwrap()
    }
}
