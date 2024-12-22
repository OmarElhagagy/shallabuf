#[no_mangle]
pub extern "C" fn host_func(param: i32) -> i32 {
    param
}

#[no_mangle]
pub extern "C" fn hello() -> i32 {
    host_func(3)
}

fn main() {
    println!("{}", hello());
}
