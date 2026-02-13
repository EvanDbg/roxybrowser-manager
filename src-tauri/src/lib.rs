mod commands;
mod models;

use commands::{process, profile, settings};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 创建系统托盘菜单
            let show = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
            let separator1 = tauri::menu::PredefinedMenuItem::separator(app)?;
            let start_roxy = MenuItemBuilder::with_id("start_roxy", "启动 RoxyBrowser").build(app)?;
            let stop_roxy = MenuItemBuilder::with_id("stop_roxy", "停止 RoxyBrowser").build(app)?;
            let separator2 = tauri::menu::PredefinedMenuItem::separator(app)?;
            let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .item(&separator1)
                .item(&start_roxy)
                .item(&stop_roxy)
                .item(&separator2)
                .item(&quit)
                .build()?;

            // 创建系统托盘图标
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("RoxyBrowser Manager")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "start_roxy" => {
                        let _ = process::start_roxy();
                    }
                    "stop_roxy" => {
                        let _ = process::stop_roxy();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button, .. } = event {
                        if button == tauri::tray::MouseButton::Left {
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 进程管理
            process::get_roxy_status,
            process::start_roxy,
            process::stop_roxy,
            // 配置文件管理
            profile::list_users,
            profile::switch_user,
            profile::delete_user,
            profile::update_user_note,
            profile::prepare_for_new_user,
            profile::finalize_new_user,
            // 导入导出
            profile::export_profiles,
            profile::import_profiles,
            // 设置管理
            settings::get_roxy_exe_path,
            settings::set_roxy_exe_path,
            settings::validate_roxy_exe_path,
            settings::auto_detect_roxy_path,
            settings::browse_for_exe,
            settings::browse_for_folder,
            settings::clear_roxy_exe_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
