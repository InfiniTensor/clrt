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
        let ver = query_string(clGetPlatformInfo, self.0, CL_PLATFORM_VERSION);
        // See <https://registry.khronos.org/OpenCL/specs/3.0-unified/html/OpenCL_API.html#CL_PLATFORM_VERSION>
        let ver = ver
            .strip_prefix("OpenCL ")
            .expect("Version string should start with 'OpenCL '");
        let (num, specific) = ver.split_once(' ').unwrap_or((ver, ""));

        let (major, minor) = num.split_once('.').unwrap();
        Version {
            major: major.parse().unwrap(),
            minor: minor.parse().unwrap(),
            specific: specific.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Version {
    major: u32,
    minor: u32,
    specific: String,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OpenCL {}.{}", self.major, self.minor)?;
        if !self.specific.is_empty() {
            write!(f, " {}", self.specific)?;
        }
        Ok(())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering::Equal;
        Some(match self.major.cmp(&other.major) {
            Equal => self.minor.cmp(&other.minor),
            ord => ord,
        })
    }
}

#[test]
fn test() {
    for platform in Platform::all() {
        println!("{} ({})", platform.name(), platform.version())
    }
}
