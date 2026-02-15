//! This crate is responsible for getting data from the actual RingEdge 2 Maimai Touchscreen COM and
//! wrapping it in the way that Maimai DX (based on ALLs system) can read it.
//!
//! Since PreDX cabinet touch lacks some Touch areas that Deluxe touch has, we basically map them to
//! existing ones in [`finale`] module
//! So if you press, for example, B1 area in Maimai DX, it will also press E1 and E2 (which is close to B1)

use crate::config;
use crate::error::Result;
use crate::state::SharedState;
use crate::touch::deluxe::*;
use crate::touch::finale::*;
use log::info;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread::JoinHandle;

mod deluxe;
mod finale;
pub(crate) mod packet;

pub const HALT: &[u8] = "{HALT}".as_bytes();
pub const STAT: &[u8] = "{STAT}".as_bytes();

pub fn setup(
    config: &config::Touch,
    exit_sig: Arc<AtomicBool>,
    _shared_state: Option<SharedState>,
) -> Result<Vec<JoinHandle<Result<()>>>> {
    info!("Initializing Touchscreen");

    let dx_p1 = Deluxe::new(&config.dx_p1_port, 1).ok();
    let dx_p2 = Deluxe::new(&config.dx_p2_port, 2).ok();

    let mut finale = Finale::new(
        config.clone(),
        dx_p1.as_ref().and_then(|x| x.try_clone().ok()),
        dx_p2.as_ref().and_then(|x| x.try_clone().ok()),
    )?;

    finale.try_init(config.init_retry_count.unwrap_or(i64::MAX))?;

    let mut handles = Vec::new();

    handles.push(Finale::spawn_thread(finale, exit_sig.clone())?);

    if let Some(dx) = dx_p1 {
        handles.push(Deluxe::spawn_thread(dx, exit_sig.clone())?);
    }
    if let Some(dx) = dx_p2 {
        handles.push(Deluxe::spawn_thread(dx, exit_sig.clone())?);
    }

    info!("Touchscreen is ready. Good luck touchin'");

    Ok(handles)
}
