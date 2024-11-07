use std::fmt;

use cl3::{
    device::{
        cl_device_svm_capabilities, CL_DEVICE_SVM_ATOMICS, CL_DEVICE_SVM_COARSE_GRAIN_BUFFER,
        CL_DEVICE_SVM_FINE_GRAIN_BUFFER, CL_DEVICE_SVM_FINE_GRAIN_SYSTEM,
    },
    info_type::InfoType,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct SvmCapabilities(cl_device_svm_capabilities);

impl From<InfoType> for SvmCapabilities {
    #[inline]
    fn from(value: InfoType) -> Self {
        Self(value.into())
    }
}

impl SvmCapabilities {
    #[inline]
    pub fn coarse_grain_buffer(&self) -> bool {
        self.0 & CL_DEVICE_SVM_COARSE_GRAIN_BUFFER != 0
    }

    #[inline]
    pub fn fine_grain_buffer(&self) -> bool {
        self.0 & CL_DEVICE_SVM_FINE_GRAIN_BUFFER != 0
    }

    #[inline]
    pub fn fine_grain_system(&self) -> bool {
        self.0 & CL_DEVICE_SVM_FINE_GRAIN_SYSTEM != 0
    }

    #[inline]
    pub fn atomics(&self) -> bool {
        self.0 & CL_DEVICE_SVM_ATOMICS != 0
    }
}

impl fmt::Display for SvmCapabilities {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        if self.coarse_grain_buffer() {
            if first {
                first = false;
            }
            write!(f, "Coarse")?
        }
        if self.fine_grain_buffer() {
            if first {
                first = false;
            } else {
                write!(f, " + ")?
            }
            write!(f, "Fine-Buf")?
        }
        if self.fine_grain_system() {
            if first {
                first = false;
            } else {
                write!(f, " + ")?
            }
            write!(f, "Fine-Sys")?
        }
        if self.atomics() {
            if !first {
                write!(f, " + ")?
            }
            write!(f, "Atomics")?
        }
        Ok(())
    }
}
