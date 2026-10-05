//! Suggestions automatiques partout : reçoit les touches du système, attend une courte
//! pause, propose 3 mots dans l'overlay et remplace le mot tapé quand l'élève fait
//! Ctrl + 1/2/3.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

use crate::lexique::Lexique;
use crate::predicteur;
use crate::saisie::{Changement, Tampon, Touche};

const PAUSE: Duration = Duration::from_millis(150);
const LETTRES_MIN: usize = 3;
const MARGE: i32 = 24;

enum Message {
    Touche(Touche),
    Choisir(usize),
}

#[derive(Clone, Serialize)]
struct Affichage {
    saisie: String,
    mots: Vec<String>,
}

pub struct Auto {
    tx: Sender<Message>,
    actif: Arc<AtomicBool>,
}

/// Raccourcis Ctrl + 1/2/3, actifs seulement quand l'overlay est visible.
pub fn raccourcis_choix() -> [Shortcut; 3] {
    [Code::Digit1, Code::Digit2, Code::Digit3].map(|c| Shortcut::new(Some(Modifiers::CONTROL), c))
}

impl Auto {
    pub fn demarrer(app: AppHandle) -> Auto {
        let (tx, rx) = mpsc::channel();
        let actif = Arc::new(AtomicBool::new(true));
        let a = actif.clone();
        thread::spawn(move || boucle(app, rx, a));
        Auto { tx, actif }
    }

    /// Branche l'écoute du clavier du système sur le coordinateur.
    pub fn ecouter(&self) -> Result<(), String> {
        let (touches_tx, touches_rx) = mpsc::channel();
        crate::clavier::ecouter(touches_tx)?;
        let tx = self.tx.clone();
        thread::spawn(move || {
            for t in touches_rx {
                if tx.send(Message::Touche(t)).is_err() {
                    break;
                }
            }
        });
        Ok(())
    }

    pub fn choisir(&self, index: usize) {
        let _ = self.tx.send(Message::Choisir(index));
    }

    pub fn activer(&self, actif: bool) {
        self.actif.store(actif, Ordering::SeqCst);
        if !actif {
            let _ = self.tx.send(Message::Touche(Touche::Echap));
        }
    }
}

fn boucle(app: AppHandle, rx: Receiver<Message>, actif: Arc<AtomicBool>) {
    let mut tampon = Tampon::default();
    let mut proposes: Vec<String> = Vec::new();
    let mut a_calculer = false;
    loop {
        let message = if a_calculer {
            rx.recv_timeout(PAUSE)
        } else {
            rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
        };
        match message {
            Ok(Message::Touche(t)) => {
                if !actif.load(Ordering::SeqCst) {
                    tampon.vider();
                    if !proposes.is_empty() {
                        proposes.clear();
                        cacher(&app);
                    }
                    continue;
                }
                match tampon.traiter(t) {
                    Changement::Rien => {}
                    Changement::Mot => a_calculer = true,
                    Changement::Vide => {
                        a_calculer = false;
                        if !proposes.is_empty() {
                            proposes.clear();
                            cacher(&app);
                        }
                    }
                }
            }
            Ok(Message::Choisir(i)) => {
                if let Some(mot) = proposes.get(i) {
                    let mot = garder_majuscule(tampon.mot(), mot);
                    let n = tampon.longueur();
                    tampon.vider();
                    proposes.clear();
                    cacher(&app);
                    if let Err(e) = remplacer(&app, n, &mot) {
                        let _ = app.emit("auto-erreur", e);
                    }
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                a_calculer = false;
                proposes = proposer(tampon.mot());
                if proposes.is_empty() {
                    cacher(&app);
                } else {
                    montrer(
                        &app,
                        Affichage {
                            saisie: tampon.mot().to_string(),
                            mots: proposes
                                .iter()
                                .map(|m| garder_majuscule(tampon.mot(), m))
                                .collect(),
                        },
                    );
                }
            }
            Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

/// Les 3 meilleures suggestions, ou rien si le mot est déjà bien écrit.
fn proposer(saisie: &str) -> Vec<String> {
    if saisie.chars().count() < LETTRES_MIN {
        return Vec::new();
    }
    let suggestions = predicteur::suggerer(Lexique::global(), saisie, 3);
    let minuscule = saisie.to_lowercase();
    if suggestions.iter().any(|s| s.mot == minuscule) {
        return Vec::new();
    }
    suggestions.into_iter().map(|s| s.mot).collect()
}

/// « Fotografi » → « Photographie » : on garde la majuscule initiale de l'élève.
pub fn garder_majuscule(saisie: &str, mot: &str) -> String {
    let majuscule = saisie.chars().next().is_some_and(char::is_uppercase);
    let mut c = mot.chars();
    match (majuscule, c.next()) {
        (true, Some(premiere)) => premiere.to_uppercase().chain(c).collect(),
        _ => mot.to_string(),
    }
}

fn montrer(app: &AppHandle, affichage: Affichage) {
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || {
        let Some(overlay) = app2.get_webview_window("overlay") else {
            return;
        };
        if let (Ok(Some(ecran)), Ok(taille)) = (overlay.primary_monitor(), overlay.outer_size()) {
            let zone = ecran.work_area();
            let x = zone.position.x + zone.size.width as i32 - taille.width as i32 - MARGE;
            let y = zone.position.y + zone.size.height as i32 - taille.height as i32 - MARGE;
            let _ = overlay.set_position(PhysicalPosition::new(x, y));
        }
        let _ = overlay.emit("overlay", affichage);
        let _ = overlay.show();
        for r in raccourcis_choix() {
            if !app2.global_shortcut().is_registered(r) {
                let _ = app2.global_shortcut().register(r);
            }
        }
    });
}

fn cacher(app: &AppHandle) {
    let app2 = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(overlay) = app2.get_webview_window("overlay") {
            let _ = overlay.hide();
        }
        let _ = app2
            .global_shortcut()
            .unregister_multiple(raccourcis_choix());
    });
}

/// Efface le mot tapé et écrit la suggestion. La frappe se fait sur le fil principal
/// (exigence de macOS pour la simulation de touches).
fn remplacer(app: &AppHandle, effacer: usize, mot: &str) -> Result<(), String> {
    let (tx, rx) = mpsc::channel();
    let mot = mot.to_string();
    app.run_on_main_thread(move || {
        let resultat = (|| {
            let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
            // L'élève tient encore Ctrl : sans ça, Retour arrière effacerait des mots entiers.
            enigo
                .key(Key::Control, Direction::Release)
                .map_err(|e| e.to_string())?;
            for _ in 0..effacer {
                enigo
                    .key(Key::Backspace, Direction::Click)
                    .map_err(|e| e.to_string())?;
            }
            enigo.text(&mot).map_err(|e| e.to_string())
        })();
        let _ = tx.send(resultat);
    })
    .map_err(|e| e.to_string())?;
    rx.recv_timeout(Duration::from_secs(3))
        .map_err(|_| "Le clavier n'a pas répondu.".to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn majuscule_conservee() {
        assert_eq!(
            garder_majuscule("Fotografi", "photographie"),
            "Photographie"
        );
        assert_eq!(
            garder_majuscule("fotografi", "photographie"),
            "photographie"
        );
        assert_eq!(garder_majuscule("Elefan", "éléphant"), "Éléphant");
    }

    #[test]
    fn rien_si_mot_correct_ou_trop_court() {
        assert!(proposer("maison").is_empty());
        assert!(proposer("fo").is_empty());
        assert_eq!(proposer("fotografi")[0], "photographie");
    }
}
