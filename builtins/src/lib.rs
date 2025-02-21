use std::ffi::{c_char, CString};

#[repr(C)]
pub struct RunOutput {
    pub data_ptr: *const c_char,
    pub data_len: u32,
}

impl Default for RunOutput {
    fn default() -> Self {
        RunOutput {
            data_ptr: std::ptr::null(),
            data_len: 0,
        }
    }
}

impl From<String> for RunOutput {
    fn from(data: String) -> Self {
        let data_len: u32 = data.len() as u32;

        match CString::new(data) {
            Ok(data) => RunOutput {
                data_ptr: data.into_raw(),
                data_len,
            },
            Err(_) => RunOutput {
                data_ptr: std::ptr::null(),
                data_len: 0,
            },
        }
    }
}
