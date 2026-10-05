//! Synthèse vocale hors ligne avec les voix du système (Windows / macOS).
//! Un fil dédié possède le moteur et lit le texte phrase par phrase, ce qui permet à
//! l'interface de surligner la phrase en cours sans dépendre des rappels de chaque OS.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tts::Tts;

#[derive(Clone, Serialize)]
#[serde(tag = "etat", rename_all = "lowercase")]
pub enum Evenement {
    Phrase { index: usize },
    Fin,
    Erreur { message: String },
}

enum Commande {
    Parler(Vec<String>),
    Arreter,
}

pub struct Voix {
    tx: Sender<Commande>,
}

/// Découpe un texte en phrases (ponctuation forte ou retour à la ligne).
pub fn decouper_phrases(texte: &str) -> Vec<String> {
    let mut phrases = Vec::new();
    let mut courante = String::new();
    for c in texte.chars() {
        if c == '\n' {
            pousser(&mut phrases, &mut courante);
            continue;
        }
        courante.push(c);
        if matches!(c, '.' | '!' | '?' | '…' | ';') {
            pousser(&mut phrases, &mut courante);
        }
    }
    pousser(&mut phrases, &mut courante);
    phrases
}

fn pousser(phrases: &mut Vec<String>, courante: &mut String) {
    let t = courante.trim();
    if t.chars().any(char::is_alphanumeric) {
        phrases.push(t.to_string());
    } else if let Some(derniere) = phrases.last_mut() {
        // ponctuation isolée (ex. « ?! ») : on la rattache à la phrase précédente
        derniere.push_str(t);
    }
    courante.clear();
}

fn choisir_voix_francaise(tts: &mut Tts) -> Result<(), String> {
    let voix = tts.voices().map_err(|e| e.to_string())?;
    let francaises: Vec<_> = voix
        .into_iter()
        .filter(|v| v.language().primary_language() == "fr")
        .collect();
    let choix = francaises
        .iter()
        .find(|v| v.language().region() == Some("CA"))
        .or_else(|| francaises.first());
    match choix {
        Some(v) => tts.set_voice(v).map_err(|e| e.to_string()),
        None => Err("Aucune voix française n'est installée sur cet ordinateur.".into()),
    }
}

impl Voix {
    pub fn demarrer(app: AppHandle) -> Voix {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || boucle(app, rx));
        Voix { tx }
    }

    pub fn parler(&self, phrases: Vec<String>) {
        let _ = self.tx.send(Commande::Parler(phrases));
    }

    pub fn arreter(&self) {
        let _ = self.tx.send(Commande::Arreter);
    }
}

fn boucle(app: AppHandle, rx: Receiver<Commande>) {
    let emettre = |e: Evenement| {
        let _ = app.emit("voix", e);
    };
    let mut tts = match Tts::default() {
        Ok(t) => t,
        Err(e) => {
            // On vide les commandes pour signaler l'erreur à chaque demande.
            for _ in rx {
                emettre(Evenement::Erreur {
                    message: format!("Synthèse vocale indisponible : {e}"),
                });
            }
            return;
        }
    };
    let avertissement = choisir_voix_francaise(&mut tts).err();
    let suit_la_parole = tts.supported_features().is_speaking;

    let mut prochaine: Option<Commande> = None;
    loop {
        let commande = match prochaine.take() {
            Some(c) => c,
            None => match rx.recv() {
                Ok(c) => c,
                Err(_) => return,
            },
        };
        let phrases = match commande {
            Commande::Arreter => {
                let _ = tts.stop();
                emettre(Evenement::Fin);
                continue;
            }
            Commande::Parler(p) => p,
        };
        if let Some(message) = &avertissement {
            emettre(Evenement::Erreur {
                message: message.clone(),
            });
        }
        let _ = tts.stop();

        if !suit_la_parole {
            // Sans suivi de l'état, on lit tout d'un coup.
            emettre(Evenement::Phrase { index: 0 });
            if let Err(e) = tts.speak(phrases.join(" "), true) {
                emettre(Evenement::Erreur {
                    message: e.to_string(),
                });
            }
            continue;
        }

        'phrases: for (index, phrase) in phrases.iter().enumerate() {
            emettre(Evenement::Phrase { index });
            if let Err(e) = tts.speak(phrase.clone(), true) {
                emettre(Evenement::Erreur {
                    message: e.to_string(),
                });
                break;
            }
            let debut = Instant::now();
            let mut a_commence = false;
            loop {
                match rx.recv_timeout(Duration::from_millis(80)) {
                    Ok(c) => {
                        let _ = tts.stop();
                        if matches!(c, Commande::Parler(_)) {
                            prochaine = Some(c);
                        }
                        break 'phrases;
                    }
                    Err(RecvTimeoutError::Disconnected) => return,
                    Err(RecvTimeoutError::Timeout) => {}
                }
                if tts.is_speaking().unwrap_or(false) {
                    a_commence = true;
                } else if a_commence || debut.elapsed() > Duration::from_millis(1500) {
                    break;
                }
            }
        }
        if prochaine.is_none() {
            emettre(Evenement::Fin);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoupe_en_phrases() {
        assert_eq!(
            decouper_phrases("Bonjour. Ça va ?! Oui\nmerci"),
            vec!["Bonjour.", "Ça va ?!", "Oui", "merci"]
        );
        assert!(decouper_phrases("  \n ").is_empty());
    }
}
