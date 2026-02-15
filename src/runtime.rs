use crate::config::Config;
use crate::error::Result;
use crate::state::{ModuleStatus, SharedState};
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleName {
    Touch,
    Jvs,
    Reader,
}

impl ModuleName {
    pub fn as_str(&self) -> &str {
        match self {
            ModuleName::Touch => "Touch",
            ModuleName::Jvs => "JVS",
            ModuleName::Reader => "Card Reader",
        }
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

    pub fn start_all(&mut self) {
        #[cfg(feature = "touch")]
        if self.config.touch.enabled {
            self.start_module(ModuleName::Touch);
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

    pub fn start_module(&mut self, name: ModuleName) {
        self.stop_module(name);

        self.set_status(name, ModuleStatus::Initializing);

        let exit_sig = Arc::new(AtomicBool::new(false));
        let state = self.shared_state.clone();

        let result = match name {
            #[cfg(feature = "touch")]
            ModuleName::Touch => {
                crate::touch::setup(&self.config.touch, exit_sig.clone(), Some(state))
            }
            #[cfg(feature = "jvs")]
            ModuleName::Jvs => {
                crate::jvs::setup(&self.config.jvs, exit_sig.clone(), Some(state))
            }
            #[cfg(feature = "reader")]
            ModuleName::Reader => {
                crate::card_reader::setup(&self.config.reader, exit_sig.clone(), Some(state))
            }
            #[allow(unreachable_patterns)]
            _ => {
                error!("Module {:?} not compiled in", name);
                self.set_status(name, ModuleStatus::Stopped);
                return;
            }
        };

        match result {
            Ok(handles) => {
                for handle in handles {
                    self.modules.push((name, ModuleHandle {
                        handle,
                        exit_sig: exit_sig.clone(),
                    }));
                }
                self.set_status(name, ModuleStatus::Running);
            }
            Err(e) => {
                error!("Failed to start module {:?}: {}", name, e);
                self.set_status(name, ModuleStatus::Error);
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
        self.set_status(name, ModuleStatus::Stopped);
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

    pub fn shared_state(&self) -> &SharedState {
        &self.shared_state
    }

    pub fn exit_signals(&self) -> Vec<Arc<AtomicBool>> {
        self.modules.iter().map(|(_, m)| m.exit_sig.clone()).collect()
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

    pub fn module_status(&self, name: ModuleName) -> ModuleStatus {
        let statuses = self.shared_state.statuses.lock().unwrap();
        match name {
            ModuleName::Touch => statuses.touch.unwrap_or(ModuleStatus::Stopped),
            ModuleName::Jvs => statuses.jvs.unwrap_or(ModuleStatus::Stopped),
            ModuleName::Reader => statuses.reader.unwrap_or(ModuleStatus::Stopped),
        }
    }

    fn set_status(&self, name: ModuleName, status: ModuleStatus) {
        let mut statuses = self.shared_state.statuses.lock().unwrap();
        match name {
            ModuleName::Touch => statuses.touch = Some(status),
            ModuleName::Jvs => statuses.jvs = Some(status),
            ModuleName::Reader => statuses.reader = Some(status),
        }
    }
}

impl Drop for ModuleRuntime {
    fn drop(&mut self) {
        self.stop_all();
    }
}
