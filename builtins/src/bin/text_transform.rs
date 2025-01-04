#![no_main]

#[no_mangle]
pub extern "C" fn host_func(param: i32) -> i32 {
    param
}

#[no_mangle]
pub extern "C" fn _start() -> i32 {
    host_func(3)
}
