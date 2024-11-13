#![feature(internal_output_capture)]

mod unix;
mod windows;

#[cfg(unix)]
pub fn print_error<F: FnOnce()>(cb: F) {
    unix::print_error(cb);
}

#[cfg(windows)]
pub fn print_error<F: FnOnce()>(cb: F) {
    windows::print_error(cb);
}
