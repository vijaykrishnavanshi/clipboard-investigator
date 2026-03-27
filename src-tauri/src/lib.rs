use serde::Serialize;

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
#[tauri::command]
fn read_clipboard() -> Vec<ClipboardEntry> {
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

            // Binary data (or text fallback)
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
#[tauri::command]
fn read_clipboard() -> Vec<ClipboardEntry> {
    vec![]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![read_clipboard])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
