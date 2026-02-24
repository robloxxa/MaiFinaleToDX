use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::runtime::ModuleName;

#[cfg(feature = "gui")]
use crate::gui;

#[cfg(feature = "touch")]
use crate::touch;

#[cfg(feature = "jvs")]
use crate::jvs;

#[cfg(feature = "reader")]
use crate::card_reader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleStatus {
    Stopped,
    Initializing,
    Running,
    Error(Arc<str>),
}

#[derive(Debug, Clone)]
pub struct ModuleStatuses {
    inner: Arc<Mutex<HashMap<ModuleName, ModuleStatus>>>,
}

impl Default for ModuleStatuses {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ModuleStatuses {
    pub fn get(&self, name: ModuleName) -> ModuleStatus {
        self.inner
            .lock()
            .unwrap()
            .get(&name)
            .cloned()
            .unwrap_or(ModuleStatus::Stopped)
    }

    pub fn set(&self, name: ModuleName, status: ModuleStatus) {
        self.inner.lock().unwrap().insert(name, status);
    }
}

#[derive(Clone)]
pub struct SharedState {
    #[cfg(feature = "gui")]
    pub gui: Arc<gui::State>,
    
    #[cfg(feature = "touch")]
    pub touch: Arc<touch::TouchState>,
    
    #[cfg(feature = "jvs")]
    pub jvs: Arc<jvs::State>,
    
    #[cfg(feature = "reader")]
    pub reader: Arc<Mutex<card_reader::State>>,
    
    pub statuses: ModuleStatuses,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "gui")]
            gui: Arc::new(gui::State::new()),

            #[cfg(feature = "touch")]
            touch: Arc::new(touch::TouchState::new()),
            
            #[cfg(feature = "jvs")]
            jvs: Arc::new(jvs::State::default()),
            
            #[cfg(feature = "reader")]
            reader: Arc::new(Mutex::new(card_reader::State::default())),
            
            statuses: ModuleStatuses::default(),
        }
    }
}


