#![cfg(windows)]

pub fn print_error<F: FnOnce()>(cb: F) {
    cb();
}
