//! Suit le mot en cours de frappe, à partir des touches captées partout dans le système.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Touche {
    Caractere(char),
    /// Retour arrière.
    Effacer,
    /// Espace, Entrée, flèches, clic, raccourci… : le mot en cours est terminé.
    Separateur,
    Echap,
    /// Touche sans effet sur le mot (Maj seule, touche morte…).
    Ignorer,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Changement {
    Rien,
    /// Le mot en cours a changé.
    Mot,
    /// Le mot en cours est terminé ou abandonné.
    Vide,
}

#[derive(Default)]
pub struct Tampon {
    mot: String,
}

fn fait_partie_du_mot(c: char) -> bool {
    c.is_alphabetic() || matches!(c, '\'' | '’' | '-')
}

impl Tampon {
    pub fn mot(&self) -> &str {
        &self.mot
    }

    /// Nombre de caractères à effacer pour remplacer le mot.
    pub fn longueur(&self) -> usize {
        self.mot.chars().count()
    }

    pub fn vider(&mut self) {
        self.mot.clear();
    }

    pub fn traiter(&mut self, touche: Touche) -> Changement {
        match touche {
            Touche::Ignorer => Changement::Rien,
            Touche::Caractere(c) if fait_partie_du_mot(c) => {
                self.mot.push(c);
                Changement::Mot
            }
            Touche::Effacer if !self.mot.is_empty() => {
                self.mot.pop();
                if self.mot.is_empty() {
                    Changement::Vide
                } else {
                    Changement::Mot
                }
            }
            _ => {
                self.mot.clear();
                Changement::Vide
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taper(t: &mut Tampon, texte: &str) {
        for c in texte.chars() {
            t.traiter(Touche::Caractere(c));
        }
    }

    #[test]
    fn suit_le_mot() {
        let mut t = Tampon::default();
        taper(&mut t, "fotografi");
        assert_eq!(t.mot(), "fotografi");
        assert_eq!(t.traiter(Touche::Effacer), Changement::Mot);
        assert_eq!(t.mot(), "fotograf");
        assert_eq!(t.longueur(), 8);
    }

    #[test]
    fn espace_et_ponctuation_terminent_le_mot() {
        let mut t = Tampon::default();
        taper(&mut t, "le ");
        assert_eq!(t.mot(), "");
        taper(&mut t, "chat,");
        assert_eq!(t.mot(), "");
        taper(&mut t, "aujourd'hui");
        assert_eq!(t.mot(), "aujourd'hui");
        assert_eq!(t.traiter(Touche::Separateur), Changement::Vide);
        assert_eq!(t.mot(), "");
    }

    #[test]
    fn accents_et_majuscules() {
        let mut t = Tampon::default();
        taper(&mut t, "Éléfan");
        assert_eq!(t.mot(), "Éléfan");
        assert_eq!(t.longueur(), 6);
    }

    #[test]
    fn effacer_jusqu_au_vide() {
        let mut t = Tampon::default();
        taper(&mut t, "a");
        assert_eq!(t.traiter(Touche::Effacer), Changement::Vide);
        assert_eq!(t.traiter(Touche::Effacer), Changement::Vide);
        assert_eq!(t.traiter(Touche::Ignorer), Changement::Rien);
    }
}
