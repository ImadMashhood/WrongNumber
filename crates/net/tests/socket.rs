use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    time::{Duration, Instant},
};
use wrong_number_net::{Client, Event, Server};
#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Message {
    Hello(u32),
    Ack(u32),
}
#[test]
fn real_socket_preserves_frame_order_and_handles_fragmented_packets() {
    let mut server = Server::<Message>::bind("127.0.0.1:0").unwrap();
    let client = Client::<Message>::connect(server.address).unwrap();
    client.send(&Message::Hello(1)).unwrap();
    client.send(&Message::Hello(2)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut got = vec![];
    while got.len() < 2 && Instant::now() < deadline {
        server.collect_peers();
        let events: Vec<_> = server.events.lock().unwrap().try_iter().collect();
        for event in events {
            if let Event::Message(id, Message::Hello(n)) = event {
                got.push(n);
                server.send(id, &Message::Ack(n)).unwrap();
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(got, vec![1, 2]);
    let mut acks = vec![];
    while acks.len() < 2 && Instant::now() < deadline {
        for event in client.events.lock().unwrap().try_iter() {
            if let Event::Message(_, Message::Ack(n)) = event {
                acks.push(n);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(acks, vec![1, 2]);
    let mut raw = std::net::TcpStream::connect(server.address).unwrap();
    raw.write_all(b"{\"Hello\":").unwrap();
    std::thread::sleep(Duration::from_millis(20));
    raw.write_all(b"3}\n").unwrap();
    let mut fragmented = false;
    while !fragmented && Instant::now() < deadline {
        for e in server.events.lock().unwrap().try_iter() {
            if matches!(e, Event::Message(_, Message::Hello(3))) {
                fragmented = true;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(fragmented);
}
#[test]
fn malformed_peer_is_disconnected_without_corrupting_other_connections() {
    let server = Server::<Message>::bind("127.0.0.1:0").unwrap();
    let mut raw = std::net::TcpStream::connect(server.address).unwrap();
    raw.write_all(b"{invalid}\n").unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut rejected = false;
    while !rejected && Instant::now() < deadline {
        for e in server.events.lock().unwrap().try_iter() {
            if let Event::Disconnected(_, reason) = e {
                rejected = reason.contains("Invalid packet");
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert!(rejected);
}
