//! Écoute globale du clavier (et des clics) pour suivre le mot en cours partout.
//! Rien n'est enregistré : chaque touche est convertie en [`Touche`] puis envoyée au
//! coordinateur, qui ne garde que le mot en cours.

use std::sync::mpsc::Sender;

use crate::saisie::Touche;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Démarre l'écoute sur un fil dédié. Renvoie une erreur lisible si le système la refuse.
pub fn ecouter(tx: Sender<Touche>) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    return macos::ecouter(tx);
    #[cfg(target_os = "windows")]
    return windows::ecouter(tx);
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        drop(tx);
        Err("Les suggestions automatiques ne sont pas encore offertes sur ce système.".into())
    }
}
