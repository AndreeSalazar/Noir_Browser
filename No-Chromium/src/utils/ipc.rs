//! Sistema de Comunicación Inter-Proceso (IPC)
//!
//! Define los tipos de mensajes intercambiados entre:
//! - Browser Process ↔ Renderer Process
//! - Renderer Process ↔ GPU Process  
//! - Renderer Process ↔ Network Process
//!
//! Usa canales MPSC de Tokio para comunicación async.

use tokio::sync::{mpsc, oneshot};
use crate::browser::TabId;

// ============================================================================
// MENSAJES: Browser Process ↔ Renderer Process
// ============================================================================

#[derive(Debug, Clone)]
pub enum BrowserMessage {
    /// Navegar a una URL en una pestaña específica
    Navigate {
        url: String,
        tab_id: TabId,
    },
    
    /// Detener la carga de una pestaña
    StopLoading {
        tab_id: TabId,
    },
    
    /// Obtener el título actual de una pestaña
    GetTitle {
        tab_id: TabId,
        reply: oneshot::Sender<String>,
    },
    
    /// Cerrar una pestaña y liberar recursos
    CloseTab {
        tab_id: TabId,
    },
    
    /// Actualizar estado de la UI para una pestaña
    UpdateState {
        tab_id: TabId,
        state: TabState,
    },
}

#[derive(Debug, Clone)]
pub enum RendererMessage {
    /// Confirmación de navegación iniciada
    NavigateStarted {
        tab_id: TabId,
        url: String,
    },
    
    /// Notificación de carga completada
    LoadComplete {
        tab_id: TabId,
        title: String,
        url: String,
    },
    
    /// Error durante la carga
    LoadError {
        tab_id: TabId,
        error: String,
    },
    
    /// Confirmación de cierre de pestaña
    TabClosed {
        tab_id: TabId,
    },
    
    /// Request para renderizar un frame
    RequestRender {
        tab_id: TabId,
        frame_data: FrameData,
    },
}

// ============================================================================
// MENSAJES: Renderer Process ↔ GPU Process
// ============================================================================

#[derive(Debug)]
pub enum RenderMessage {
    /// Submitir comandos de renderizado a la GPU
    SubmitFrame {
        commands: Vec<CommandBuffer>,
        semaphore: Semaphore,
        tab_id: TabId,
    },
    
    /// Notificar que el swap chain es inválido (resize, etc.)
    SwapChainInvalid {
        tab_id: TabId,
    },
    
    /// Redimensionar superficie de renderizado
    Resize {
        tab_id: TabId,
        width: u32,
        height: u32,
        scale_factor: f64,
    },
    
    /// Confirmación de frame presentado
    FramePresented {
        tab_id: TabId,
        frame_id: u64,
        present_time_ns: u64,
    },
}

// Tipos simplificados para comandos Vulkan
#[derive(Debug, Clone)]
pub struct CommandBuffer {
    pub data: Vec<u8>,
    pub pipeline_id: u32,
    pub bindless_offset: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Semaphore(pub u64); // Handle simplificado

#[derive(Debug, Clone)]
pub struct FrameData {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub textures: Vec<TextureHandle>,
    pub viewport: Viewport,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureHandle(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

// ============================================================================
// MENSAJES: Renderer Process ↔ Network Process
// ============================================================================

#[derive(Debug, Clone)]
pub enum NetworkMessage {
    /// Fetch asíncrono de una URL
    FetchUrl {
        url: url::Url,
        headers: Headers,
        method: HttpMethod,
        body: Option<Vec<u8>>,
        reply: oneshot::Sender<Result<Response, NetworkError>>,
        tab_id: TabId,
    },
    
    /// Establecer conexión WebSocket
    WebSocketConnect {
        url: url::Url,
        protocols: Vec<String>,
        reply: oneshot::Sender<Result<WsStream, NetworkError>>,
    },
    
    /// Resolver hostname vía DNS-over-HTTPS
    DnsResolve {
        hostname: String,
        reply: oneshot::Sender<Result<std::net::IpAddr, NetworkError>>,
    },
    
    /// Configurar proxy SOCKS5 (modo Tor)
    ConfigureProxy {
        proxy_url: String,
        credentials: Option<(String, String)>,
        reply: oneshot::Sender<Result<(), NetworkError>>,
    },
    
    /// Rotar circuito de anonimato (Tor mode)
    RotateCircuit {
        reply: oneshot::Sender<Result<(), NetworkError>>,
    },
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub inner: Vec<(String, String)>,
}

impl Headers {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    
    pub fn insert(&mut self, key: String, value: String) {
        self.inner.push((key, value));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Patch,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub status: u16,
    pub status_text: String,
    pub headers: Headers,
    pub body: Vec<u8>,
    pub url: String,
}

#[derive(Debug, Clone)]
pub struct WsStream {
    // Handle simplificado para stream WebSocket
    pub id: u64,
}

#[derive(Debug, Clone)]
pub enum NetworkError {
    Timeout,
    ConnectionRefused,
    DnsFailure,
    TlsError(String),
    ProxyError(String),
    Unknown(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timeout => write!(f, "Request timeout"),
            Self::ConnectionRefused => write!(f, "Connection refused"),
            Self::DnsFailure => write!(f, "DNS resolution failed"),
            Self::TlsError(e) => write!(f, "TLS error: {}", e),
            Self::ProxyError(e) => write!(f, "Proxy error: {}", e),
            Self::Unknown(e) => write!(f, "Network error: {}", e),
        }
    }
}

impl std::error::Error for NetworkError {}

// ============================================================================
// ESTADO DE PESTAÑA (compartido entre módulos)
// ============================================================================

#[derive(Debug, Clone)]
pub enum TabState {
    Idle,
    Loading { url: String, progress: f32 },
    Loaded { url: String, title: String },
    Error { url: String, message: String },
    Closed,
}

// ============================================================================
// CANALES IPC TIPO-SAFE
// ============================================================================

/// Par de canales para comunicación bidireccional Browser↔Renderer
pub struct BrowserRendererChannels {
    pub browser_to_renderer: mpsc::Sender<BrowserMessage>,
    pub renderer_to_browser: mpsc::Sender<RendererMessage>,
}

impl BrowserRendererChannels {
    pub fn new(buffer_size: usize) -> Self {
        let (b_to_r_tx, _) = mpsc::channel(buffer_size);
        let (_, r_to_b_tx) = mpsc::channel(buffer_size);
        // Los receivers se crean en los procesos respectivos
        Self {
            browser_to_renderer: b_to_r_tx,
            renderer_to_browser: r_to_b_tx,
        }
    }
}

/// Canales para comunicación Renderer↔GPU
pub struct RenderChannels {
    pub renderer_to_gpu: mpsc::Sender<RenderMessage>,
    pub gpu_to_renderer: mpsc::Sender<RenderMessage>,
}

/// Canales para comunicación Renderer↔Network
pub struct NetworkChannels {
    pub renderer_to_network: mpsc::Sender<NetworkMessage>,
    pub network_to_renderer: mpsc::Sender<NetworkMessage>,
}
