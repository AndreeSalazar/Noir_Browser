use tokio::sync::{mpsc, oneshot};
use std::fmt;

pub type TabId = u64;

#[derive(Debug)]
pub enum BrowserMessage {
    Navigate { url: String, tab_id: TabId },
    StopLoading { tab_id: TabId },
    GetTitle { tab_id: TabId, reply: oneshot::Sender<String> },
    CloseTab { tab_id: TabId },
    UpdateState { tab_id: TabId, title: String, url: String },
}

#[derive(Debug)]
pub enum RendererMessage {
    Navigate { url: String, tab_id: TabId },
    Resize { width: u32, height: u32 },
    CloseTab { tab_id: TabId },
}

#[derive(Debug)]
pub enum NetworkMessage {
    Fetch { url: String, reply: oneshot::Sender<Vec<u8>> },
}

pub struct IpcChannels {
    pub browser_to_renderer_tx: mpsc::Sender<RendererMessage>,
    pub browser_to_renderer_rx: mpsc::Receiver<RendererMessage>,
    pub renderer_to_browser_tx: mpsc::Sender<BrowserMessage>,
    pub renderer_to_browser_rx: mpsc::Receiver<BrowserMessage>,
}

impl IpcChannels {
    pub fn new() -> Self {
        let (b_to_r_tx, b_to_r_rx) = mpsc::channel(32);
        let (r_to_b_tx, r_to_b_rx) = mpsc::channel(32);
        Self {
            browser_to_renderer_tx: b_to_r_tx,
            browser_to_renderer_rx: b_to_r_rx,
            renderer_to_browser_tx: r_to_b_tx,
            renderer_to_browser_rx: r_to_b_rx,
        }
    }
}
