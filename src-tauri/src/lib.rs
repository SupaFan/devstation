mod commands;
mod models;

use commands::{git, project, runner, system};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            // Project commands
            project::load_config,
            project::save_config,
            project::select_folders,
            project::scan_projects,
            project::get_project_detail,
            project::update_project_name,
            project::toggle_favorite,
            project::add_project,
            project::remove_projects,
            // Git commands
            git::get_remote_url,
            git::get_branch,
            git::batch_pull,
            git::check_outdated,
            // Runner commands
            runner::run_dev,
            runner::run_build,
            runner::run_script,
            runner::stop_process_on_port,
            // System commands
            system::open_in_ide,
            system::open_in_terminal,
            system::open_in_finder,
            system::detect_port_in_use,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
