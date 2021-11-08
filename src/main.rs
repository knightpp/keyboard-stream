use anyhow::{Context, Result};
use argh::FromArgs;
use evdev_rs::{
    enums::{EventCode, EV_KEY},
    InputEvent, ReadFlag, TimeVal,
};
use keyboard_stream::{Event, NopRemote, Remote, TcpRemote, UdpRemote};
use std::{net::UdpSocket, str::FromStr};
/// Keyboard server
#[derive(Debug, FromArgs)]
struct Args {
    #[argh(positional)]
    device_path: String,
    /// addr of the remote to which send keys
    #[argh(option)]
    remote_addr: Option<String>,
    /// connection type, defaults to 'tcp'
    #[argh(option, default = "Default::default()", short = 'c')]
    conn: ConnectionType,
}

#[derive(Debug, Clone, Copy)]
enum ConnectionType {
    Nop,
    Udp,
    Tcp,
}

impl FromStr for ConnectionType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "nop" => Ok(ConnectionType::Nop),
            "udp" => Ok(ConnectionType::Udp),
            "tcp" => Ok(ConnectionType::Tcp),
            _ => Err(anyhow::anyhow!("unknown type: {}", s)),
        }
    }
}
impl Default for ConnectionType {
    fn default() -> Self {
        ConnectionType::Tcp
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();

    sudo::escalate_if_needed()?;

    let server_addr = "127.0.0.1:0";
    let remote: Box<dyn Remote> = match args.conn {
        ConnectionType::Nop => Box::new(NopRemote::new()),
        ConnectionType::Udp => {
            let socket = UdpSocket::bind(server_addr).context("socket bind failed")?;
            if let Some(addr) = &args.remote_addr {
                socket.connect(addr).context("socket connect failed")?;
            }
            println!("UDP binded on {}", socket.local_addr()?);
            Box::new(UdpRemote::new(socket))
        }
        ConnectionType::Tcp => {
            if let Some(remote_addr) = &args.remote_addr {
                Box::new(TcpRemote::connect(remote_addr)?)
            } else {
                Box::new(TcpRemote::wait_for_client(server_addr)?)
            }
        }
    };
    println!("Starting with connection {:?}", args.conn);
    if args.remote_addr.is_none() {
        run_sender(&args.device_path, remote)
    } else {
        run_receiver(&args.device_path, remote)
    }
    .context("run failed")?;
    Ok(())
}

fn run_sender(device_path: &str, mut remote: Box<dyn Remote>) -> Result<()> {
    let mut device = evdev_rs::Device::new_from_file(
        std::fs::File::open(device_path).context("couldn't open device file")?,
    )
    .context("couldn't create a device from file")?;
    device
        .grab(evdev_rs::GrabMode::Grab)
        .context("grab failed")?;
    loop {
        let ev = device
            .next_event(ReadFlag::NORMAL | ReadFlag::BLOCKING)
            .map(|val| val.1);
        match ev {
            Ok(ev) => {
                if let EventCode::EV_KEY(key) = ev.event_code {
                    if key == EV_KEY::KEY_ESC {
                        device
                            .grab(evdev_rs::GrabMode::Ungrab)
                            .context("ungrab failed")?;
                        break;
                    }
                }
                remote
                    .send(Event {
                        code: ev.event_code,
                        value: ev.value,
                    })
                    .context("sending event failed")?;
                println!("{:?}", ev);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    Ok(())
}

fn run_receiver(device_path: &str, mut remote: Box<dyn Remote>) -> Result<()> {
    let file = std::fs::File::open(device_path).context("opening file failed")?;
    let device = evdev_rs::Device::new_from_file(file).context("opening device failed")?;
    let input = evdev_rs::UInputDevice::create_from_device(&device)
        .context("creating uinput device failed")?;

    loop {
        let event = remote.receive().context("receive from remote failed")?;
        println!("Received event: {:?}", event);
        input
            .write_event(&InputEvent {
                event_code: event.code,
                value: event.value,
                time: TimeVal::new(0, 0),
            })
            .context("write_event failed")?;
    }
}
