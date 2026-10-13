#![feature(custom_test_frameworks)]
#![test_runner(std_tests::runner)]

use std::collections::HashMap;
use std::io::{self, ErrorKind};

#[test]
fn args_do_not_panic() {
    let _: Vec<_> = std::env::args_os().collect();
}

#[test]
fn temp_dir_exists() {
    let dir = std::env::temp_dir();
    assert!(dir.is_absolute(), "{dir:?}");
    assert!(dir.is_dir(), "{dir:?}");
}

#[test]
fn hashmap_random_state() {
    let mut m = HashMap::new();
    for i in 0..1000 {
        m.insert(i, i * 2);
    }
    assert_eq!(m[&500], 1000);
}

#[test]
fn socket_error_codes() {
    assert_eq!(io::Error::from_raw_os_error(7).kind(), ErrorKind::ConnectionRefused);
    assert_eq!(io::Error::from_raw_os_error(6).kind(), ErrorKind::WouldBlock);
    assert!(io::Error::from_raw_os_error(7).to_string().contains("connection refused"));
}

#[test]
fn stdout_and_stderr() {
    println!("stdout works");
    eprintln!("stderr works");
}
