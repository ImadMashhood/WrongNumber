//! Bounded newline-JSON TCP transport. Socket I/O stays off the rendering thread.
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TryRecvError},
    },
    thread,
    time::{Duration, Instant},
};
pub const PROTOCOL: u32 = 11;
pub const MAX_FRAME: usize = 2 * 1024 * 1024;
pub enum Event<T> {
    Connected(u64),
    Message(u64, T),
    Disconnected(u64, String),
}
pub struct Peer {
    pub sender: SyncSender<serde_json::Value>,
}
pub struct Server<T> {
    pub address: SocketAddr,
    pub events: std::sync::Mutex<Receiver<Event<T>>>,
    new_peers: std::sync::Mutex<Receiver<(u64, Peer)>>,
    pub peers: std::collections::BTreeMap<u64, Peer>,
    stop: Arc<AtomicBool>,
}
pub struct Client<T> {
    pub events: std::sync::Mutex<Receiver<Event<T>>>,
    sender: SyncSender<serde_json::Value>,
    stop: Arc<AtomicBool>,
}
fn write_value<T: Serialize>(value: &T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|e| e.to_string())
}
fn worker<T: DeserializeOwned + Send + 'static>(
    id: u64,
    mut stream: TcpStream,
    out: Receiver<serde_json::Value>,
    events: SyncSender<Event<T>>,
    stop: Arc<AtomicBool>,
    max_frame: usize,
) {
    let _ = stream.set_nonblocking(true);
    let _ = stream.set_nodelay(true);
    let _ = events.send(Event::Connected(id));
    let mut incoming = vec![];
    let mut queue: VecDeque<Vec<u8>> = VecDeque::new();
    let mut offset = 0;
    let mut last_read = Instant::now();
    let mut last_write = Instant::now();
    let mut queued_bytes = 0;
    let mut rate = (Instant::now(), 0);
    let reason = 'connection: loop {
        if stop.load(Ordering::Relaxed) {
            break "Connection stopped".into();
        }
        let mut buffer = [0_u8; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break 'connection "Peer disconnected".into(),
                Ok(n) => {
                    last_read = Instant::now();
                    incoming.extend_from_slice(&buffer[..n]);
                    if incoming.len() > max_frame {
                        break 'connection format!("Frame exceeds {max_frame} bytes");
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => break 'connection e.to_string(),
            }
        }
        while let Some(end) = incoming.iter().position(|b| *b == b'\n') {
            if rate.0.elapsed() >= Duration::from_secs(1) {
                rate = (Instant::now(), 0);
            }
            rate.1 += 1;
            if rate.1 > 120 {
                break 'connection "Message rate exceeded".into();
            }
            let frame: Vec<_> = incoming.drain(..=end).collect();
            match serde_json::from_slice::<T>(&frame[..frame.len() - 1]) {
                Ok(value) => {
                    if events.send(Event::Message(id, value)).is_err() {
                        break 'connection "Application closed".into();
                    }
                }
                Err(e) => break 'connection format!("Invalid packet: {e}"),
            }
        }
        loop {
            match out.try_recv() {
                Ok(v) => {
                    let mut bytes = serde_json::to_vec(&v).unwrap();
                    if bytes.len() > MAX_FRAME {
                        break 'connection "Outgoing frame too large".into();
                    }
                    bytes.push(b'\n');
                    queued_bytes += bytes.len();
                    queue.push_back(bytes);
                    if queued_bytes > MAX_FRAME * 4 {
                        break 'connection "Slow peer".into();
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break 'connection "Application closed".into(),
            }
        }
        if let Some(front) = queue.front() {
            match stream.write(&front[offset..]) {
                Ok(0) => break "Peer write closed".into(),
                Ok(n) => {
                    offset += n;
                    queued_bytes -= n;
                    last_write = Instant::now();
                    if offset == front.len() {
                        queue.pop_front();
                        offset = 0;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => break e.to_string(),
            }
        }
        if last_read.elapsed() > Duration::from_secs(15)
            || (!queue.is_empty() && last_write.elapsed() > Duration::from_secs(10))
        {
            break "Peer timed out".into();
        }
        thread::sleep(Duration::from_millis(2));
    };
    let _ = events.send(Event::Disconnected(id, reason));
}
impl<T: DeserializeOwned + Send + 'static> Server<T> {
    pub fn bind(address: &str) -> Result<Self, String> {
        let listener = TcpListener::bind(address).map_err(|e| e.to_string())?;
        let address = listener.local_addr().unwrap();
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let (tx, events) = mpsc::sync_channel(256);
        let (peer_tx, new_peers) = mpsc::sync_channel(16);
        let stop = Arc::new(AtomicBool::new(false));
        let running = stop.clone();
        let slots = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        thread::spawn(move || {
            let mut next = 1;
            while !running.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        if slots.load(Ordering::Relaxed) >= 12 {
                            drop(stream);
                            continue;
                        }
                        slots.fetch_add(1, Ordering::Relaxed);
                        let (sender, out) = mpsc::sync_channel(16);
                        let id = next;
                        next += 1;
                        if peer_tx.send((id, Peer { sender })).is_err() {
                            break;
                        }
                        let events = tx.clone();
                        let stop = running.clone();
                        let count = slots.clone();
                        thread::spawn(move || {
                            worker(id, stream, out, events, stop, 8192);
                            count.fetch_sub(1, Ordering::Relaxed);
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            address,
            events: std::sync::Mutex::new(events),
            new_peers: std::sync::Mutex::new(new_peers),
            peers: Default::default(),
            stop,
        })
    }
    pub fn send<V: Serialize>(&mut self, id: u64, value: &V) -> Result<(), String> {
        self.collect_peers();
        self.peers
            .get(&id)
            .ok_or("Unknown connection")?
            .sender
            .try_send(write_value(value)?)
            .map_err(|e| e.to_string())
    }
    pub fn collect_peers(&mut self) {
        for (id, p) in self.new_peers.lock().unwrap().try_iter() {
            self.peers.insert(id, p);
        }
    }
    pub fn disconnect(&mut self, id: u64) {
        self.peers.remove(&id);
    }
}
impl<T> Drop for Server<T> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
impl<T: DeserializeOwned + Send + 'static> Client<T> {
    pub fn connect(address: SocketAddr) -> Result<Self, String> {
        let stream = TcpStream::connect_timeout(&address, Duration::from_secs(3))
            .map_err(|e| e.to_string())?;
        let (sender, out) = mpsc::sync_channel(32);
        let (tx, events) = mpsc::sync_channel(128);
        let stop = Arc::new(AtomicBool::new(false));
        let running = stop.clone();
        thread::spawn(move || worker(0, stream, out, tx, running, MAX_FRAME));
        Ok(Self {
            events: std::sync::Mutex::new(events),
            sender,
            stop,
        })
    }
    pub fn send<V: Serialize>(&self, value: &V) -> Result<(), String> {
        self.sender
            .try_send(write_value(value)?)
            .map_err(|e| e.to_string())
    }
}
impl<T> Drop for Client<T> {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}
