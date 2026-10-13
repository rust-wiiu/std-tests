#![feature(custom_test_frameworks)]
#![test_runner(std_tests::runner)]

use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

/// Fresh directory per test, since libtest runs tests in parallel.
fn tmpdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).unwrap();
    dir
}

#[test]
fn write_read_roundtrip() {
    let dir = tmpdir("roundtrip");
    // Sizes around the 64-byte alignment and above the 128 KiB bounce buffer.
    for len in [0, 1, 63, 64, 65, 4096, 200_000] {
        let path = dir.join(format!("f{len}"));
        let data: Vec<u8> = (0..len).map(|i| i as u8).collect();
        fs::write(&path, &data).unwrap();
        assert_eq!(fs::read(&path).unwrap(), data, "len {len}");
        assert_eq!(fs::metadata(&path).unwrap().len(), len as u64);
    }
}

#[test]
fn append() {
    let path = tmpdir("append").join("f");
    fs::write(&path, b"abc").unwrap();
    let mut f = OpenOptions::new().append(true).open(&path).unwrap();
    f.write_all(b"def").unwrap();
    drop(f);
    assert_eq!(fs::read(&path).unwrap(), b"abcdef");
}

#[test]
fn create_new() {
    let path = tmpdir("create_new").join("f");
    File::create_new(&path).unwrap();
    let err = File::create_new(&path).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::AlreadyExists);
}

#[test]
fn seek_and_overwrite() {
    let path = tmpdir("seek").join("f");
    fs::write(&path, b"0123456789").unwrap();
    let mut f = OpenOptions::new().read(true).write(true).open(&path).unwrap();
    assert_eq!(f.seek(SeekFrom::Start(3)).unwrap(), 3);
    f.write_all(b"xy").unwrap();
    assert_eq!(f.stream_position().unwrap(), 5);
    assert_eq!(f.seek(SeekFrom::End(-2)).unwrap(), 8);
    let mut tail = String::new();
    f.read_to_string(&mut tail).unwrap();
    assert_eq!(tail, "89");
    drop(f);
    assert_eq!(fs::read(&path).unwrap(), b"012xy56789");
}

#[test]
fn set_len() {
    let path = tmpdir("set_len").join("f");
    fs::write(&path, b"0123456789").unwrap();
    // The Cafe FS refuses to open a file twice, so each check reopens it.
    let f = OpenOptions::new().write(true).open(&path).unwrap();
    f.set_len(4).unwrap();
    drop(f);
    assert_eq!(fs::read(&path).unwrap(), b"0123");
    let f = OpenOptions::new().write(true).open(&path).unwrap();
    f.set_len(6).unwrap();
    drop(f);
    assert_eq!(fs::read(&path).unwrap(), b"0123\0\0");
}

#[test]
fn truncate_on_open() {
    let path = tmpdir("truncate").join("f");
    fs::write(&path, b"long contents").unwrap();
    fs::write(&path, b"short").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"short");
}

#[test]
fn metadata() {
    let dir = tmpdir("metadata");
    let path = dir.join("f");
    fs::write(&path, b"abc").unwrap();
    let meta = fs::metadata(&path).unwrap();
    assert!(meta.is_file());
    assert!(!meta.is_dir());
    assert!(!meta.file_type().is_symlink());
    assert!(fs::metadata(&dir).unwrap().is_dir());
    let _ = meta.modified();
}

#[test]
fn read_dir() {
    let dir = tmpdir("read_dir");
    fs::write(dir.join("a"), b"").unwrap();
    fs::write(dir.join("b"), b"").unwrap();
    fs::create_dir(dir.join("sub")).unwrap();
    let mut names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["a", "b", "sub"]);
}

#[test]
fn rename() {
    let dir = tmpdir("rename");
    fs::write(dir.join("old"), b"x").unwrap();
    fs::rename(dir.join("old"), dir.join("new")).unwrap();
    assert!(!dir.join("old").exists());
    assert_eq!(fs::read(dir.join("new")).unwrap(), b"x");
}

#[test]
fn remove_errors() {
    let dir = tmpdir("remove_errors");
    fs::create_dir(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/f"), b"").unwrap();
    assert_eq!(fs::remove_file(dir.join("sub")).unwrap_err().kind(), ErrorKind::IsADirectory);
    assert_eq!(
        fs::remove_dir(dir.join("sub/f")).unwrap_err().kind(),
        ErrorKind::NotADirectory
    );
    assert_eq!(
        fs::remove_dir(dir.join("sub")).unwrap_err().kind(),
        ErrorKind::DirectoryNotEmpty
    );
    fs::remove_dir_all(dir.join("sub")).unwrap();
    assert!(!dir.join("sub").exists());
}

#[test]
fn not_found() {
    let err = fs::read(std::env::temp_dir().join("does/not/exist")).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
}

#[test]
fn current_dir() {
    // Not restored: the initial directory is `/`, which the FS cannot stat.
    let dir = tmpdir("current_dir");
    fs::create_dir(dir.join("sub")).unwrap();
    std::env::set_current_dir(&dir).unwrap();
    assert_eq!(std::env::current_dir().unwrap(), dir);
    std::env::set_current_dir("sub").unwrap();
    assert_eq!(std::env::current_dir().unwrap(), dir.join("sub"));
}
