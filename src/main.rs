use std::io::{BufReader, Read};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use ::clap;
use ::crossbeam_channel;
use ::serialport;
use clap::Parser;

mod helper_funcs;
mod jvs;
mod reader_proxy;
mod touch;

#[derive(Parser, Debug)]
#[command(author = "robloxxa", version, about, long_about = None)]
/// Tool that allow playing Maimai DX on original Maimai Finale Cabinet
pub struct Arguments {
    /// When set to True (or presented) will disable touch features
    #[arg(long, default_value = "false")]
    touch_disabled: bool,

    /// When set to True (or presented) will disable JVS features
    #[arg(long, default_value = "false")]
    jvs_disabled: bool,

    /// When set to True (or presented) will disable Proxying
    #[arg(long, default_value = "false")]
    reader_proxy_disabled: bool,

    /// COM Port for Finale touch
    #[arg(long, default_value = "COM9")]
    re2_touch_com: String,

    /// COM Port for Deluxe Player 1 touch
    #[arg(long, default_value = "COM6")]
    alls_p1_touch_com: String,

    /// COM Port for Deluxe Player 2 touch
    #[arg(long, default_value = "COM7")]
    alls_p2_touch_com: String,

    /// COM Port for Finale's JVS
    #[arg(long, default_value = "COM24")]
    re2_jvs_com: String,
}

fn main() {
    let args = Arguments::parse();
    if !args.touch_disabled {
        let (re2, alls) = touch::spawn_thread(&args);
        re2.join().unwrap();
        alls.join().unwrap();
    }
}

fn spawn_jvs_thread() {
    todo!()
}
