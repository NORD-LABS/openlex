pub mod auto;
mod clavier;
pub mod lexique;
pub mod phonetiseur;
pub mod pont;
pub mod predicteur;
pub mod saisie;
pub mod voix;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use auto::Auto;
use lexique::Lexique;
use predicteur::Suggestion;
use voix::Voix;

#[derive(Clone, Serialize)]
struct Lecture {
    phrases: Vec<String>,
}

/// Erreur de démarrage de l'écoute du clavier (permission refusée…), pour l'interface.
struct ErreurAuto(Mutex<Option<String>>);

#[derive(Serialize)]
struct EtatAuto {
    erreur: Option<String>,
}

#[tauri::command]
fn auto_etat(erreur: State<ErreurAuto>) -> EtatAuto {
    EtatAuto {
        erreur: erreur.0.lock().ok().and_then(|e| e.clone()),
    }
}

#[tauri::command]
fn auto_activer(actif: bool, auto: State<Auto>) {
    auto.activer(actif);
}

#[tauri::command]
fn suggerer(saisie: String) -> Vec<Suggestion> {
    predicteur::suggerer(Lexique::global(), &saisie, 9)
}

/// Lit un texte à voix haute ; renvoie les phrases pour le surlignage.
#[tauri::command]
fn lire(texte: String, voix: State<Voix>) -> Vec<String> {
    let phrases = voix::decouper_phrases(&texte);
    voix.parler(phrases.clone());
    phrases
}

#[tauri::command]
fn arreter(voix: State<Voix>) {
    voix.arreter();
}

/// Rend la main à l'application de l'élève, y colle le mot, puis réaffiche OpenLex.
#[tauri::command]
async fn inserer(mot: String, app: AppHandle) -> Result<(), String> {
    let fenetre = app
        .get_webview_window("main")
        .ok_or("fenêtre introuvable")?;
    let _ = fenetre.hide();
    #[cfg(target_os = "macos")]
    let _ = app.hide();
    thread::sleep(Duration::from_millis(250));
    let resultat = pont::coller(&app, &format!("{mot} "));
    let _ = fenetre.show();
    resultat
}

fn lire_selection(app: &AppHandle) {
    match pont::lire_selection(app) {
        Ok(texte) => {
            let phrases = voix::decouper_phrases(&texte);
            app.state::<Voix>().parler(phrases.clone());
            let _ = app.emit("selection", Lecture { phrases });
        }
        Err(message) => {
            let _ = app.emit("voix", voix::Evenement::Erreur { message });
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "macos")]
    let modificateurs = Modifiers::SUPER | Modifiers::SHIFT;
    #[cfg(not(target_os = "macos"))]
    let modificateurs = Modifiers::CONTROL | Modifiers::SHIFT;
    let raccourci_lire = Shortcut::new(Some(modificateurs), Code::KeyL);
    let choix = auto::raccourcis_choix();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, raccourci, evenement| {
                    if evenement.state() != ShortcutState::Pressed {
                        return;
                    }
                    if raccourci == &raccourci_lire {
                        let app = app.clone();
                        thread::spawn(move || lire_selection(&app));
                    } else if let Some(i) = choix.iter().position(|c| c == raccourci) {
                        app.state::<Auto>().choisir(i);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            app.manage(Voix::demarrer(app.handle().clone()));
            let auto = Auto::demarrer(app.handle().clone());
            let erreur = auto.ecouter().err();
            app.manage(auto);
            app.manage(ErreurAuto(Mutex::new(erreur)));
            // Décompresse le lexique tout de suite pour que la première suggestion soit rapide.
            thread::spawn(|| {
                Lexique::global();
            });
            if let Err(e) = app.global_shortcut().register(raccourci_lire) {
                eprintln!("Raccourci global indisponible : {e}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            suggerer,
            lire,
            arreter,
            inserer,
            auto_etat,
            auto_activer
        ])
        .run(tauri::generate_context!())
        .expect("erreur au lancement d'OpenLex");
}
