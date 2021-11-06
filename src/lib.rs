use anyhow::Result;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub code: evdev_rs::enums::EventCode,
    pub value: i32,
}

pub trait Remote {
    fn send(&self, event: Event) -> Result<()>;
    fn receive(&self) -> Result<Event>;
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
    fn send(&self, _: Event) -> Result<()> {
        Ok(())
    }

    fn receive(&self) -> Result<Event> {
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
    fn send(&self, event: Event) -> Result<()> {
        let bytes = serde_json::to_vec(&event)?;
        self.socket.send(&bytes)?;
        Ok(())
    }

    fn receive(&self) -> Result<Event> {
        let mut buf = vec![0u8; 64];
        let n = self.socket.recv(&mut buf)?;
        let buf = &buf[..n];
        Ok(serde_json::from_slice(buf)?)
    }
}
