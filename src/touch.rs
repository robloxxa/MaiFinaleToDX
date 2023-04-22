// This crate is responsible for getting data from the actual RingEdge 2 Maimai Touchscreen COM and
// wrapping it in the way that Maimai DX (based on ALLs system) can read it.
//
// Since RingEdge 2 touch lacks some Touch areas that ALLs touch has, we basically map them to
// existing ones (see alls_touch_areas crate)
// So if you press, for example, B1 area in Maimai DX, it will also press E1 and E2 (which is is close to B1)

use crate::config::Settings;
use log::info;
use serialport::ClearBuffer;
use std::io::Write;
use std::thread::JoinHandle;
use std::{io, thread};

use crate::touch::alls::*;
use crate::touch::ringedge2::*;

mod alls;
mod ringedge2;

pub const RSET: &[u8] = "{RSET}".as_bytes();
pub const HALT: &[u8] = "{HALT}".as_bytes();
pub const STAT: &[u8] = "{STAT}".as_bytes();

pub struct AllsMessageCmd {
    player_num: usize,
    cmd: AllsTouchMasterCommand,
}

pub fn spawn_thread(
    args: &Settings,
    done_recv: crossbeam_channel::Receiver<()>,
) -> io::Result<(JoinHandle<io::Result<()>>, JoinHandle<io::Result<()>>)> {
    let (sender, receiver) = crossbeam_channel::bounded::<AllsMessageCmd>(10);

    let mut alls_p1_touch = Alls::new(args.touch_alls_p1_com.clone(), 0, sender.clone())?;
    let mut alls_p2_touch = Alls::new(args.touch_alls_p2_com.clone(), 1, sender.clone())?;

    let alls_p1_port = alls_p1_touch.port.try_clone_native()?;
    let alls_p2_port = alls_p2_touch.port.try_clone_native()?;

    let mut re2_touch = RingEdge2::new(args.touch_re2_com.clone(), alls_p1_port, alls_p2_port)?;
    re2_touch.port.write(HALT)?;
    re2_touch.port.write(STAT)?;

    let alls_handle = thread::spawn(move || loop {
        alls_p1_touch.read();
        alls_p2_touch.read();
    });

    let re2_handle = thread::spawn(move || -> io::Result<()> {
        let rcv = receiver.clone();

        loop {
            if let Err(err) = done_recv.try_recv() {
                break;
            }
            rcv.try_iter().for_each(|c| {
                re2_touch.parse_command_from_alls(c);
            });
            re2_touch.read();
        }

        re2_touch.port.write("{HALT}".as_bytes())?;

        Ok(())
    });

    info!("Touchscreen is enabled, good luck touchin'!");
    info!("If touchscreen doesn't work, restart the application, go in test menu and exit it so checks run again");

    Ok((re2_handle, alls_handle))
}
