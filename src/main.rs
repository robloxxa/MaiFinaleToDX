use crate::config::{Cli, Config};
use crate::error::Result;
use crate::runtime::ModuleRuntime;
use crate::state::SharedState;
use clap::Parser;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use anyhow::Context;
use std::sync::atomic::Ordering;
use winapi::um::timeapi;

mod config;
mod error;
mod helper_funcs;
mod keyboard;
mod port;
mod runtime;
mod state;

#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "touch")]
mod touch;

#[cfg(feature = "jvs")]
mod jvs;

#[cfg(feature = "reader")]
mod card_reader;

fn main() {
    if let Err(e) = setup() {
        error!("Error: {}", e);
    }

    #[cfg(not(feature = "gui"))]
    {
        use std::process::Command;
        let _ = Command::new("cmd.exe").arg("/c").arg("pause").status();
    }
}

fn setup() -> Result<()> {
    let mut cli = Cli::parse();

    let log_level = cli.log_level.take().unwrap_or_else(|| "info".to_string());
    let filter = EnvFilter::try_new(&log_level)
        .with_context(|| format!("Invalid log level: {}", log_level))?;

    let (log_to_file, _guard) = if cli.log_to_file {
        let file_appender = tracing_appender::rolling::daily("./logs", "log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        (
            Some(fmt::layer().with_ansi(false).with_writer(non_blocking)),
            Some(guard),
        )
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_ansi(true)
                .with_thread_names(true)
                .pretty(),
        )
        .with(log_to_file)
        .init();

    // SAFETY: Sets Windows timer resolution to 1ms for better COM port timing
    unsafe {
        let result = timeapi::timeBeginPeriod(1);
        if result != 0 {
            warn!("Failed to set timer resolution to 1ms (error code: {}). This may affect COM port performance.", result);
        }
    }

    let config = Config::init(&cli)?;

    let shared_state = SharedState::new();
    let mut runtime = ModuleRuntime::new(config, shared_state);
    runtime.start_all();

    #[cfg(feature = "gui")]
    if !cli.no_gui {
        return gui::run(runtime);
    }

    run_cli(runtime)
}

fn run_cli(runtime: ModuleRuntime) -> Result<()> {
    let exit_signals = runtime.exit_signals();

    ctrlc::set_handler(move || {
        info!("Got CTRL+C, exiting...");
        for sig in &exit_signals {
            sig.store(true, Ordering::Release);
        }
    })
    .context("Failed to setup CTRL+C handler")?;

    runtime.join_all();
    Ok(())
}
