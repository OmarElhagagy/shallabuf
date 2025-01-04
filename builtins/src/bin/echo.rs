extern "C" {
    fn host_func(param: i32) -> i32;
}

fn main() {
    unsafe {
        let _ = host_func(42);
    }
}
