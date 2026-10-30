mod cleaner;
mod debloat;
mod ps;
mod system;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            is_admin,
            relaunch_as_admin,
            scan_cleanup,
            run_cleanup,
            list_bloat,
            create_restore_point,
            remove_bloat,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar la aplicación");
}
