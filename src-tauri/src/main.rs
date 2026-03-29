// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn print_help() {
    eprintln!("Clipboard Investigator");
    eprintln!();
    eprintln!("Usage: clipboard-investigator [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --cli          Print clipboard contents to stdout and exit");
    eprintln!("  --json         Output as JSON (implies --cli)");
    eprintln!("  --types        List clipboard type names only (implies --cli)");
    eprintln!("  --help         Show this help message");
    eprintln!();
    eprintln!("Without options, launches the GUI app.");
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let cli = args.iter().any(|a| a == "--cli");
    let json = args.iter().any(|a| a == "--json");
    let types = args.iter().any(|a| a == "--types");
    let help = args.iter().any(|a| a == "--help" || a == "-h");

    if help {
        print_help();
        return;
    }

    if cli || json || types {
        let entries = app_lib::read_clipboard_entries();

        if json {
            println!("{}", serde_json::to_string_pretty(&entries).unwrap_or_default());
        } else if types {
            for entry in &entries {
                println!("{}", entry.type_name);
            }
        } else {
            if entries.is_empty() {
                println!("(clipboard is empty)");
                return;
            }
            println!("{} type(s) on clipboard:\n", entries.len());
            for entry in &entries {
                let kind = if entry.is_text { "text" } else { "binary" };
                println!("  {}  [{}]  {}", entry.type_name, kind, format_size(entry.size));
                if entry.is_text && !entry.data.is_empty() {
                    let preview: String = entry.data.chars().take(200).collect();
                    let truncated = if entry.data.chars().count() > 200 { "..." } else { "" };
                    for line in preview.lines().take(5) {
                        println!("    {}", line);
                    }
                    if truncated == "..." || entry.data.lines().count() > 5 {
                        println!("    ...");
                    }
                }
                println!();
            }
        }
        return;
    }

    app_lib::run();
}
