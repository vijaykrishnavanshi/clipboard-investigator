use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

static MENU_COUNTER: AtomicU64 = AtomicU64::new(0);
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(true);
static LAST_CHANGE_COUNT: AtomicU64 = AtomicU64::new(u64::MAX);
/// Timestamp (secs since epoch) of the last tray click — skip updates while menu is open
static MENU_OPENED_AT: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize, Clone)]
struct ClipboardEntry {
    type_name: String,
    data: String,
    is_text: bool,
    size: usize,
}

#[cfg(target_os = "macos")]
fn uti_is_text(uti: &str) -> bool {
    let lower = uti.to_lowercase();
    lower.contains("text")
        || lower.contains("string")
        || lower.contains("utf8")
        || lower.contains("utf16")
        || lower.contains("html")
        || lower.contains("rtf")
        || lower.contains("url")
        || lower.contains("xml")
        || lower.contains("json")
        || lower.contains("plist")
        || lower.contains("source-url")
        || lower == "public.url"
        || lower == "public.file-url"
}

#[cfg(target_os = "macos")]
fn read_clipboard_entries() -> Vec<ClipboardEntry> {
    use base64::Engine;
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;

    unsafe {
        let cls = Class::get("NSPasteboard").unwrap();
        let pb: *mut Object = msg_send![cls, generalPasteboard];
        if pb.is_null() {
            return vec![];
        }

        let types: *mut Object = msg_send![pb, types];
        if types.is_null() {
            return vec![];
        }

        let count: usize = msg_send![types, count];
        let mut entries = Vec::new();

        for i in 0..count {
            let type_obj: *mut Object = msg_send![types, objectAtIndex: i];
            if type_obj.is_null() {
                continue;
            }

            let utf8_ptr: *const i8 = msg_send![type_obj, UTF8String];
            if utf8_ptr.is_null() {
                continue;
            }
            let type_name = CStr::from_ptr(utf8_ptr).to_string_lossy().to_string();
            let is_text = uti_is_text(&type_name);

            if is_text {
                let ns_string: *mut Object = msg_send![pb, stringForType: type_obj];
                if !ns_string.is_null() {
                    let str_ptr: *const i8 = msg_send![ns_string, UTF8String];
                    if !str_ptr.is_null() {
                        let text = CStr::from_ptr(str_ptr).to_string_lossy().to_string();
                        entries.push(ClipboardEntry {
                            size: text.len(),
                            data: text,
                            is_text: true,
                            type_name,
                        });
                        continue;
                    }
                }
            }

            let ns_data: *mut Object = msg_send![pb, dataForType: type_obj];
            if !ns_data.is_null() {
                let length: usize = msg_send![ns_data, length];
                let bytes_ptr: *const u8 = msg_send![ns_data, bytes];
                if !bytes_ptr.is_null() && length > 0 {
                    let bytes = std::slice::from_raw_parts(bytes_ptr, length);
                    entries.push(ClipboardEntry {
                        type_name,
                        size: length,
                        data: base64::engine::general_purpose::STANDARD.encode(bytes),
                        is_text: false,
                    });
                } else {
                    entries.push(ClipboardEntry {
                        type_name,
                        size: 0,
                        data: String::new(),
                        is_text: false,
                    });
                }
            }
        }

        entries
    }
}

#[cfg(not(target_os = "macos"))]
fn read_clipboard_entries() -> Vec<ClipboardEntry> {
    vec![]
}

#[tauri::command]
fn read_clipboard() -> Vec<ClipboardEntry> {
    read_clipboard_entries()
}

fn get_clipboard_summary() -> String {
    let entries = read_clipboard_entries();
    if entries.is_empty() {
        return "(empty)".to_string();
    }

    if let Some(text_entry) = entries.iter().find(|e| e.is_text) {
        let preview = if text_entry.data.chars().count() > 60 {
            let truncated: String = text_entry.data.chars().take(60).collect();
            format!("{}...", truncated)
        } else {
            text_entry.data.clone()
        };
        let preview = preview.replace('\n', " ").replace('\r', "");
        return preview;
    }

    let types: Vec<&str> = entries.iter().map(|e| e.type_name.as_str()).collect();
    if types.iter().any(|t| t.contains("image")) {
        let size = entries
            .iter()
            .find(|e| e.type_name.contains("image"))
            .map(|e| e.size)
            .unwrap_or(0);
        return format!("[Image — {} bytes]", size);
    }

    format!("[{} types on clipboard]", entries.len())
}

#[cfg(target_os = "macos")]
fn clipboard_change_count() -> u64 {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let cls = Class::get("NSPasteboard").unwrap();
        let pb: *mut Object = msg_send![cls, generalPasteboard];
        if pb.is_null() {
            return 0;
        }
        let count: i64 = msg_send![pb, changeCount];
        count as u64
    }
}

#[cfg(not(target_os = "macos"))]
fn clipboard_change_count() -> u64 {
    0
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn build_tray_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    // Don't rebuild while the menu is likely open (within 10s of last click)
    let opened = MENU_OPENED_AT.load(Ordering::Relaxed);
    if opened > 0 && now_secs().saturating_sub(opened) < 10 {
        return Ok(());
    }

    // Only rebuild if clipboard content actually changed
    let current = clipboard_change_count();
    let previous = LAST_CHANGE_COUNT.swap(current, Ordering::Relaxed);
    if current == previous {
        return Ok(());
    }

    build_tray_menu_inner(app)
}

