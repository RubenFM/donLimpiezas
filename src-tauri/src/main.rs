// Evita que se abra una consola junto a la ventana en las builds de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    don_limpiezas_lib::run()
}
