use crate::{
    bindings::{
        clGetPlatformIDs, clGetPlatformInfo, cl_platform_id, CL_PLATFORM_NAME, CL_PLATFORM_VERSION,
    },
    utils::query_string,
    AsRaw,
};
use std::{fmt, ptr::null_mut};

#[repr(transparent)]
pub struct Platform(cl_platform_id);

impl AsRaw for Platform {
    type Raw = cl_platform_id;
    #[inline]
    unsafe fn as_raw(&self) -> Self::Raw {
        self.0
    }
}

impl Platform {
    pub fn all() -> Vec<Self> {
        let mut num = 0;
        unsafe { clGetPlatformIDs(0, null_mut(), &mut num) };

        let mut ans = vec![null_mut(); num as _];
        unsafe { clGetPlatformIDs(num, ans.as_mut_ptr(), &mut num) };
        assert_eq!(num as usize, ans.len());

        ans.into_iter().map(Self).collect()
    }

    #[inline]
    pub fn name(&self) -> String {
        query_string(clGetPlatformInfo, self.0, CL_PLATFORM_NAME)
    }

    #[inline]
    pub fn version(&self) -> Version {
        let s = query_string(clGetPlatformInfo, self.0, CL_PLATFORM_VERSION);
        let mut split = s.split_whitespace();

        assert_eq!(split.next().unwrap(), "OpenCL");
        let (major, minor) = split.next().unwrap().split_once('.').unwrap();
        let specific = split.next().map(ToString::to_string).unwrap_or_default();
        assert!(split.next().is_none());

        Version {
            major: major.parse().unwrap(),
            minor: minor.parse().unwrap(),
            specific,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    specific: String,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)?;
        if !self.specific.is_empty() {
            write!(f, " \"{}\"", self.specific)?;
        }
        Ok(())
    }
}

#[test]
fn test() {
    for platform in Platform::all() {
        println!("{} v{}", platform.name(), platform.version())
    }
}
