//! Convertit un mot écrit « comme il sonne » (ou correctement) en phonétique, dans
//! l'alphabet de Lexique (voir [`crate::lexique`]). Règles simples et volontairement
//! tolérantes : le prédicteur compense les approximations par une distance floue.

use crate::lexique::{ON, SCHWA};

fn est_voyelle(c: char) -> bool {
    matches!(
        c,
        'a' | 'e'
            | 'i'
            | 'o'
            | 'u'
            | 'y'
            | 'é'
            | 'è'
            | 'ê'
            | 'ë'
            | 'à'
            | 'â'
            | 'î'
            | 'ï'
            | 'ô'
            | 'û'
            | 'ù'
            | 'œ'
    )
}

fn est_e_i_y(c: Option<char>) -> bool {
    matches!(c, Some('e' | 'i' | 'y' | 'é' | 'è' | 'ê' | 'ë' | 'î' | 'ï'))
}

/// Garde les lettres (minuscules, accents conservés) ; ignore apostrophes, traits d'union…
fn normaliser(texte: &str) -> Vec<char> {
    texte
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|c| c.is_alphabetic())
        .collect()
}

pub fn phonetiser(texte: &str) -> Vec<u8> {
    let w = normaliser(texte);
    let n = w.len();
    let at = |i: usize| w.get(i).copied();
    let suit = |i: usize, motif: &str| motif.chars().enumerate().all(|(k, c)| at(i + k) == Some(c));
    let voy = |i: usize| at(i).is_some_and(est_voyelle);
    let fin = |i: usize| i >= n;
    // Voyelle nasale : la voyelle (longueur `l` à partir de `i`) est suivie de n/m,
    // eux-mêmes suivis d'une consonne autre que n/m, ou de la fin du mot.
    let nasale = |i: usize, l: usize| {
        matches!(at(i + l), Some('n' | 'm'))
            && !voy(i + l + 1)
            && !matches!(at(i + l + 1), Some('n' | 'm' | 'h'))
    };

    let mut out = Vec::with_capacity(n + 2);
    let mut i = 0;
    while i < n {
        let c = w[i];
        let prec = if i > 0 { Some(w[i - 1]) } else { None };
        let avance = match c {
            'h' => 1,
            'e' => {
                if suit(i, "eau") {
                    out.push(b'o');
                    3
                } else if suit(i, "eill") {
                    out.extend(b"Ej");
                    4
                } else if suit(i, "eil") && fin(i + 3) {
                    out.extend(b"Ej");
                    3
                } else if suit(i, "eu") || suit(i, "eû") {
                    out.push(b'2');
                    2
                } else if suit(i, "ei") && nasale(i, 2) {
                    out.push(b'5');
                    3
                } else if suit(i, "ei") {
                    out.push(b'E');
                    2
                } else if nasale(i, 1) {
                    out.push(b'@');
                    2
                } else if n > 2 && fin(i + 2) && matches!(at(i + 1), Some('r' | 'z')) {
                    out.push(b'e');
                    2
                } else if n > 2 && fin(i + 2) && at(i + 1) == Some('t') {
                    out.push(b'E');
                    2
                } else if fin(i + 2) && at(i + 1) == Some('s') {
                    if n <= 3 {
                        out.push(b'e');
                    }
                    2
                } else if fin(i + 1) {
                    if n <= 2 {
                        out.push(SCHWA);
                    }
                    1
                } else if at(i + 1) == Some('x')
                    || (!voy(i + 1) && (!voy(i + 2) || fin(i + 2)) && at(i + 2) != Some('h'))
                {
                    out.push(b'E');
                    1
                } else if i == 0 {
                    // en début de mot, « e » se prononce presque toujours é (ecole, elefan)
                    out.push(b'e');
                    1
                } else {
                    out.push(SCHWA);
                    1
                }
            }
            'é' => {
                out.push(b'e');
                1
            }
            'è' | 'ê' | 'ë' => {
                out.push(b'E');
                1
            }
            'a' | 'à' | 'â' => {
                if suit(i, "aient") && fin(i + 5) {
                    out.push(b'E');
                    5
                } else if suit(i, "aill") {
                    out.extend(b"aj");
                    4
                } else if suit(i, "ail") && fin(i + 3) {
                    out.extend(b"aj");
                    3
                } else if suit(i, "ai") && nasale(i, 2) {
                    out.push(b'5');
                    3
                } else if suit(i, "ai") || suit(i, "aî") {
                    out.push(b'E');
                    2
                } else if suit(i, "au") {
                    out.push(b'o');
                    2
                } else if suit(i, "ay") && voy(i + 2) {
                    out.extend(b"Ej");
                    2
                } else if nasale(i, 1) {
                    out.push(b'@');
                    2
                } else {
                    out.push(b'a');
                    1
                }
            }
            'o' | 'ô' => {
                if suit(i, "oin") && nasale(i, 2) {
                    out.extend(b"w5");
                    3
                } else if suit(i, "oi") || suit(i, "oî") || suit(i, "oy") {
                    out.extend(b"wa");
                    2
                } else if suit(i, "ou") || suit(i, "oû") || suit(i, "où") {
                    out.push(b'u');
                    2
                } else if nasale(i, 1) {
                    out.push(ON);
                    2
                } else {
                    out.push(b'o');
                    1
                }
            }
            'œ' => {
                out.push(b'9');
                if at(i + 1) == Some('u') {
                    2
                } else {
                    1
                }
            }
            'u' | 'û' | 'ù' => {
                if nasale(i, 1) {
                    out.push(b'1');
                    2
                } else if at(i + 1) == Some('i') {
                    out.extend(b"8i");
                    2
                } else {
                    out.push(b'y');
                    1
                }
            }
            'i' | 'î' | 'ï' => {
                if suit(i, "ill") {
                    if i == 0 {
                        out.extend(b"il");
                    } else if prec.is_some_and(est_voyelle) {
                        out.push(b'j');
                    } else {
                        out.extend(b"ij");
                    }
                    3
                } else if suit(i, "ien") && nasale(i + 1, 1) {
                    out.extend(b"j5");
                    3
                } else if nasale(i, 1) {
                    out.push(b'5');
                    2
                } else if voy(i + 1)
                    && prec.is_some_and(|p| !est_voyelle(p))
                    // « ie » final : le e est muet (pharmacie)
                    && !(at(i + 1) == Some('e') && (fin(i + 2) || (at(i + 2) == Some('s') && fin(i + 3))))
                {
                    out.push(b'j');
                    1
                } else {
                    out.push(b'i');
                    1
                }
            }
            'y' => {
                if nasale(i, 1) {
                    out.push(b'5');
                    2
                } else if voy(i + 1) && (i == 0 || prec.is_some_and(est_voyelle)) {
                    out.push(b'j');
                    1
                } else {
                    out.push(b'i');
                    1
                }
            }
            'c' => {
                if suit(i, "ch") {
                    out.push(b'S');
                    2
                } else if suit(i, "ck") {
                    out.push(b'k');
                    2
                } else if suit(i, "cc") && est_e_i_y(at(i + 2)) {
                    out.extend(b"ks");
                    2
                } else if suit(i, "cc") {
                    out.push(b'k');
                    2
                } else if est_e_i_y(at(i + 1)) {
                    out.push(b's');
                    1
                } else {
                    out.push(b'k');
                    1
                }
            }
            'ç' => {
                out.push(b's');
                1
            }
            'g' => {
                if suit(i, "gn") {
                    out.push(b'N');
                    2
                } else if suit(i, "gu") && est_e_i_y(at(i + 2)) {
                    out.push(b'g');
                    2
                } else if at(i + 1) == Some('e') && matches!(at(i + 2), Some('a' | 'o' | 'u')) {
                    out.push(b'Z');
                    2
                } else if est_e_i_y(at(i + 1)) {
                    out.push(b'Z');
                    1
                } else if fin(i + 1) && n > 2 {
                    1
                } else {
                    out.push(b'g');
                    if at(i + 1) == Some('g') {
                        2
                    } else {
                        1
                    }
                }
            }
            'j' => {
                out.push(b'Z');
                1
            }
            'p' => {
                if suit(i, "ph") {
                    out.push(b'f');
                    2
                } else if fin(i + 1) && n > 2 {
                    1
                } else {
                    out.push(b'p');
                    if at(i + 1) == Some('p') {
                        2
                    } else {
                        1
                    }
                }
            }
            'q' => {
                out.push(b'k');
                if at(i + 1) == Some('u') {
                    2
                } else {
                    1
                }
            }
            's' => {
                if suit(i, "sch") {
                    out.push(b'S');
                    3
                } else if suit(i, "sh") {
                    out.push(b'S');
                    2
                } else if suit(i, "ss") {
                    out.push(b's');
                    2
                } else if fin(i + 1) && n > 2 {
                    1
                } else if i > 0 && prec.is_some_and(est_voyelle) && voy(i + 1) {
                    out.push(b'z');
                    1
                } else {
                    out.push(b's');
                    1
                }
            }
            't' => {
                if suit(i, "tion") && prec != Some('s') && i > 0 {
                    out.extend(b"sj");
                    out.push(ON);
                    4
                } else if suit(i, "th") {
                    out.push(b't');
                    2
                } else if fin(i + 1) && n > 2 {
                    1
                } else {
                    out.push(b't');
                    if at(i + 1) == Some('t') {
                        2
                    } else {
                        1
                    }
                }
            }
            'd' => {
                if fin(i + 1) && n > 2 {
                    1
                } else {
                    out.push(b'd');
                    if at(i + 1) == Some('d') {
                        2
                    } else {
                        1
                    }
                }
            }
            'x' => {
                if fin(i + 1) && n > 2 {
                    // muet en fin de mot (ciseaux, prix)
                } else if i == 1 && prec == Some('e') && voy(i + 1) {
                    out.extend(b"gz");
                } else {
                    out.extend(b"ks");
                }
                1
            }
            'r' => {
                out.push(b'R');
                if at(i + 1) == Some('r') {
                    2
                } else {
                    1
                }
            }
            'k' | 'b' | 'f' | 'l' | 'm' | 'n' | 'v' | 'w' | 'z' => {
                out.push(c as u8);
                if at(i + 1) == Some(c) {
                    2
                } else {
                    1
                }
            }
            _ => 1,
        };
        i += avance;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexique::encoder_phon;

    fn verifier(cas: &[(&str, &str)]) {
        let mut erreurs = Vec::new();
        for (mot, attendu) in cas {
            let obtenu = phonetiser(mot);
            if obtenu != encoder_phon(attendu) {
                erreurs.push(format!(
                    "{mot} → {} (attendu {attendu})",
                    String::from_utf8_lossy(&obtenu)
                ));
            }
        }
        assert!(erreurs.is_empty(), "{}", erreurs.join("\n"));
    }

    #[test]
    fn mots_bien_ecrits() {
        verifier(&[
            ("photographie", "fotogRafi"),
            ("bateau", "bato"),
            ("encore", "@koR"),
            ("champignon", "S@piN§"),
            ("famille", "famij"),
            ("maison", "mEz§"),
            ("garçon", "gaRs§"),
            ("oiseau", "wazo"),
            ("pharmacie", "faRmasi"),
            ("aujourd'hui", "oZuRd8i"),
            ("école", "ekol"),
            ("éléphant", "elef@"),
            ("soleil", "solEj"),
            ("travail", "tRavaj"),
            ("citron", "sitR§"),
            ("quatre", "katR"),
            ("question", "kEstj§"),
            ("nation", "nasj§"),
            ("jardin", "ZaRd5"),
            ("pain", "p5"),
            ("lundi", "l1di"),
            ("grenouille", "gR°nuj"),
            ("les", "le"),
            ("chat", "Sa"),
            ("rouge", "RuZ"),
            ("manger", "m@Ze"),
            ("fille", "fij"),
            ("hôpital", "opital"),
            ("œuf", "9f"),
            ("ciseaux", "sizo"),
            ("girafe", "ZiRaf"),
            ("guitare", "gitaR"),
            ("je", "Z°"),
        ]);
    }

    #[test]
    fn mots_ecrits_au_son() {
        verifier(&[
            ("fotografi", "fotogRafi"),
            ("bato", "bato"),
            ("ankor", "@koR"),
            ("chanpignon", "S@piN§"),
            ("ojourdui", "oZuRd8i"),
            ("kestion", "kEstj§"),
            ("sizo", "sizo"),
            ("jiraf", "ZiRaf"),
            ("gato", "gato"),
            ("elefan", "el°f@"),
            ("ecol", "ekol"),
            ("oto", "oto"),
        ]);
    }

    #[test]
    fn ignore_ponctuation_et_majuscules() {
        assert_eq!(phonetiser("Bateau!"), phonetiser("bateau"));
        assert!(phonetiser("").is_empty());
        assert!(phonetiser("123 ?").is_empty());
    }
}
