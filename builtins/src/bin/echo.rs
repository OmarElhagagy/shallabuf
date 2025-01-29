#![no_main]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use serde_json::Value;

#[no_mangle]
/// # Safety
/// The caller must ensure that `input_ptr` is a valid pointer to a null-terminated string.
/// The caller is also responsible for managing the memory of the returned pointer.
///
/// # Panics
/// This function will panic if `CString::new` fails, which occurs if the input contains internal null bytes.
pub unsafe extern "C" fn run(input_ptr: *const c_char) -> *const c_char {
    if input_ptr.is_null() {
        return std::ptr::null();
    }

    let input = CStr::from_ptr(input_ptr);

    let Ok(json_str) = input.to_str() else {
        return std::ptr::null();
    };

    let Ok(parsed) = serde_json::from_str::<Value>(json_str) else {
        return std::ptr::null();
    };

    let Some(message) = parsed.get("message") else {
        return std::ptr::null();
    };

    match CString::new(message.to_string()) {
        Ok(data) => data.into_raw(),
        Err(_) => std::ptr::null(),
    }
}
