use std::fmt;

use crate::bindings::{
    cl_device_svm_capabilities, CL_DEVICE_SVM_ATOMICS, CL_DEVICE_SVM_COARSE_GRAIN_BUFFER,
    CL_DEVICE_SVM_FINE_GRAIN_BUFFER, CL_DEVICE_SVM_FINE_GRAIN_SYSTEM,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct SvmCapabilities(cl_device_svm_capabilities);

impl From<cl_device_svm_capabilities> for SvmCapabilities {
    #[inline]
    fn from(capabilities: cl_device_svm_capabilities) -> Self {
        Self(capabilities)
    }
}

impl SvmCapabilities {
    #[inline]
    pub fn coarse_grain_buffer(&self) -> bool {
        self.0 & CL_DEVICE_SVM_COARSE_GRAIN_BUFFER as cl_device_svm_capabilities != 0
    }

    #[inline]
    pub fn fine_grain_buffer(&self) -> bool {
        self.0 & CL_DEVICE_SVM_FINE_GRAIN_BUFFER as cl_device_svm_capabilities != 0
    }

    #[inline]
    pub fn fine_grain_system(&self) -> bool {
        self.0 & CL_DEVICE_SVM_FINE_GRAIN_SYSTEM as cl_device_svm_capabilities != 0
    }

    #[inline]
    pub fn atomics(&self) -> bool {
        self.0 & CL_DEVICE_SVM_ATOMICS as cl_device_svm_capabilities != 0
    }
}

impl fmt::Display for SvmCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fn write(f: &mut fmt::Formatter, first: &mut bool, pre: &str, info: &str) -> fmt::Result {
            if *first {
                *first = false
            } else {
                write!(f, "{pre}")?
            }
            write!(f, "{info}")
        }

        let mut first = true;
        if self.coarse_grain_buffer() {
            write(f, &mut first, "", "Coarse")?
        }
        if self.fine_grain_buffer() {
            write(f, &mut first, " + ", "Fine-Buf")?
        }
        if self.fine_grain_system() {
            write(f, &mut first, " + ", "Fine-Sys")?
        }
        if self.atomics() {
            write(f, &mut first, " + ", "Atomics")?
        }
        if first {
            write!(f, "None")?
        }
        Ok(())
    }
}
