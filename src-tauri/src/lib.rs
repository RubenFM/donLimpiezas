mod cleaner;
mod cmdline;
mod debloat;
mod duplicates;
mod log;
mod ps;
mod startup;
mod sysinfo;
mod system;
mod tweaks;
mod uninstaller;
mod updater;

use tauri::{Emitter, Manager};

/// Ejecuta trabajo pesado (disco, PowerShell) fuera del hilo de la interfaz.
async fn blocking<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn is_admin() -> bool {
    system::is_admin()
}

#[tauri::command]
async fn system_info() -> Result<sysinfo::SystemInfo, String> {
    blocking(|| Ok(sysinfo::info())).await
}

#[tauri::command]
fn relaunch_as_admin(app: tauri::AppHandle) -> Result<(), String> {
    system::relaunch_as_admin()?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
async fn scan_cleanup() -> Result<Vec<cleaner::CategoryScan>, String> {
    blocking(|| Ok(cleaner::scan())).await
}

#[tauri::command]
async fn run_cleanup(ids: Vec<String>) -> Result<Vec<cleaner::CleanResult>, String> {
    blocking(move || Ok(cleaner::clean(&ids))).await
}

#[tauri::command]
async fn list_programs() -> Result<Vec<uninstaller::Program>, String> {
    blocking(|| Ok(uninstaller::list())).await
}

/// Espera a que termine el desinstalador, que puede tardar lo que el usuario quiera.
#[tauri::command]
async fn uninstall_program(id: String) -> Result<uninstaller::UninstallResult, String> {
    blocking(move || uninstaller::uninstall(&id)).await
}

#[tauri::command]
async fn list_bloat() -> Result<Vec<debloat::BloatApp>, String> {
    blocking(|| debloat::list(system::is_admin())).await
}

#[tauri::command]
async fn create_restore_point() -> Result<String, String> {
    blocking(debloat::create_restore_point).await
}

#[tauri::command]
async fn remove_bloat(
    ids: Vec<String>,
    all_users: bool,
) -> Result<Vec<debloat::RemoveResult>, String> {
    if all_users && !system::is_admin() {
        return Err("Quitar apps para todos los usuarios requiere administrador.".into());
    }
    blocking(move || debloat::remove(&ids, all_users)).await
}

#[tauri::command]
async fn default_duplicate_folders() -> Result<Vec<String>, String> {
    blocking(duplicates::default_folders).await
}

/// Emite `duplicates-progress` mientras analiza; se detiene con `cancel_duplicates`.
#[tauri::command]
async fn find_duplicates(
    app: tauri::AppHandle,
    folders: Vec<String>,
    min_size: u64,
) -> Result<duplicates::ScanResult, String> {
    blocking(move || {
        let state = app.state::<duplicates::State>();
        duplicates::find(&folders, min_size, &state, |progress| {
            let _ = app.emit("duplicates-progress", progress);
        })
    })
    .await
}

#[tauri::command]
fn cancel_duplicates(state: tauri::State<duplicates::State>) {
    state.cancel();
}

#[tauri::command]
async fn trash_duplicates(
    app: tauri::AppHandle,
    paths: Vec<String>,
) -> Result<duplicates::TrashResult, String> {
    blocking(move || duplicates::trash(&paths, &app.state::<duplicates::State>())).await
}

#[tauri::command]
async fn list_tweaks() -> Result<Vec<tweaks::TweakState>, String> {
    blocking(|| Ok(tweaks::list())).await
}

#[tauri::command]
async fn apply_tweaks(ids: Vec<String>) -> Result<Vec<tweaks::TweakResult>, String> {
    blocking(move || tweaks::apply(&ids, system::is_admin())).await
}

#[tauri::command]
async fn revert_tweaks(ids: Vec<String>) -> Result<Vec<tweaks::TweakResult>, String> {
    blocking(move || tweaks::revert(&ids, system::is_admin())).await
}

#[tauri::command]
async fn list_updates(app: tauri::AppHandle) -> Result<Vec<updater::Update>, String> {
    blocking(move || updater::list(&app.state::<updater::State>())).await
}

/// Espera a que termine el instalador del programa.
#[tauri::command]
async fn update_program(app: tauri::AppHandle, id: String) -> Result<updater::UpdateResult, String> {
    blocking(move || updater::update(&id, &app.state::<updater::State>())).await
}

#[tauri::command]
async fn list_startup() -> Result<Vec<startup::StartupEntry>, String> {
    blocking(|| Ok(startup::list())).await
}

#[tauri::command]
async fn set_startup_enabled(
    source: startup::Source,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    blocking(move || startup::set_enabled(source, &name, enabled, system::is_admin())).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(duplicates::State::default())
        .manage(updater::State::default())
        .invoke_handler(tauri::generate_handler![
            is_admin,
            relaunch_as_admin,
            system_info,
            scan_cleanup,
            run_cleanup,
            list_programs,
            uninstall_program,
            list_bloat,
            create_restore_point,
            remove_bloat,
            default_duplicate_folders,
            find_duplicates,
            cancel_duplicates,
            trash_duplicates,
            list_tweaks,
            apply_tweaks,
            revert_tweaks,
            list_startup,
            set_startup_enabled,
            list_updates,
            update_program,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
