#![feature(custom_test_frameworks, tcp_linger)]
#![test_runner(std_tests::runner)]

use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, Shutdown, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::thread;
use std::time::Duration;

const LOCALHOST: Ipv4Addr = Ipv4Addr::LOCALHOST;

#[test]
fn tcp_loopback() {
    let listener = TcpListener::bind((LOCALHOST, 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    assert_ne!(addr.port(), 0);
    let client = thread::spawn(move || {
        let mut s = TcpStream::connect(addr).unwrap();
        s.write_all(b"ping").unwrap();
        let mut buf = [0; 4];
        s.read_exact(&mut buf).unwrap();
        buf
    });
    let (mut s, peer) = listener.accept().unwrap();
    assert_eq!(peer.ip(), LOCALHOST);
    let mut buf = [0; 4];
    assert_eq!(s.peek(&mut buf).unwrap(), 4);
    s.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"ping");
    s.write_all(b"pong").unwrap();
    assert_eq!(&client.join().unwrap(), b"pong");
}

#[test]
fn tcp_shutdown_gives_eof() {
    let listener = TcpListener::bind((LOCALHOST, 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let client = thread::spawn(move || {
        let s = TcpStream::connect(addr).unwrap();
        s.shutdown(Shutdown::Write).unwrap();
    });
    let (mut s, _) = listener.accept().unwrap();
    let mut buf = Vec::new();
    assert_eq!(s.read_to_end(&mut buf).unwrap(), 0);
    client.join().unwrap();
}

#[test]
fn tcp_options() {
    let listener = TcpListener::bind((LOCALHOST, 0)).unwrap();
    let s = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    s.set_nodelay(true).unwrap();
    assert!(s.nodelay().unwrap());
    s.set_ttl(32).unwrap();
    assert_eq!(s.ttl().unwrap(), 32);
    s.set_linger(Some(Duration::from_secs(1))).unwrap();
    assert_eq!(s.linger().unwrap(), Some(Duration::from_secs(1)));
    assert!(s.take_error().unwrap().is_none());
}

#[test]
fn tcp_connect_refused() {
    let port = TcpListener::bind((LOCALHOST, 0)).unwrap().local_addr().unwrap().port();
    let err = TcpStream::connect((LOCALHOST, port)).unwrap_err();
    // nsysnet does not reset loopback connections to closed ports; the
    // connect times out after about 20 seconds instead.
    assert!(
        matches!(err.kind(), ErrorKind::ConnectionRefused | ErrorKind::TimedOut),
        "{err:?}"
    );
}

#[test]
fn tcp_nonblocking_accept() {
    let listener = TcpListener::bind((LOCALHOST, 0)).unwrap();
    listener.set_nonblocking(true).unwrap();
    assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
}

#[test]
fn udp_loopback() {
    let a = UdpSocket::bind((LOCALHOST, 0)).unwrap();
    let b = UdpSocket::bind((LOCALHOST, 0)).unwrap();
    a.send_to(b"hello", b.local_addr().unwrap()).unwrap();
    let mut buf = [0; 16];
    let (n, from) = b.recv_from(&mut buf).unwrap();
    assert_eq!(&buf[..n], b"hello");
    assert_eq!(from, a.local_addr().unwrap());
}

#[test]
fn udp_connected() {
    let a = UdpSocket::bind((LOCALHOST, 0)).unwrap();
    let b = UdpSocket::bind((LOCALHOST, 0)).unwrap();
    a.connect(b.local_addr().unwrap()).unwrap();
    a.send(b"x").unwrap();
    let mut buf = [0; 4];
    assert_eq!(b.recv(&mut buf).unwrap(), 1);
    assert_eq!(a.peer_addr().unwrap(), b.local_addr().unwrap());
}

#[test]
fn udp_multicast_options() {
    let s = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    s.set_broadcast(true).unwrap();
    assert!(s.broadcast().unwrap());
    s.set_multicast_ttl_v4(4).unwrap();
    assert_eq!(s.multicast_ttl_v4().unwrap(), 4);
    s.set_multicast_loop_v4(false).unwrap();
    assert!(!s.multicast_loop_v4().unwrap());
    let group = Ipv4Addr::new(224, 0, 0, 251);
    s.join_multicast_v4(&group, &Ipv4Addr::UNSPECIFIED).unwrap();
    s.leave_multicast_v4(&group, &Ipv4Addr::UNSPECIFIED).unwrap();
}

#[test]
fn ipv6_is_unsupported() {
    assert_eq!(UdpSocket::bind("[::1]:0").unwrap_err().kind(), ErrorKind::Unsupported);
}

#[test]
fn resolve_localhost() {
    let addrs: Vec<_> = ("localhost", 80).to_socket_addrs().unwrap().collect();
    assert!(addrs.iter().any(|a| a.ip() == LOCALHOST), "{addrs:?}");
}
