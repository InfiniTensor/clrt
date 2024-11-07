#![deny(warnings)]

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
pub use svm::{SvmBlob, SvmByte};

/// 资源的原始形式的表示。通常来自底层库的定义。
pub trait AsRaw {
    /// 原始形式的类型。
    type Raw: Unpin + 'static;
    /// # Safety
    ///
    /// The caller must ensure that the returned item is dropped before the original item.
    unsafe fn as_raw(&self) -> Self::Raw;
}
