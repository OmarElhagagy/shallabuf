#![no_main]

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

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

    let echoed_str = match input.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return std::ptr::null(),
    };

    match CString::new(echoed_str) {
        Ok(data) => data.into_raw(),
        Err(_) => std::ptr::null(),
    }
}
