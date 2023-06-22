// This crate is responsible for getting data from the actual RingEdge 2 Maimai Touchscreen COM and
// wrapping it in the way that Maimai DX (based on ALLs system) can read it.
//
// Since Finale cabinet touch lacks some Touch areas that Deluxe touch has, we basically map them to
// existing ones (see touch::deluxe::)
// So if you press, for example, B1 area in Maimai DX, it will also press E1 and E2 (which is is close to B1)

use crate::config::Settings;
use log::info;

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{io, thread};

use crate::touch::deluxe::*;
use crate::touch::finale::*;

mod deluxe;
mod finale;

// pub const RSET: &[u8] = "{RSET}".as_bytes();
pub const HALT: &[u8] = "{HALT}".as_bytes();
pub const STAT: &[u8] = "{STAT}".as_bytes();

pub fn spawn_thread(
    args: &Settings,
    exit_sig: &Arc<AtomicBool>,
) -> io::Result<(JoinHandle<io::Result<()>>, JoinHandle<io::Result<()>>)> {
    let (sender, receiver) = crossbeam_channel::bounded::<MessageCmd>(10);

    let mut dx_p1_touch = Deluxe::new(args.touch_alls_p1_com.clone(), 0, sender.clone())?;
    let mut dx_p2_touch = Deluxe::new(args.touch_alls_p2_com.clone(), 1, sender.clone())?;

    let dx_p1_port = dx_p1_touch.port.try_clone_native()?;
    let dx_p2_port = dx_p2_touch.port.try_clone_native()?;

    let mut fe_touch = RingEdge2::new(args.touch_re2_com.clone(), dx_p1_port, dx_p2_port)?;
    fe_touch.port.write_all(HALT)?;
    fe_touch.port.write_all(STAT)?;
    let dx_sig = exit_sig.clone();
    let fe_sig = exit_sig.clone();

    let deluxe_handle = thread::Builder::new()
        .name("Deluxe Touch Thread".to_string())
        .spawn(move || -> io::Result<()> {
            while dx_sig.load(Ordering::Acquire) {
                dx_p1_touch.read();
                dx_p2_touch.read();
            }
            Ok(())
        })
        .unwrap();
    let finale_handle = thread::Builder::new()
        .name("Finale Touch Thread".to_string())
        .spawn(move || -> io::Result<()> {
            let rcv = receiver.clone();
            while fe_sig.load(Ordering::Acquire) {
                for c in rcv.try_iter() {
                    fe_touch.parse_command_from_alls(c)?;
                }
                fe_touch.read();
            }

            fe_touch.port.write_all(HALT)?;

            Ok(())
        })
        .unwrap();

    info!("Touchscreen is ready, good luck touchin'!");
    info!("If touchscreen doesn't work, restart the application, go in test menu and exit it so checks run again");

    Ok((finale_handle, deluxe_handle))
}
