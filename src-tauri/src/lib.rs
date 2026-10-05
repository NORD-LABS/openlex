pub mod lexique;
pub mod phonetiseur;
pub mod predicteur;

use lexique::Lexique;
use predicteur::Suggestion;

#[tauri::command]
fn suggerer(saisie: String) -> Vec<Suggestion> {
    predicteur::suggerer(Lexique::global(), &saisie, 9)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![suggerer])
        .run(tauri::generate_context!())
        .expect("erreur au lancement d'OpenLex");
}
