//! Prédiction : à partir d'une saisie « au son » (ou d'un début de mot), propose les
//! orthographes correctes, triées par proximité sonore puis par fréquence.

use std::collections::HashMap;

use serde::Serialize;

use crate::lexique::{Lexique, ON, SCHWA};
use crate::phonetiseur::phonetiser;

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub mot: String,
    /// 0 = sonne pareil ; plus c'est grand, plus c'est éloigné.
    pub score: f32,
}

/// Les coûts sont en demi-unités : 2 = une vraie différence, 1 = sons très proches.
const PLEIN: u16 = 2;
const DEMI: u16 = 1;

/// Voyelles que la plupart des élèves (et des accents québécois) ne distinguent pas :
/// les confondre ne coûte rien.
fn equivalent(a: u8, b: u8) -> bool {
    matches!(
        (a, b),
        (b'e', b'E') | (b'E', b'e') | (b'o', b'O') | (b'O', b'o') | (b'2', b'9') | (b'9', b'2')
    )
}

fn groupe(p: u8) -> u8 {
    match p {
        b'e' | b'E' | b'2' | b'9' | SCHWA => 1,
        b'o' | b'O' => 2,
        b'5' | b'1' => 4,
        b'i' | b'j' => 5,
        b'u' | b'w' => 6,
        b'y' | b'8' => 7,
        b's' | b'z' => 8,
        b'@' | ON => 9,
        _ => 0,
    }
}

fn cout_substitution(a: u8, b: u8) -> u16 {
    if a == b || equivalent(a, b) {
        0
    } else if groupe(a) != 0 && groupe(a) == groupe(b) {
        DEMI
    } else {
        PLEIN
    }
}

fn cout_indel(p: u8) -> u16 {
    if p == SCHWA {
        DEMI
    } else {
        PLEIN
    }
}

/// Distance de Levenshtein pondérée, abandonnée dès qu'elle dépasse `limite`.
fn distance(a: &[u8], b: &[u8], limite: u16, ligne: &mut Vec<u16>) -> Option<u16> {
    ligne.clear();
    ligne.push(0);
    for &cb in b {
        let dernier = *ligne.last().unwrap();
        ligne.push(dernier + cout_indel(cb));
    }
    for &ca in a {
        let mut diag = ligne[0];
        ligne[0] += cout_indel(ca);
        let mut min_ligne = ligne[0];
        for (j, &cb) in b.iter().enumerate() {
            let haut = ligne[j + 1];
            let v = (diag + cout_substitution(ca, cb))
                .min(haut + cout_indel(ca))
                .min(ligne[j] + cout_indel(cb));
            diag = haut;
            ligne[j + 1] = v;
            min_ligne = min_ligne.min(v);
        }
        if min_ligne > limite {
            return None;
        }
    }
    let d = *ligne.last().unwrap();
    (d <= limite).then_some(d)
}

fn garder<'a>(meilleurs: &mut HashMap<&'a str, (i32, u32)>, mot: &'a str, score: i32, freq: u32) {
    let s = meilleurs.entry(mot).or_insert((score, freq));
    if score < s.0 {
        s.0 = score;
    }
}

pub fn suggerer(lexique: &Lexique, saisie: &str, nombre: usize) -> Vec<Suggestion> {
    let saisie = saisie.trim().to_lowercase();
    let phon = phonetiser(&saisie);
    if phon.is_empty() {
        return Vec::new();
    }

    let limite = match phon.len() {
        0..=3 => 2,
        4..=6 => 3,
        _ => 4,
    };
    let ecart = (limite / DEMI) as usize;

    // ortho → (score en demi-unités, fréquence) ; on garde le meilleur score par mot.
    let mut meilleurs: HashMap<&str, (i32, u32)> = HashMap::new();

    let mut ligne = Vec::with_capacity(32);
    let min = phon.len().saturating_sub(ecart);
    let max = (phon.len() + ecart).min(lexique.par_longueur.len().saturating_sub(1));
    for longueur in min..=max {
        for &i in &lexique.par_longueur[longueur] {
            let e = &lexique.entrees[i as usize];
            if let Some(d) = distance(&phon, &e.phon, limite, &mut ligne) {
                garder(&mut meilleurs, &e.ortho, d as i32, e.freq);
            }
        }
    }

    // Complétion par début de mot (l'élève connaît le début de l'orthographe).
    let longueur_saisie = saisie.chars().count();
    if longueur_saisie >= 3 {
        for e in lexique
            .entrees
            .iter()
            .filter(|e| e.ortho.starts_with(&saisie))
        {
            let score = if e.ortho == saisie { -2 } else { 1 };
            garder(&mut meilleurs, &e.ortho, score, e.freq);
        }
    }

    let mut liste: Vec<_> = meilleurs.into_iter().collect();
    liste.sort_by(|a, b| a.1 .0.cmp(&b.1 .0).then(b.1 .1.cmp(&a.1 .1)));
    liste
        .into_iter()
        .take(nombre)
        .map(|(mot, (score, _))| Suggestion {
            mot: mot.to_string(),
            score: score.max(0) as f32 / PLEIN as f32,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mots(saisie: &str) -> Vec<String> {
        suggerer(Lexique::global(), saisie, 5)
            .into_iter()
            .map(|s| s.mot)
            .collect()
    }

    fn dans_top(saisie: &str, attendu: &str, top: usize) {
        let m = mots(saisie);
        assert!(
            m.iter().take(top).any(|x| x == attendu),
            "{saisie} : « {attendu} » absent du top {top} : {m:?}"
        );
    }

    #[test]
    fn mots_au_son() {
        dans_top("fotografi", "photographie", 3);
        dans_top("oto", "auto", 3);
        dans_top("ankor", "encore", 3);
        dans_top("bato", "bateau", 3);
        dans_top("fami", "famille", 3);
        dans_top("farmasi", "pharmacie", 3);
        dans_top("chanpignon", "champignon", 3);
        dans_top("ojourdui", "aujourd'hui", 3);
        dans_top("wazo", "oiseau", 3);
        dans_top("sizo", "ciseaux", 3);
        dans_top("elefan", "éléphant", 3);
        dans_top("ecol", "école", 3);
        dans_top("pome", "pomme", 1);
    }

    #[test]
    fn mot_correct_en_premier() {
        assert_eq!(mots("maison")[0], "maison");
    }

    #[test]
    fn saisie_vide() {
        assert!(mots("").is_empty());
        assert!(mots("  ").is_empty());
    }

    #[test]
    fn distance_ponderee() {
        let mut l = Vec::new();
        assert_eq!(distance(b"oto", b"oto", 4, &mut l), Some(0));
        assert_eq!(distance(b"@koR", b"@kOR", 4, &mut l), Some(0));
        assert_eq!(distance(b"*l*f@", b"elef@", 4, &mut l), Some(2 * DEMI));
        assert_eq!(distance(b"fami", b"famij", 4, &mut l), Some(PLEIN));
        assert_eq!(distance(b"abc", b"xyz", 2, &mut l), None);
    }
}
