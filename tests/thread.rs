#![feature(custom_test_frameworks)]
#![test_runner(std_tests::runner)]

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock, mpsc};
use std::thread;
use std::time::{Duration, Instant};

#[test]
fn spawn_join() {
    assert_eq!(thread::spawn(|| 40 + 2).join().unwrap(), 42);
}

#[test]
fn many_threads() {
    let counter = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..32)
        .map(|_| {
            let counter = counter.clone();
            thread::spawn(move || counter.fetch_add(1, Ordering::Relaxed))
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(counter.load(Ordering::Relaxed), 32);
}

#[test]
fn names() {
    let name = thread::Builder::new()
        .name("worker".into())
        .spawn(|| thread::current().name().map(str::to_owned))
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(name.as_deref(), Some("worker"));
}

#[test]
fn stack_size() {
    let sum = thread::Builder::new()
        .stack_size(256 * 1024)
        .spawn(|| {
            let buf = [1u8; 128 * 1024];
            buf.iter().map(|&b| b as u32).sum::<u32>()
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(sum, 128 * 1024);
}

#[test]
fn available_parallelism() {
    assert_eq!(thread::available_parallelism().unwrap().get(), 3);
}

#[test]
fn sleep() {
    let start = Instant::now();
    thread::sleep(Duration::from_millis(50));
    assert!(start.elapsed() >= Duration::from_millis(50));
}

#[test]
fn mutex_condvar() {
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();
    let h = thread::spawn(move || {
        *pair2.0.lock().unwrap() = true;
        pair2.1.notify_one();
    });
    let mut ready = pair.0.lock().unwrap();
    while !*ready {
        ready = pair.1.wait(ready).unwrap();
    }
    drop(ready);
    h.join().unwrap();
}

#[test]
fn condvar_timeout() {
    let m = Mutex::new(());
    let cv = Condvar::new();
    let start = Instant::now();
    let (_g, res) = cv.wait_timeout(m.lock().unwrap(), Duration::from_millis(50)).unwrap();
    assert!(res.timed_out());
    assert!(start.elapsed() >= Duration::from_millis(50));
}

#[test]
fn contended_mutex() {
    let m = Arc::new(Mutex::new(0u32));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let m = m.clone();
            thread::spawn(move || {
                for _ in 0..1000 {
                    *m.lock().unwrap() += 1;
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(*m.lock().unwrap(), 8000);
}

#[test]
fn rwlock_and_barrier() {
    let lock = Arc::new(RwLock::new(0));
    let barrier = Arc::new(Barrier::new(4));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let (lock, barrier) = (lock.clone(), barrier.clone());
            thread::spawn(move || {
                barrier.wait();
                *lock.write().unwrap() += 1;
                *lock.read().unwrap()
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(*lock.read().unwrap(), 4);
}

#[test]
fn channel() {
    let (tx, rx) = mpsc::channel();
    for i in 0..4 {
        let tx = tx.clone();
        thread::spawn(move || tx.send(i).unwrap());
    }
    drop(tx);
    let mut got: Vec<i32> = rx.iter().collect();
    got.sort();
    assert_eq!(got, [0, 1, 2, 3]);
}

#[test]
fn scoped() {
    let mut data = [1, 2, 3];
    thread::scope(|s| {
        for x in &mut data {
            s.spawn(move || *x *= 10);
        }
    });
    assert_eq!(data, [10, 20, 30]);
}

#[test]
fn thread_local_dtor_runs() {
    static DROPS: AtomicUsize = AtomicUsize::new(0);
    struct D;
    impl Drop for D {
        fn drop(&mut self) {
            DROPS.fetch_add(1, Ordering::SeqCst);
        }
    }
    thread_local!(static T: D = const { D });
    thread::spawn(|| T.with(|_| ())).join().unwrap();
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

#[test]
fn thread_local_is_per_thread() {
    thread_local!(static C: Cell<u32> = const { Cell::new(0) });
    C.set(5);
    assert_eq!(thread::spawn(|| C.get()).join().unwrap(), 0);
    assert_eq!(C.get(), 5);
}

#[test]
fn park_unpark() {
    let t = thread::spawn(|| thread::park());
    thread::sleep(Duration::from_millis(10));
    t.thread().unpark();
    t.join().unwrap();
}

#[test]
fn panic_is_caught_by_join() {
    assert!(thread::spawn(|| panic!("expected")).join().is_err());
}
