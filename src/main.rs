use ::clap;
use clap::Parser;
use flexi_logger::{colored_opt_format, opt_format, Logger};
use log::{debug, info, warn};
use std::time::Instant;

mod config;
mod helper_funcs;
mod jvs;
mod touch;

#[derive(Parser, Debug)]
#[command(author = "robloxxa", version, about, long_about = None)]
/// Tool that allow playing Maimai DX on original Maimai Finale Cabinet
pub struct Arguments {
    /// Log level, options: INFO, WARN,
    #[arg(long, default_value = "info")]
    pub log_level: String,

    /// When set to True (or presented) will disable touch features
    #[arg(long, default_value = "false")]
    pub disable_touch: bool,

    /// When set to True (or presented) will disable JVS features
    #[arg(long, default_value = "false")]
    pub disable_jvs: bool,

    /// COM Port for Finale touch
    #[arg(long, default_value = "COM9")]
    pub touch_re2_com: String,

    /// COM Port for Deluxe Player 1 touch
    #[arg(long, default_value = "COM6")]
    pub touch_alls_p1_com: String,

    /// COM Port for Deluxe Player 2 touch
    #[arg(long, default_value = "COM7")]
    pub touch_alls_p2_com: String,

    /// COM Port for Finale's JVS
    #[arg(long, default_value = "COM24")]
    pub jvs_re2_com: String,
}

fn main() {
    let args = Arguments::parse();
    Logger::try_with_str(args.log_level.clone())
        .unwrap()
        .format(colored_opt_format)
        .start()
        .unwrap();
    if !args.disable_touch {
        let (re2, alls) = touch::spawn_thread(&args);
        re2.join().unwrap();
        alls.join().unwrap();
    } else {
        warn!("\"disable_touch\" was set to True. Touch features disabled")
    }

    if !args.disable_jvs {
    } else {
        warn!("\"disable_jvs\" was set to True. JVS features disabled")
    }
}
