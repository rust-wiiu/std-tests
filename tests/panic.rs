#![feature(custom_test_frameworks)]
#![test_runner(std_tests::runner)]

#[test]
fn catch_unwind_same_thread() {
    assert!(std::panic::catch_unwind(|| panic!("expected")).is_err());
}
