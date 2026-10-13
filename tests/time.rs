#![feature(custom_test_frameworks)]
#![test_runner(std_tests::runner)]

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[test]
fn instant_is_monotonic() {
    let mut prev = Instant::now();
    for _ in 0..1000 {
        let now = Instant::now();
        assert!(now >= prev);
        prev = now;
    }
}

#[test]
fn system_time_is_after_2020() {
    let since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    assert!(since_epoch > Duration::from_secs(1_577_836_800));
}

#[test]
fn instant_tracks_system_time() {
    let (i0, s0) = (Instant::now(), SystemTime::now());
    std::thread::sleep(Duration::from_millis(200));
    let di = i0.elapsed();
    let ds = s0.elapsed().unwrap();
    let diff = if di > ds { di - ds } else { ds - di };
    assert!(diff < Duration::from_millis(50), "instant {di:?} vs system {ds:?}");
}
