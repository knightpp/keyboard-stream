use anyhow::{Context, Result};
use argh::FromArgs;
use evdev_rs::{
    enums::{EventCode, EV_KEY},
    ReadFlag,
};
use keyboard_stream::{Event, Remote, UdpRemote};
use std::net::UdpSocket;

/// Keyboard server
#[derive(Debug, FromArgs)]
struct Args {
    #[argh(positional)]
    device_path: String,
    /// addr of the remote to which send keys
    #[argh(option)]
    remote_addr: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = argh::from_env();
    sudo::escalate_if_needed()?;

    let socket = UdpSocket::bind("127.0.0.1:3400").context("socket bind failed")?;
    socket
        .connect(&args.remote_addr)
        .context("socket connect failed")?;
    let remote = UdpRemote::new(socket);
    run(&args.device_path, remote).context("run failed")?;
    Ok(())
}

fn run<R: Remote>(device_path: &str, remote: R) -> Result<()> {
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
