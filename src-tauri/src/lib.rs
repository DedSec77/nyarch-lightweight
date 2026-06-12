// nyarch-lite — a lightweight desktop shell.
//
// Unlike the full `nyarch-client`, this build does NOT bundle the React app.
// It simply opens the live website in a native WebView window, so:
//   * the binary/installer is tiny (no embedded frontend),
//   * the app always shows the latest deployed site (no rebuild on update),
//   * a system-tray icon keeps it running in the background for notifications.
//
// The site URL is taken from the NYARCH_URL env var at build time (baked in),
// falling back to the constant below. Set it in CI via a repo secret/variable.

use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_notification::NotificationExt;

// Compile-time default; override by exporting NYARCH_URL before `tauri build`.
const DEFAULT_URL: &str = match option_env!("NYARCH_URL") {
    Some(u) => u,
    None => "https://nyarch.example.com",
};

static HINTED_TRAY: AtomicBool = AtomicBool::new(false);

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Linux / WebKitGTK compatibility (see nyarch-client for full rationale):
    // NVIDIA + Wayland (Hyprland/Sway) crashes/stutters, and AppImage white-
    // screens. Run the WebView through XWayland, disable the DMABUF renderer,
    // and turn off the WebKit sandbox inside AppImage. All user-overridable.
    #[cfg(target_os = "linux")]
    {
        let set_default = |k: &str, v: &str| {
            if std::env::var_os(k).is_none() {
                std::env::set_var(k, v);
            }
        };
        let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var("XDG_SESSION_TYPE")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false);
        if is_wayland {
            set_default("GDK_BACKEND", "x11");
        }
        set_default("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        set_default("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        if std::env::var_os("APPIMAGE").is_some() {
            set_default("WEBKIT_FORCE_SANDBOX", "0");
        }
    }

    let url = std::env::var("NYARCH_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
    let parsed = url
        .parse()
        .unwrap_or_else(|_| DEFAULT_URL.parse().expect("valid default URL"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            // Build the main window pointing at the remote site.
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(parsed))
                .title("nyarch")
                .inner_size(1200.0, 800.0)
                .min_inner_size(480.0, 600.0)
                .resizable(true)
                .build()?;

            // ── system tray ──────────────────────────────────────────
            let show_item = MenuItem::with_id(app, "show", "Show nyarch", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("nyarch")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
                if !HINTED_TRAY.swap(true, Ordering::Relaxed) {
                    let _ = window
                        .app_handle()
                        .notification()
                        .builder()
                        .title("nyarch is still running")
                        .body("Closed to the tray. Right-click the tray icon to quit.")
                        .show();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running nyarch-lightweight");
}
