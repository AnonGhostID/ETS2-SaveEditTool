fn main() {
    // Validate bundled runtime resource directories without Tauri/WebView build hooks.
    for directory in ["locales", "resources/plugins"] {
        let path = std::path::Path::new(directory);
        if path.exists() {
            println!("cargo:rerun-if-changed={directory}");
            reject_runtime_logs(path);
        }
    }
}

fn reject_runtime_logs(directory: &std::path::Path) {
    let entries = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("Cannot inspect install resources: {error}"));
    for entry in entries {
        let entry = entry.expect("Cannot inspect install resource");
        let path = entry.path();
        let kind = entry
            .file_type()
            .expect("Cannot inspect install resource type");
        if kind.is_dir() {
            reject_runtime_logs(&path);
        } else if kind.is_file() {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if name.ends_with(".log")
                || name.contains(".log.")
                || name.ends_with(".jsonl")
                || ((name.starts_with("application-") || name.starts_with("emergency-"))
                    && name.ends_with(".txt"))
            {
                panic!(
                    "Runtime log in install resources: {}. Remove it before packaging.",
                    path.display()
                );
            }
        } else if kind.is_symlink() {
            panic!(
                "Linked install resource cannot be checked for runtime logs: {}",
                path.display()
            );
        }
    }
}
