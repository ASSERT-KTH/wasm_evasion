use libc::c_char;


extern {
    pub fn hello();
    pub fn superoptimize(lhs: *const c_char, cb: extern "C" fn(*const c_char, i32) -> i32) -> i32;
}