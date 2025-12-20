//! This crate is responsible for getting data from the actual RingEdge 2 Maimai Touchscreen COM and
//! wrapping it in the way that Maimai DX (based on ALLs system) can read it.
//!
//! Since PreDX cabinet touch lacks some Touch areas that Deluxe touch has, we basically map them to
//! existing ones in [`finale`] module
//! So if you press, for example, B1 area in Maimai DX, it will also press E1 and E2 (which is close to B1)

use crate::config;
use crate::error::Result;
use crate::touch::deluxe::*;
use crate::touch::finale::*;
use log::info;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

mod deluxe;
mod finale;
pub(crate) mod packet;

// pub const RSET: &[u8] = "{RSET}".as_bytes();
pub const HALT: &[u8] = "{HALT}".as_bytes();
pub const STAT: &[u8] = "{STAT}".as_bytes();

pub fn setup(
    config: &config::Touch,
    handles: &mut Vec<JoinHandle<Result<()>>>,
    exit_sig: Arc<AtomicBool>,
) -> Result<()> {
    info!("Initializing Touchscreen");

    let dx_p1 = Deluxe::new(&config.dx_p1_port, 1).ok();
    let dx_p2 = Deluxe::new(&config.dx_p2_port, 2).ok();

    let mut finale = Finale::new(
        &config.finale_port,
        &config.p1_threshold,
        &config.p2_threshold,
        dx_p1.as_ref().and_then(|x| x.try_clone().ok()),
        dx_p2.as_ref().and_then(|x| x.try_clone().ok()),
    )?;

    finale.init()?;

    let finale_thread = Finale::spawn_thread(finale, exit_sig.clone())?;

    let dx_p1_thread = dx_p1
        .and_then(|x| Some(Deluxe::spawn_thread(x, exit_sig.clone())))
        .transpose()?;
    let dx_p2_thread = dx_p2
        .and_then(|x| Some(Deluxe::spawn_thread(x, exit_sig.clone())))
        .transpose()?;

    handles.push(finale_thread);
    dx_p1_thread.map(|t| handles.push(t));
    dx_p2_thread.map(|t| handles.push(t));

    info!("Touchscreen is ready. Good luck touchin'");

    Ok(())
}
