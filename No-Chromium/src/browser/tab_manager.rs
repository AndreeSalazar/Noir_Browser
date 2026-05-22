use crate::utils::ipc::{TabId, RendererMessage};
use tokio::sync::mpsc;
use std::collections::HashMap;

/// Estado de una pestaña - unificado con el tipo público en browser::mod.rs
#[derive(Debug)]
pub struct TabState {
    pub id: TabId,
    pub url: String,
    pub title: String,
    pub tx: mpsc::Sender<RendererMessage>,
}

pub struct TabManager {
    tabs: HashMap<TabId, TabState>,
    next_id: TabId,
}

impl TabManager {
    pub fn new() -> Self { 
        Self { 
            tabs: HashMap::new(), 
            next_id: 1 
        } 
    }

    pub fn create_tab(&mut self, url: String) -> anyhow::Result<TabId> {
        let (tx, _rx) = mpsc::channel(32);
        let id = self.next_id;
        self.next_id += 1;
        self.tabs.insert(id, TabState { 
            id, 
            url, 
            title: "New Tab".into(), 
            tx 
        });
        Ok(id)
    }

    pub fn remove_tab(&mut self, id: TabId) -> anyhow::Result<()> {
        self.tabs.remove(&id);
        Ok(())
    }

    pub fn get_state(&self, id: TabId) -> Option<&TabState> { 
        self.tabs.get(&id) 
    }
    
    /// Returns Vec of (TabId, &TabState) pairs for active tabs
    pub fn list_active(&self) -> Vec<(TabId, &TabState)> { 
        self.tabs.iter().map(|(id, state)| (*id, state)).collect() 
    }
}
