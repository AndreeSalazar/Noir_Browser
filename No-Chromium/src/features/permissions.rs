//! Permissions API - Permisos granulares para APIs del navegador
//!
//! notifications, geolocation, camera, microphone, clipboard, midi, etc.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    Notifications,
    Geolocation,
    Camera,
    Microphone,
    ClipboardRead,
    ClipboardWrite,
    Midi,
    BackgroundSync,
    PersistentStorage,
    Push,
    Sensors,
    ScreenWakeLock,
}

impl Permission {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "notifications" | "notification" => Some(Self::Notifications),
            "geolocation" | "geo" => Some(Self::Geolocation),
            "camera" | "video" => Some(Self::Camera),
            "microphone" | "mic" | "audio" => Some(Self::Microphone),
            "clipboard-read" | "clipboardread" => Some(Self::ClipboardRead),
            "clipboard-write" | "clipboardwrite" => Some(Self::ClipboardWrite),
            "midi" => Some(Self::Midi),
            "background-sync" | "backgroundsync" => Some(Self::BackgroundSync),
            "persistent-storage" | "persistentstorage" => Some(Self::PersistentStorage),
            "push" => Some(Self::Push),
            "sensors" => Some(Self::Sensors),
            "screen-wake-lock" | "wakelock" => Some(Self::ScreenWakeLock),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Notifications => "notifications",
            Self::Geolocation => "geolocation",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::ClipboardRead => "clipboard-read",
            Self::ClipboardWrite => "clipboard-write",
            Self::Midi => "midi",
            Self::BackgroundSync => "background-sync",
            Self::PersistentStorage => "persistent-storage",
            Self::Push => "push",
            Self::Sensors => "sensors",
            Self::ScreenWakeLock => "screen-wake-lock",
        }
    }

    pub fn all() -> Vec<Permission> {
        vec![
            Self::Notifications, Self::Geolocation, Self::Camera, Self::Microphone,
            Self::ClipboardRead, Self::ClipboardWrite, Self::Midi, Self::BackgroundSync,
            Self::PersistentStorage, Self::Push, Self::Sensors, Self::ScreenWakeLock,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

impl PermissionState {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Granted => "granted",
            Self::Denied => "denied",
            Self::Prompt => "prompt",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub id: u32,
    pub permission: Permission,
    pub origin: String,
    pub state: PermissionState,
}

pub struct PermissionsManager {
    /// Permisos globales (defaults)
    defaults: HashMap<Permission, PermissionState>,
    /// Permisos por origen
    per_origin: HashMap<String, HashMap<Permission, PermissionState>>,
    next_id: u32,
    /// Block list: orígenes bloqueados completamente
    blocked: Vec<String>,
}

impl PermissionsManager {
    pub fn new() -> Self {
        let mut defaults = HashMap::new();
        for p in Permission::all() {
            defaults.insert(p, PermissionState::Prompt);
        }
        Self {
            defaults,
            per_origin: HashMap::new(),
            next_id: 1,
            blocked: Vec::new(),
        }
    }

    /// Set default para un permiso
    pub fn set_default(&mut self, p: Permission, state: PermissionState) {
        self.defaults.insert(p, state);
    }

    /// Request permiso para un origen
    pub fn request(&mut self, permission: Permission, origin: &str) -> PermissionState {
        if self.blocked.contains(&origin.to_string()) {
            return PermissionState::Denied;
        }
        let state = *self.defaults.get(&permission).unwrap_or(&PermissionState::Prompt);
        self.per_origin
            .entry(origin.to_string())
            .or_default()
            .insert(permission, state);
        state
    }

    /// Check estado actual de un permiso
    pub fn check(&self, permission: Permission, origin: &str) -> PermissionState {
        if let Some(origin_perms) = self.per_origin.get(origin) {
            if let Some(state) = origin_perms.get(&permission) {
                return *state;
            }
        }
        *self.defaults.get(&permission).unwrap_or(&PermissionState::Prompt)
    }

    /// Revoca un permiso para un origen
    pub fn revoke(&mut self, permission: Permission, origin: &str) {
        self.per_origin
            .entry(origin.to_string())
            .or_default()
            .insert(permission, PermissionState::Denied);
    }

    /// Otorga un permiso para un origen
    pub fn grant(&mut self, permission: Permission, origin: &str) {
        self.per_origin
            .entry(origin.to_string())
            .or_default()
            .insert(permission, PermissionState::Granted);
    }

    /// Bloquea un origen completo
    pub fn block_origin(&mut self, origin: &str) {
        if !self.blocked.contains(&origin.to_string()) {
            self.blocked.push(origin.to_string());
        }
    }

    pub fn unblock_origin(&mut self, origin: &str) {
        self.blocked.retain(|o| o != origin);
    }

    pub fn is_blocked(&self, origin: &str) -> bool {
        self.blocked.contains(&origin.to_string())
    }

    /// Reset todos los permisos de un origen
    pub fn reset_origin(&mut self, origin: &str) {
        self.per_origin.remove(origin);
    }

    /// Lista permisos otorgados a un origen
    pub fn granted_for(&self, origin: &str) -> Vec<Permission> {
        self.per_origin.get(origin)
            .map(|perms| perms.iter()
                .filter(|(_, s)| **s == PermissionState::Granted)
                .map(|(p, _)| *p)
                .collect())
            .unwrap_or_default()
    }

    /// Lista permisos denegados a un origen
    pub fn denied_for(&self, origin: &str) -> Vec<Permission> {
        self.per_origin.get(origin)
            .map(|perms| perms.iter()
                .filter(|(_, s)| **s == PermissionState::Denied)
                .map(|(p, _)| *p)
                .collect())
            .unwrap_or_default()
    }

    /// Check si permiso es high-risk (requiere prompt)
    pub fn is_high_risk(p: Permission) -> bool {
        matches!(p,
            Permission::Notifications | Permission::Geolocation |
            Permission::Camera | Permission::Microphone |
            Permission::ClipboardRead | Permission::Midi |
            Permission::PersistentStorage | Permission::Sensors
        )
    }
}

impl Default for PermissionsManager {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_from_str() {
        assert_eq!(Permission::from_str("notifications"), Some(Permission::Notifications));
        assert_eq!(Permission::from_str("geolocation"), Some(Permission::Geolocation));
        assert_eq!(Permission::from_str("camera"), Some(Permission::Camera));
        assert_eq!(Permission::from_str("unknown"), None);
    }

    #[test]
    fn test_permission_to_str() {
        assert_eq!(Permission::Notifications.to_str(), "notifications");
    }

    #[test]
    fn test_permission_all() {
        let all = Permission::all();
        assert!(all.len() >= 10);
    }

    #[test]
    fn test_state_to_str() {
        assert_eq!(PermissionState::Granted.to_str(), "granted");
        assert_eq!(PermissionState::Denied.to_str(), "denied");
        assert_eq!(PermissionState::Prompt.to_str(), "prompt");
    }

    #[test]
    fn test_manager_new_defaults() {
        let m = PermissionsManager::new();
        assert_eq!(m.check(Permission::Geolocation, "any.com"), PermissionState::Prompt);
    }

    #[test]
    fn test_request() {
        let mut m = PermissionsManager::new();
        let state = m.request(Permission::Notifications, "example.com");
        assert_eq!(state, PermissionState::Prompt);
    }

    #[test]
    fn test_grant() {
        let mut m = PermissionsManager::new();
        m.grant(Permission::Geolocation, "example.com");
        assert_eq!(m.check(Permission::Geolocation, "example.com"), PermissionState::Granted);
    }

    #[test]
    fn test_revoke() {
        let mut m = PermissionsManager::new();
        m.grant(Permission::Camera, "example.com");
        m.revoke(Permission::Camera, "example.com");
        assert_eq!(m.check(Permission::Camera, "example.com"), PermissionState::Denied);
    }

    #[test]
    fn test_block_origin() {
        let mut m = PermissionsManager::new();
        m.block_origin("evil.com");
        assert!(m.is_blocked("evil.com"));
        assert_eq!(m.request(Permission::Geolocation, "evil.com"), PermissionState::Denied);
    }

    #[test]
    fn test_unblock_origin() {
        let mut m = PermissionsManager::new();
        m.block_origin("evil.com");
        m.unblock_origin("evil.com");
        assert!(!m.is_blocked("evil.com"));
    }

    #[test]
    fn test_reset_origin() {
        let mut m = PermissionsManager::new();
        m.grant(Permission::Camera, "x.com");
        m.grant(Permission::Microphone, "x.com");
        m.reset_origin("x.com");
        assert_eq!(m.check(Permission::Camera, "x.com"), PermissionState::Prompt);
    }

    #[test]
    fn test_granted_for() {
        let mut m = PermissionsManager::new();
        m.grant(Permission::Camera, "x.com");
        m.grant(Permission::Microphone, "x.com");
        m.revoke(Permission::Geolocation, "x.com");
        let granted = m.granted_for("x.com");
        assert_eq!(granted.len(), 2);
    }

    #[test]
    fn test_denied_for() {
        let mut m = PermissionsManager::new();
        m.grant(Permission::Camera, "x.com");
        m.revoke(Permission::Geolocation, "x.com");
        let denied = m.denied_for("x.com");
        assert_eq!(denied.len(), 1);
        assert_eq!(denied[0], Permission::Geolocation);
    }

    #[test]
    fn test_set_default() {
        let mut m = PermissionsManager::new();
        m.set_default(Permission::Notifications, PermissionState::Granted);
        assert_eq!(m.check(Permission::Notifications, "new.com"), PermissionState::Granted);
    }

    #[test]
    fn test_is_high_risk() {
        assert!(PermissionsManager::is_high_risk(Permission::Camera));
        assert!(!PermissionsManager::is_high_risk(Permission::ClipboardWrite));
    }

    #[test]
    fn test_check_unset_returns_default() {
        let mut m = PermissionsManager::new();
        m.set_default(Permission::Geolocation, PermissionState::Denied);
        assert_eq!(m.check(Permission::Geolocation, "never-asked.com"), PermissionState::Denied);
    }

    #[test]
    fn test_per_origin_isolation() {
        let mut m = PermissionsManager::new();
        m.grant(Permission::Camera, "a.com");
        m.revoke(Permission::Camera, "b.com");
        assert_eq!(m.check(Permission::Camera, "a.com"), PermissionState::Granted);
        assert_eq!(m.check(Permission::Camera, "b.com"), PermissionState::Denied);
        assert_eq!(m.check(Permission::Camera, "c.com"), PermissionState::Prompt);
    }
}
