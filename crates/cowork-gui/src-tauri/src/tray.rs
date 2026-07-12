// System Tray - Per-window tray icon with dynamic agent icons
//
// Each Cowork Forge process (window) creates its own tray icon.
// The tray menu shows the current project name (disabled), and actions
// to show the window, open settings, show about, and quit.
// When an agent is working, the tray icon switches to that agent's avatar.

use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Wry,
};

// ============================================================================
// Tray State
// ============================================================================

struct TrayState {
    project_name: Option<String>,
    current_agent: Option<String>,
    is_working: bool,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            project_name: None,
            current_agent: None,
            is_working: false,
        }
    }
}

// ============================================================================
// Tray Manager
// ============================================================================

pub struct TrayManager {
    app: Option<AppHandle>,
    tray: Option<TrayIcon>,
    state: Mutex<TrayState>,
}

impl TrayManager {
    fn new() -> Self {
        Self {
            app: None,
            tray: None,
            state: Mutex::new(TrayState::default()),
        }
    }

    pub fn init(&mut self, app: &AppHandle) -> tauri::Result<()> {
        self.app = Some(app.clone());

        let menu = self.build_menu(app)?;
        let icon = self.get_current_icon();
        let tooltip = self.get_tooltip();

        let tray = TrayIconBuilder::with_id("cowork-tray")
            .icon(icon)
            .menu(&menu)
            .tooltip(&tooltip)
            .on_menu_event(on_menu_event)
            .on_tray_icon_event(on_tray_icon_event)
            .build(app)?;

        self.tray = Some(tray);
        Ok(())
    }

    fn build_menu(&self, app: &AppHandle) -> tauri::Result<Menu<Wry>> {
        let state = self.state.lock().unwrap();

        let project_label = state
            .project_name
            .as_deref()
            .unwrap_or("No project opened");

        let project_item = MenuItem::with_id(app, "project_name", project_label, false, None::<&str>)?;
        let sep1 = PredefinedMenuItem::separator(app)?;
        let show_item = MenuItem::with_id(app, "show", "Show Cowork GUI", true, None::<&str>)?;
        let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
        let about_item = MenuItem::with_id(app, "about", "About Cowork Forge", true, None::<&str>)?;
        let sep2 = PredefinedMenuItem::separator(app)?;
        let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

        Menu::with_items(
            app,
            &[
                &project_item,
                &sep1,
                &show_item,
                &settings_item,
                &about_item,
                &sep2,
                &quit_item,
            ],
        )
    }

    fn get_current_icon(&self) -> Image<'static> {
        let state = self.state.lock().unwrap();
        if state.is_working {
            if let Some(agent_name) = &state.current_agent {
                return get_agent_icon(agent_name);
            }
        }
        get_default_icon()
    }

    fn get_tooltip(&self) -> String {
        let state = self.state.lock().unwrap();
        let mut parts: Vec<String> = vec!["Cowork Forge".to_string()];
        if let Some(name) = &state.project_name {
            parts.push(name.clone());
        }
        if state.is_working {
            if let Some(agent) = &state.current_agent {
                parts.push(format!("Working: {}", agent));
            } else {
                parts.push("Working...".to_string());
            }
        }
        parts.join(" - ")
    }

    fn update_icon(&self) {
        if let Some(tray) = &self.tray {
            let icon = self.get_current_icon();
            let _ = tray.set_icon(Some(icon));
        }
    }

    fn update_tooltip(&self) {
        if let Some(tray) = &self.tray {
            let tooltip = self.get_tooltip();
            let _ = tray.set_tooltip(Some(&tooltip));
        }
    }

    fn update_menu(&self) {
        if let (Some(app), Some(tray)) = (&self.app, &self.tray) {
            if let Ok(menu) = self.build_menu(app) {
                let _ = tray.set_menu(Some(menu));
            }
        }
    }

    pub fn set_project_name(&self, name: Option<String>) {
        {
            let mut state = self.state.lock().unwrap();
            state.project_name = name;
        }
        self.update_menu();
        self.update_tooltip();
    }

    pub fn set_working(&self, is_working: bool) {
        {
            let mut state = self.state.lock().unwrap();
            state.is_working = is_working;
            if !is_working {
                state.current_agent = None;
            }
        }
        self.update_icon();
        self.update_tooltip();
    }

    pub fn set_current_agent(&self, agent_name: Option<String>) {
        let changed = {
            let mut state = self.state.lock().unwrap();
            let changed = state.current_agent != agent_name
                || (agent_name.is_some() && !state.is_working);
            if changed {
                state.current_agent = agent_name.clone();
                if agent_name.is_some() {
                    state.is_working = true;
                }
            }
            changed
        };
        if changed {
            self.update_icon();
            self.update_tooltip();
        }
    }
}

