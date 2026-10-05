# OpenLex — Design (MVP v0.1.0)

Date : 2026-10-05 · Auteur : Théodore Beaupré (NORD LABS)

## But

Alternative **libre et gratuite** à Lexibar : aide à la lecture et à l'écriture pour élèves
dyslexiques / dysorthographiques du Québec. Cible à long terme : écoles, centres de services
scolaires, ministère de l'Éducation. Le MVP doit être présentable à un·e orthopédagogue.

## Contraintes (validées avec Théo)

- **Windows d'abord** (parc scolaire majoritaire), Chromebook ensuite (hors MVP), macOS pour le dev.
- **100 % hors ligne** : aucun réseau, aucun compte, aucune télémétrie (Loi 25, élèves mineurs,
  mode avion en classe).
- Doit fonctionner **dans Word et dans Google Docs** sans s'y intégrer directement.
- L'élève ne peut pas installer lui-même à l'école → l'installation passe par l'école.

## Fonctions du MVP

1. **Synthèse vocale** : lire le texte sélectionné (raccourci global) ou le texte du champ, avec
   les voix françaises du système (hors ligne). Surlignage de la phrase en cours (v1) ; surlignage
   mot à mot plus tard si l'API le permet.
2. **Prédiction phonétique** : l'élève tape un mot comme il le prononce (« fotografi ») et OpenLex
   propose les bonnes orthographes (« photographie »), triées par proximité sonore puis fréquence.
   Complète aussi par préfixe orthographique. Un clic (ou chiffre 1–9) colle le mot dans
   l'application active.

Hors MVP (YAGNI) : illustrations, correcteur, extension Chrome, iPad.

## Architecture

App de bureau **Tauri 2** : fenêtre flottante « toujours au-dessus », UI en TypeScript + Vite
(sans framework), cœur en Rust.

Modules Rust, une responsabilité chacun :

| Module | Rôle | Dépend de |
|---|---|---|
| `lexique` | Charge le lexique embarqué (orthographe, phonétique, fréquence) | données |
| `phonetiseur` | Convertit une saisie « au son » en chaîne phonétique Lexique | — |
| `predicteur` | Cherche les mots proches phonétiquement + complétion par préfixe | lexique, phonetiseur |
| `voix` | Parler / arrêter, voix FR du système | crate `tts` |
| `pont` | Lire la sélection (copie simulée) et coller un mot ; restaure le presse-papier | `enigo`, `arboard` |

`predicteur` et `phonetiseur` sont purs et testés unitairement sans UI.

## Données

**Lexique 3.83** (lexique.org, New & Pallier), licence **CC BY-SA 4.0**. Un script de build
extrait `ortho`, `phon`, fréquence (films + livres), dédoublonne, et produit un TSV compressé
embarqué dans le binaire. Les données dérivées restent sous CC BY-SA 4.0, avec attribution dans
`DATA_LICENSE.md` et le README. Code : **GPL-3.0**.

## Gestion d'erreurs

- Aucune voix française trouvée → message clair dans l'UI, la prédiction reste utilisable.
- Sélection vide / copie échouée → message « Sélectionne du texte d'abord ».
- Presse-papier toujours restauré après lecture/collage.

## Tests

- Unitaires Rust : phonétiseur (cas de règles), prédicteur (« fotografi » → « photographie »,
  « oto » → « auto »/« haut »…, classement par fréquence).
- `cargo clippy -D warnings`, build front, `tauri build` sur macOS en local.
- CI GitHub Actions : tests + builds Windows (.msi/.exe) et macOS ; release sur tag `v*`.

## Livraison

Dépôt public `NORD-LABS/openlex`, tag `v0.1.0`, release GitHub avec installateur Windows.