fn build_tray_menu_inner(app: &tauri::AppHandle) -> tauri::Result<()> {
    let entries = read_clipboard_entries();
    let summary = get_clipboard_summary();
    let n = MENU_COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut menu_builder = MenuBuilder::new(app);

    // Header: clipboard preview
    let header = MenuItemBuilder::with_id(format!("header_{n}"), format!("📋 {}", summary))
        .enabled(false)
        .build(app)?;
    menu_builder = menu_builder.item(&header);

    menu_builder = menu_builder.separator();

    // Show types on clipboard
    let types_header = MenuItemBuilder::with_id(
        format!("types_{n}"),
        format!("{} type(s) on clipboard", entries.len()),
    )
    .enabled(false)
    .build(app)?;
    menu_builder = menu_builder.item(&types_header);

    for (i, entry) in entries.iter().take(10).enumerate() {
        let label = if entry.is_text {
            format!("  {} — {} bytes (text)", entry.type_name, entry.size)
        } else {
            format!("  {} — {} bytes", entry.type_name, entry.size)
        };
        let item = MenuItemBuilder::with_id(format!("entry_{}_{}", n, i), label)
            .enabled(false)
            .build(app)?;
        menu_builder = menu_builder.item(&item);
    }
    if entries.len() > 10 {
        let more = MenuItemBuilder::with_id(
            format!("more_{n}"),
            format!("  ...and {} more", entries.len() - 10),
        )
        .enabled(false)
        .build(app)?;
        menu_builder = menu_builder.item(&more);
    }

    menu_builder = menu_builder.separator();

    let open_clipboard_item = MenuItemBuilder::with_id(
        format!("openclip_{n}"),
        "Open with Current Clipboard",
    )
    .build(app)?;
    menu_builder = menu_builder.item(&open_clipboard_item);

    let show_item =
        MenuItemBuilder::with_id(format!("show_{n}"), "Open Clipboard Investigator").build(app)?;
    menu_builder = menu_builder.item(&show_item);

    let quit_item = MenuItemBuilder::with_id(format!("quit_{n}"), "Quit").build(app)?;
    menu_builder = menu_builder.item(&quit_item);

    let menu = menu_builder.build()?;

    if let Some(tray) = app.tray_by_id("main-tray") {
        tray.set_menu(Some(menu))?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn activate_app() {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let ns_app: *mut Object =
            msg_send![Class::get("NSApplication").unwrap(), sharedApplication];
        let _: () = msg_send![ns_app, activateIgnoringOtherApps: true];
    }
}

fn show_window(app: &tauri::AppHandle) {
    if WINDOW_VISIBLE.load(Ordering::Relaxed) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.set_focus();
        }
        return;
    }

    #[cfg(target_os = "macos")]
    {
        app.set_activation_policy(tauri::ActivationPolicy::Regular);
        activate_app();
    }

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
    WINDOW_VISIBLE.store(true, Ordering::Relaxed);
}

fn hide_window(app: &tauri::AppHandle) {
    if !WINDOW_VISIBLE.load(Ordering::Relaxed) {
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    WINDOW_VISIBLE.store(false, Ordering::Relaxed);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_clipboard])
        .setup(|app| {
            let handle = app.handle().clone();

            // Build a minimal initial menu (will be replaced by timer immediately)
            let open_clip_item = MenuItemBuilder::with_id("openclip_init", "Open with Current Clipboard")
                .build(&handle)?;
            let show_item = MenuItemBuilder::with_id("show_init", "Open Clipboard Investigator")
                .build(&handle)?;
            let quit_item = MenuItemBuilder::with_id("quit_init", "Quit").build(&handle)?;
            let menu = MenuBuilder::new(&handle)
                .item(&open_clip_item)
                .item(&show_item)
                .item(&quit_item)
                .build()?;

            // Load icon from the embedded PNG for the tray
            let icon_bytes = include_bytes!("../icons/32x32.png");
            let icon_image = image::load_from_memory(icon_bytes).expect("Failed to load tray icon");
            let icon_rgba = icon_image.to_rgba8();
            let (w, h) = icon_rgba.dimensions();
            let tray_icon = tauri::image::Image::new_owned(icon_rgba.into_raw(), w, h);

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(tray_icon)
                .tooltip("Clipboard Investigator")
                .menu(&menu)
                .on_menu_event(move |app, event| {
                    // Menu item clicked — menu is closing, allow refreshes again
                    MENU_OPENED_AT.store(0, Ordering::Relaxed);
                    let id = event.id().as_ref();
                    if id.starts_with("openclip") {
                        show_window(app);
                        let _ = app.emit("read-clipboard-now", ());
                    } else if id.starts_with("show") {
                        show_window(app);
                    } else if id.starts_with("quit") {
                        app.exit(0);
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    if matches!(event, tauri::tray::TrayIconEvent::Click { .. }) {
                        // Mark menu as open — pause refreshes
                        MENU_OPENED_AT.store(now_secs(), Ordering::Relaxed);
                    }
                })
                .build(app)?;

            // Populate the tray with clipboard data
            LAST_CHANGE_COUNT.store(u64::MAX, Ordering::Relaxed);
            let _ = build_tray_menu(app.handle());

            // Background timer to keep the menu fresh
            // Skips updates while the menu is open (10s after click)
            let timer_handle = app.handle().clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                let handle = timer_handle.clone();
                let _ = timer_handle.run_on_main_thread(move || {
                    let _ = build_tray_menu(&handle);
                });
            });

            Ok(())
        })
        // Keep running when all windows are closed (menu bar app)
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                hide_window(window.app_handle());
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
