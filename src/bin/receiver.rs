use anyhow::{Context, Result};
use argh::FromArgs;
use evdev_rs::{InputEvent, TimeVal};
use keyboard_stream::{Remote, UdpRemote};
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

    let socket = UdpSocket::bind("127.0.0.1:3401").context("bind to socket failed")?;
    socket
        .connect(&args.remote_addr)
        .context("connect to socket failed")?;
    let remote = UdpRemote::new(socket);
    run(&args.device_path, remote).context("run failed")?;
    Ok(())
}

fn run<R: Remote>(device_path: &str, remote: R) -> Result<()> {
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
