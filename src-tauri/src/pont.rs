//! Pont avec l'application où l'élève écrit (Word, Google Docs…) : on passe par le
//! presse-papier et des raccourcis simulés, ce qui marche partout sans intégration.
//! Le contenu du presse-papier de l'élève est toujours restauré.

use std::thread;
use std::time::Duration;

use arboard::Clipboard;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[cfg(target_os = "macos")]
const MODIFICATEUR: Key = Key::Meta;
#[cfg(not(target_os = "macos"))]
const MODIFICATEUR: Key = Key::Control;

fn raccourci(lettre: char) -> Result<(), String> {
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| {
        format!("Contrôle du clavier refusé ({e}). Sur Mac : Réglages › Confidentialité › Accessibilité.")
    })?;
    // L'élève tient peut-être encore Maj/Alt du raccourci global : on les relâche.
    for k in [Key::Shift, Key::Alt] {
        let _ = enigo.key(k, Direction::Release);
    }
    enigo
        .key(MODIFICATEUR, Direction::Press)
        .and_then(|_| enigo.key(Key::Unicode(lettre), Direction::Click))
        .and_then(|_| enigo.key(MODIFICATEUR, Direction::Release))
        .map_err(|e| e.to_string())
}

fn presse_papier() -> Result<Clipboard, String> {
    Clipboard::new().map_err(|e| format!("Presse-papier inaccessible : {e}"))
}

/// Copie la sélection de l'application active et la renvoie.
pub fn lire_selection() -> Result<String, String> {
    let mut pp = presse_papier()?;
    let ancien = pp.get_text().ok();
    let _ = pp.clear();
    raccourci('c')?;
    thread::sleep(Duration::from_millis(200));
    let texte = pp.get_text().unwrap_or_default();
    if let Some(a) = ancien {
        let _ = pp.set_text(a);
    }
    if texte.trim().is_empty() {
        Err("Sélectionne du texte d'abord, puis refais le raccourci.".into())
    } else {
        Ok(texte)
    }
}

/// Colle `texte` dans l'application active.
pub fn coller(texte: &str) -> Result<(), String> {
    let mut pp = presse_papier()?;
    let ancien = pp.get_text().ok();
    pp.set_text(texte.to_string()).map_err(|e| e.to_string())?;
    raccourci('v')?;
    // L'application cible lit le presse-papier de façon asynchrone.
    thread::sleep(Duration::from_millis(400));
    if let Some(a) = ancien {
        let _ = pp.set_text(a);
    }
    Ok(())
}
