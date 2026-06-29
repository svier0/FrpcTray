mod config;
mod util;
mod toml_helper;
mod commands;

use std::fs;
use std::path::PathBuf;

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

use config::*;
use commands::server::*;
use commands::proxy::*;
use commands::frpc::*;
use commands::backup::*;
use commands::config_cmd::*;
use commands::frpc_manager::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(FrpcManager::new())
        .setup(|app| {
            let config_dir: PathBuf = if cfg!(target_os = "windows") {
                get_executable_dir().join("conf")
            } else {
                app.path().app_data_dir()
                    .unwrap_or_else(|_| get_executable_dir())
                    .join("conf")
            };
            fs::create_dir_all(&config_dir)
                .expect("无法创建配置目录");
            set_config_dir(config_dir);

            let bin_dir: PathBuf = if cfg!(target_os = "windows") {
                get_executable_dir().join("bin")
            } else {
                app.path().app_data_dir()
                    .unwrap_or_else(|_| get_executable_dir())
                    .join("bin")
            };
            fs::create_dir_all(&bin_dir)
                .expect("无法创建 bin 目录");
            set_bin_dir(bin_dir);

            // Sync autostart state from saved config
            let config = read_app_config();
            if config.autostart {
                let _ = set_autostart(true);
            }

            let show = MenuItemBuilder::with_id("show", "显示主界面").build(app)?;
            let light = MenuItemBuilder::with_id("light", "轻量模式").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&light)
                .item(&PredefinedMenuItem::separator(app)?)
                .item(&quit)
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .show_menu_on_left_click(false)
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                    "light" => {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.hide();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_servers,
            get_server,
            create_server,
            update_server,
            delete_server,
            reorder_servers,
            list_proxies,
            create_proxy,
            update_proxy,
            delete_proxy,
            reorder_proxies,
            get_frpc_version,
            upgrade_frpc,
            export_backup,
            restore_backup,
            get_config,
            save_config,
            start_frpc,
            stop_frpc,
            start_all_frpc,
            stop_all_frpc,
            get_all_frpc_status,
            open_log_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use crate::toml_helper::*;
    use crate::config::*;
    use crate::util::*;

    fn make_proxy(name: &str, port: u16) -> ProxyItem {
        ProxyItem {
            name: name.into(),
            desc: None,
            enabled: true,
            proxy_type: "http".into(),
            local_ip: None,
            local_port: port,
            remote_port: None,
            custom_domains: None,
            locations: None,
        }
    }

    #[test]
    fn test_update_proxies_reorder() {
        let doc_str = r#"
serverAddr = "1.2.3.4"
serverPort = 7000

[[proxies]]
name = "A"
type = "http"
localPort = 80

[[proxies]]
name = "B"
type = "http"
localPort = 81

[[proxies]]
name = "C"
type = "http"
localPort = 82
"#;
        let doc: toml_edit::DocumentMut = doc_str.parse().unwrap();

        let mut output = doc.to_string();
        output = strip_proxies_section(&output);
        if !output.ends_with('\n') { output.push('\n'); }
        output.push('\n');
        output.push_str(&serialize_proxies(&[
            make_proxy("C", 82),
            make_proxy("A", 80),
            make_proxy("B", 81),
        ]));
        output.push('\n');

        println!("OUTPUT:\n{}", output);

        let names: Vec<&str> = output
            .lines()
            .filter(|l| l.trim().starts_with("name = "))
            .map(|l| l.trim().trim_start_matches("name = \"").trim_end_matches('"'))
            .collect();

        assert_eq!(names, vec!["C", "A", "B"], "Expected C,A,B but got {:?}", names);
    }

    #[test]
    fn test_compare_versions_not_installed() {
        assert!(compare_versions("0", "0.69.1"));
    }

    #[test]
    fn test_compare_versions_equal() {
        assert!(!compare_versions("0.69.1", "0.69.1"));
    }

    #[test]
    fn test_compare_versions_newer() {
        assert!(!compare_versions("0.69.1", "0.68.0"));
        assert!(!compare_versions("0.69.1", "0.69.0"));
        assert!(!compare_versions("0.69.1", "0.68.9"));
    }

    #[test]
    fn test_compare_versions_upgrade_available() {
        assert!(compare_versions("0.68.0", "0.69.1"));
        assert!(compare_versions("0.69.0", "0.69.1"));
        assert!(compare_versions("0.68.9", "0.69.0"));
    }

    #[test]
    fn test_compare_versions_with_preview_suffix() {
        assert!(!compare_versions("0.69.1", "0.69.1-preview"));
        assert!(compare_versions("0.68.0", "0.69.0-preview"));
        assert!(!compare_versions("0.69.0-preview", "0.69.0"));
    }

    #[test]
    fn test_compare_versions_major_upgrade() {
        assert!(compare_versions("0.69.1", "1.0.0"));
        assert!(!compare_versions("1.0.0", "0.99.9"));
    }

    #[test]
    fn test_parse_version_from_location_github() {
        assert_eq!(
            parse_version_from_location("https://github.com/fatedier/frp/releases/tag/v0.69.1"),
            Some("0.69.1".to_string())
        );
    }

    #[test]
    fn test_parse_version_from_location_without_v_prefix() {
        assert_eq!(
            parse_version_from_location("https://github.com/fatedier/frp/releases/tag/0.69.1"),
            None
        );
    }

    #[test]
    fn test_parse_version_from_location_empty() {
        assert_eq!(parse_version_from_location(""), None);
    }

    #[test]
    fn test_parse_version_from_location_no_tag() {
        assert_eq!(
            parse_version_from_location("https://github.com/fatedier/frp/releases/latest"),
            None
        );
    }
}
