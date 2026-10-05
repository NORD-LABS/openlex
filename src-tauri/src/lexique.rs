//! Lexique embarqué : orthographe, phonétique (alphabet Lexique) et fréquence.
//! Données dérivées de Lexique 3.83 (CC BY-SA 4.0) — voir DATA_LICENSE.md.

use std::io::Read;
use std::sync::OnceLock;

use flate2::read::GzDecoder;

const DONNEES: &[u8] = include_bytes!("../data/lexique.tsv.gz");

/// « e » muet / schwa (`°` dans Lexique).
pub const SCHWA: u8 = b'*';
/// Voyelle nasale « on » (`§` dans Lexique).
pub const ON: u8 = b'&';

/// Convertit une phonétique Lexique en octets ASCII (un par phonème).
pub fn encoder_phon(phon: &str) -> Vec<u8> {
    phon.chars()
        .map(|c| match c {
            '°' => SCHWA,
            '§' => ON,
            c if c.is_ascii() => c as u8,
            _ => b'?',
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct Entree {
    pub ortho: String,
    /// Un octet par phonème : alphabet de Lexique (R, E, O, @, 5, 1, 2, 9, S, Z, N, 8…),
    /// sauf `°` → [`SCHWA`] et `§` → [`ON`] pour rester en ASCII.
    pub phon: Vec<u8>,
    /// Occurrences par million (films + livres) × 100.
    pub freq: u32,
}

pub struct Lexique {
    /// Triées par fréquence décroissante.
    pub entrees: Vec<Entree>,
    /// `par_longueur[n]` = indices des entrées dont la phonétique a `n` phonèmes.
    pub par_longueur: Vec<Vec<u32>>,
}

impl Lexique {
    pub fn depuis_tsv(tsv: &str) -> Self {
        let mut entrees = Vec::new();
        for ligne in tsv.lines() {
            let mut c = ligne.split('\t');
            let (Some(ortho), Some(phon), Some(freq)) = (c.next(), c.next(), c.next()) else {
                continue;
            };
            entrees.push(Entree {
                ortho: ortho.to_string(),
                phon: encoder_phon(phon),
                freq: freq.parse().unwrap_or(0),
            });
        }
        let max = entrees.iter().map(|e| e.phon.len()).max().unwrap_or(0);
        let mut par_longueur = vec![Vec::new(); max + 1];
        for (i, e) in entrees.iter().enumerate() {
            par_longueur[e.phon.len()].push(i as u32);
        }
        Lexique {
            entrees,
            par_longueur,
        }
    }

    /// Le lexique embarqué, décompressé au premier appel.
    pub fn global() -> &'static Lexique {
        static LEXIQUE: OnceLock<Lexique> = OnceLock::new();
        LEXIQUE.get_or_init(|| {
            let mut tsv = String::new();
            GzDecoder::new(DONNEES)
                .read_to_string(&mut tsv)
                .expect("lexique embarqué illisible");
            Lexique::depuis_tsv(&tsv)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contient_photographie() {
        let lex = Lexique::global();
        assert!(lex.entrees.len() > 100_000);
        let e = lex
            .entrees
            .iter()
            .find(|e| e.ortho == "photographie")
            .unwrap();
        assert_eq!(e.phon, b"fotogRafi");
    }

    #[test]
    fn index_par_longueur() {
        let lex = Lexique::global();
        for (n, indices) in lex.par_longueur.iter().enumerate() {
            for &i in indices {
                assert_eq!(lex.entrees[i as usize].phon.len(), n);
            }
        }
    }
}
