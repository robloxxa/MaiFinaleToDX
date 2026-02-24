use crate::config::Config;
use crate::error::{self, Result};
use crate::state::{ModuleStatus, SharedState};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tracing::{error, info, warn};

pub trait Module: Send + 'static {
    fn init(&mut self) -> Result<()> {
        Ok(())
    }

    fn poll(&mut self) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleName {
    TouchFinale,
    TouchDeluxe(u8),
    Jvs,
    Reader,
}

impl ModuleName {
    pub const fn as_str(&self) -> &str {
        match self {
            ModuleName::TouchFinale => "Touch Finale",
            ModuleName::TouchDeluxe(1) => "Touch Deluxe P1",
            ModuleName::TouchDeluxe(2) => "Touch Deluxe P2",
            ModuleName::TouchDeluxe(_) => "Touch Deluxe",
            ModuleName::Jvs => "JVS",
            ModuleName::Reader => "Card Reader",
        }
    }
}

impl fmt::Display for ModuleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

struct ModuleHandle {
    handle: JoinHandle<Result<()>>,
    exit_sig: Arc<AtomicBool>,
}

pub struct ModuleRuntime {
    config: Config,
    shared_state: SharedState,
    modules: Vec<(ModuleName, ModuleHandle)>,
}

impl ModuleRuntime {
    pub fn new(config: Config, shared_state: SharedState) -> Self {
        Self {
            config,
            shared_state,
            modules: Vec::new(),
        }
    }

    fn spawn(
        &mut self,
        name: ModuleName,
        create: impl FnOnce(Config, Arc<AtomicBool>, SharedState) -> Result<Box<dyn Module>>
            + Send
            + 'static,
    ) {
        let exit_sig = Arc::new(AtomicBool::new(false));
        let config = self.config.clone();
        let state = self.shared_state.clone();
        let statuses = self.shared_state.statuses.clone();

        statuses.set(name, ModuleStatus::Initializing);

        let exit_sig_clone = exit_sig.clone();
        let statuses_clone = statuses.clone();

        let handle = thread::Builder::new()
            .name(format!("{} Thread", name))
            .spawn(move || {
                let result = (|| -> Result<()> {
                    let mut module = create(config, Arc::clone(&exit_sig_clone), state)?;
                    module.init()?;
                    info!("Module {} successfully initialized", name);
                    statuses_clone.set(name, ModuleStatus::Running);
                    while !exit_sig_clone.load(Ordering::Acquire) {
                        module.poll()?;
                    }
                    Ok(())
                })();

                exit_sig_clone.store(true, Ordering::Release);

                match &result {
                    Ok(()) => {
                        info!("Module {} stopped", name);
                        statuses_clone.set(name, ModuleStatus::Stopped)
                    }
                    Err(error::Error::ModuleDisabled(name)) => {
                        warn!("Module {} disabled", name);
                        statuses_clone.set(name.clone(), ModuleStatus::Stopped)
                    }
                    Err(e) => {
                        error!("Module {} error: {}", name, e);
                        statuses_clone.set(name, ModuleStatus::Error(e.to_string().into()))
                    }
                }
                result
            });

        match handle {
            Ok(handle) => {
                self.modules.push((
                    name,
                    ModuleHandle {
                        handle,
                        exit_sig,
                    },
                ));
            }
            Err(e) => {
                error!("Failed to spawn thread for {}: {}", name, e);
                statuses.set(name, ModuleStatus::Error(e.to_string().into()));
            }
        }
    }

    pub fn start_all(&mut self) {
        #[cfg(feature = "touch")]
        {
            if self.config.touch.finale.enabled {
                self.start_module(ModuleName::TouchFinale);
            }
            if self.config.touch.dx.enabled {
                let emulated = self.config.touch.dx.mode
                    == crate::config::touch::TouchMode::Emulated;
                if emulated || self.config.touch.dx.p1_port.is_some() {
                    self.start_module(ModuleName::TouchDeluxe(1));
                }
                if emulated || self.config.touch.dx.p2_port.is_some() {
                    self.start_module(ModuleName::TouchDeluxe(2));
                }
            }
        }

        #[cfg(feature = "jvs")]
        if self.config.jvs.enabled {
            self.start_module(ModuleName::Jvs);
        }

        #[cfg(feature = "reader")]
        if self.config.reader.enabled {
            self.start_module(ModuleName::Reader);
        }
    }

    pub fn module_status(&self, name: ModuleName) -> ModuleStatus {
        self.shared_state.statuses.get(name)
    }

    pub fn start_module(&mut self, name: ModuleName) {
        self.stop_module(name);

        match name {
            #[cfg(feature = "touch")]
            ModuleName::TouchFinale => {
                self.spawn(name, |cfg, exit, state| {
                    crate::touch::finale::setup(cfg.touch.finale, exit, state)
                });
            }
            #[cfg(feature = "touch")]
            ModuleName::TouchDeluxe(n) => {
                self.spawn(name, move |cfg, exit, state| {
                    crate::touch::deluxe::setup(n, cfg.touch.dx, exit, state)
                });
            }
            #[cfg(feature = "jvs")]
            ModuleName::Jvs => {
                self.spawn(name, |cfg, exit, state| {
                    crate::jvs::setup(cfg.jvs, exit, state)
                });
            }
            #[cfg(feature = "reader")]
            ModuleName::Reader => {
                self.spawn(name, |cfg, exit, state| {
                    crate::card_reader::setup(cfg.reader, exit, state)
                });
            }
            #[allow(unreachable_patterns)]
            _ => {
                error!("Module {:?} not compiled in", name);
                self.shared_state.statuses.set(name, ModuleStatus::Stopped);
            }
        }
    }

    pub fn stop_module(&mut self, name: ModuleName) {
        let mut i = 0;
        while i < self.modules.len() {
            if self.modules[i].0 == name {
                let (_, module) = self.modules.remove(i);
                module.exit_sig.store(true, Ordering::Release);
                let _ = module.handle.join();
            } else {
                i += 1;
            }
        }
        self.shared_state.statuses.set(name, ModuleStatus::Stopped);
    }

    pub fn restart_module(&mut self, name: ModuleName) {
        self.stop_module(name);
        self.start_module(name);
    }

    pub fn stop_all(&mut self) {
        for (_, module) in self.modules.drain(..) {
            module.exit_sig.store(true, Ordering::Release);
            let _ = module.handle.join();
        }
    }

    pub fn update_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn shared_state(&self) -> &SharedState {
        &self.shared_state
    }

    pub fn exit_signals(&self) -> Vec<Arc<AtomicBool>> {
        self.modules
            .iter()
            .map(|(_, m)| m.exit_sig.clone())
            .collect()
    }

    pub fn join_all(mut self) {
        for (name, module) in self.modules.drain(..) {
            match module.handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => error!("Module {:?} error: {}", name, e),
                Err(_) => error!("Module {:?} panicked", name),
            }
        }
    }
}

impl Drop for ModuleRuntime {
    fn drop(&mut self) {
        self.stop_all();
    }
}