// ============================================================================
// Icon Resolution
// ============================================================================

/// Decode an embedded PNG into a Tauri `Image` at runtime.
/// This avoids `include_image!` which requires PNGs to already be in RGBA
/// format at compile time. Using `include_bytes!` + runtime decoding handles
/// any PNG color type (RGB, RGBA, indexed, etc.).
fn decode_embedded_png(bytes: &'static [u8]) -> Image<'static> {
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Image::new_owned(rgba.into_raw(), width, height)
        }
        Err(e) => {
            tracing::error!("[Tray] Failed to decode embedded PNG: {}. Using 1x1 fallback.", e);
            Image::new_owned(vec![0, 0, 0, 0], 1, 1)
        }
    }
}

fn get_default_icon() -> Image<'static> {
    decode_embedded_png(include_bytes!("../icons/icon.png"))
}

/// Map an agent name to its avatar icon, mirroring the frontend's
/// `getAgentAvatar(agentName, stageName)` keyword matching in
/// `src/components/chat/MessageList.tsx`.
fn get_agent_icon(agent_name: &str) -> Image<'static> {
    let name_lower = agent_name.to_lowercase();
    if name_lower.contains("idea")
        || name_lower.contains("prd")
        || name_lower.contains("product manager")
        || name_lower.contains("pm agent")
    {
        decode_embedded_png(include_bytes!("../icons/avatar_role_pm.png"))
    } else if name_lower.contains("design") || name_lower.contains("architect") {
        decode_embedded_png(include_bytes!("../icons/avatar_role_designer.png"))
    } else if name_lower.contains("plan")
        || name_lower.contains("project manager")
        || name_lower.contains("engineer")
        || name_lower.contains("coding")
        || name_lower.contains("developer")
    {
        decode_embedded_png(include_bytes!("../icons/avatar_role_rd.png"))
    } else if name_lower.contains("check")
        || name_lower.contains("qa")
        || name_lower.contains("delivery")
        || name_lower.contains("reviewer")
    {
        decode_embedded_png(include_bytes!("../icons/avatar_role_qa.png"))
    } else {
        decode_embedded_png(include_bytes!("../icons/avatar_role_controller.png"))
    }
}

// ============================================================================
// Event Handlers
// ============================================================================

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        "show" => show_main_window(app),
        "settings" => {
            show_main_window(app);
            let _ = app.emit("tray_navigate", "settings");
        }
        "about" => {
            use tauri_plugin_dialog::DialogExt;
            let version = env!("CARGO_PKG_VERSION");
            let description = env!("CARGO_PKG_DESCRIPTION");
            app.dialog()
                .message(format!("Cowork Forge v{}\n\n{}\n\nAI-Native Multi-Agent Software Development Platform", version, description))
                .title("About Cowork Forge")
                .show(|_| {});
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

fn on_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    // Left-click on the tray icon restores the window (Windows convention).
    // On macOS, left-click also shows the menu by default, but showing the
    // window is the more useful action.
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_main_window(tray.app_handle());
    }
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

// ============================================================================
// Global Singleton
// ============================================================================

lazy_static! {
    static ref TRAY_MANAGER: Arc<Mutex<TrayManager>> = Arc::new(Mutex::new(TrayManager::new()));
}

/// Initialize the system tray. Should be called once during app setup.
pub fn init_tray(app: &AppHandle) {
    let mut manager = TRAY_MANAGER.lock().unwrap();
    if let Err(e) = manager.init(app) {
        tracing::error!("[Tray] Failed to initialize tray: {}", e);
    }
}

/// Update the project name shown in the tray menu.
pub fn set_project_name(name: Option<String>) {
    let manager = TRAY_MANAGER.lock().unwrap();
    manager.set_project_name(name);
}

/// Mark whether an agent is currently working.
/// When `is_working` is false, the icon resets to the default app icon.
pub fn set_working(is_working: bool) {
    let manager = TRAY_MANAGER.lock().unwrap();
    manager.set_working(is_working);
}

/// Set the current agent name. When set, the tray icon switches to that
/// agent's avatar. Pass `None` to clear.
pub fn set_current_agent(agent_name: Option<String>) {
    let manager = TRAY_MANAGER.lock().unwrap();
    manager.set_current_agent(agent_name);
}
