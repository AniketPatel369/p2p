use discovery::{Announcement, DiscoveryService, PeerRegistry};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant};

fn sample_announcement(port: u16) -> Announcement {
    Announcement {
        device_id: "device-123".to_string(),
        public_key_b64: "PUBKEYBASE64".to_string(),
        display_name: "Alice Laptop".to_string(),
        port,
    }
}

#[test]
fn announcement_round_trip_encode_decode() {
    let a = sample_announcement(5000);
    let b = Announcement::decode(&a.encode()).expect("decode works");
    assert_eq!(a, b);
}

#[test]
fn invalid_packet_is_rejected() {
    let bad = b"NOT_DISCOVERY";
    assert!(Announcement::decode(bad).is_err());
}

#[test]
fn peer_registry_expires_stale_entries() {
    let mut registry = PeerRegistry::new(Duration::from_secs(1));
    let src: SocketAddr = "127.0.0.1:12345".parse().expect("socket addr");
    let now = Instant::now();
    registry.upsert(sample_announcement(9999), src, now);
    assert_eq!(registry.len(), 1);

    registry.expire(now + Duration::from_secs(2));
    assert_eq!(registry.len(), 0);
}

#[test]
fn local_announce_discover_cycle_over_udp() {
    let receiver = DiscoveryService::bind("127.0.0.1:0".parse().expect("bind recv")).expect("receiver bind");
    let recv_addr = receiver.local_addr().expect("local addr");

    let handle = thread::spawn(move || {
        let (announcement, _src) = receiver.recv_announcement(2048).expect("recv announcement");
        announcement
    });

    // Sender uses raw socket to simulate another peer process.
    let sender = UdpSocket::bind("127.0.0.1:0").expect("bind sender");
    let sent = sender
        .send_to(&sample_announcement(7777).encode(), recv_addr)
        .expect("send announcement");
    assert!(sent > 0);

    let received = handle.join().expect("thread join");
    assert_eq!(received.device_id, "device-123");
    assert_eq!(received.display_name, "Alice Laptop");
    assert_eq!(received.port, 7777);
}
