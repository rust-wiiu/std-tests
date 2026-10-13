//! Test runner for real hardware: runs libtest, then returns to the Wii U Menu.
//!
//! `exit` leaves the console stuck, so the next wiiload upload cannot start.
//! Each test file uses this through
//! `#![feature(custom_test_frameworks)] #![test_runner(std_tests::runner)]`.
#![feature(test)]

extern crate test;

use cafe_sys::coreinit::foreground;
use cafe_sys::{ffi, proc_ui, sysapp};

pub fn runner(tests: &[&test::TestDescAndFn]) {
    // Cafe cannot pass arguments to the app, so libtest only needs a program name.
    let _ = test::test_main(&["std-tests".to_owned()], tests);
    return_to_menu();
}

extern "C" fn save_done() {
    unsafe { foreground::ready_to_release() }
}

fn return_to_menu() {
    unsafe {
        proc_ui::init(Some(save_done));
        sysapp::launch_menu();
        loop {
            match proc_ui::process_messages(ffi::TRUE) {
                proc_ui::Status::EXITING => break,
                proc_ui::Status::RELEASE_FOREGROUND => proc_ui::draw_done_release(),
                _ => {}
            }
        }
        proc_ui::shutdown();
    }
}
