use anyhow::Result;
use std::{
    io::{Read, Write},
    net::ToSocketAddrs,
};

pub trait Remote {
    fn send(&mut self, event: Event) -> Result<()>;
    fn receive(&mut self) -> Result<Event>;
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub code: evdev_rs::enums::EventCode,
    pub value: i32,
}

pub struct NopRemote {}
impl NopRemote {
    pub fn new() -> Self {
        Self {}
    }
}
impl Default for NopRemote {
    fn default() -> Self {
        Self {}
    }
}
impl Remote for NopRemote {
    fn send(&mut self, _: Event) -> Result<()> {
        Ok(())
    }

    fn receive(&mut self) -> Result<Event> {
        std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
        panic!()
    }
}
pub struct UdpRemote {
    socket: std::net::UdpSocket,
}

impl UdpRemote {
    pub fn new(socket: std::net::UdpSocket) -> Self {
        Self { socket }
    }
}

impl Remote for UdpRemote {
    fn send(&mut self, event: Event) -> Result<()> {
        let bytes = serde_json::to_vec(&event)?;
        self.socket.send(&bytes)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<Event> {
        let mut buf = vec![0u8; 64];
        let n = self.socket.recv(&mut buf)?;
        let buf = &buf[..n];
        Ok(serde_json::from_slice(buf)?)
    }
}

pub struct TcpRemote {
    stream: std::net::TcpStream,
}

impl TcpRemote {
    pub fn wait_for_client<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let listener = std::net::TcpListener::bind(addr)?;
        println!("TCP server is listening on {}", listener.local_addr()?);
        let peer = listener.accept()?;
        Ok(Self { stream: peer.0 })
    }

    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = std::net::TcpStream::connect(addr)?;
        Ok(Self { stream })
    }
}

impl Remote for TcpRemote {
    fn send(&mut self, event: Event) -> Result<()> {
        let bytes = serde_json::to_vec(&event)?;
        self.stream.write_all(&bytes)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<Event> {
        let mut buf = vec![0u8; 64];
        let n = self.stream.read(&mut buf)?;
        let buf = &buf[..n];
        Ok(serde_json::from_slice(buf)?)
    }
}
