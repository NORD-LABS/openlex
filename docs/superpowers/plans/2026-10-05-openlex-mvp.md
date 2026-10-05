# OpenLex MVP — Plan d'implémentation

Spec : `docs/superpowers/specs/2026-10-05-openlex-design.md`
Repo local : `/Users/theodorebeaupre/OPENCLICKY/openlex` · Cible : `NORD-LABS/openlex` (public)

Cocher chaque tâche quand elle est vérifiée. Commit après chaque tâche.

- [x] **T1 Scaffold** — Tauri 2 + Vite vanilla-ts (`npm create tauri-app`), identifiant
  `ca.nordlabs.openlex`, nom produit « OpenLex ». `.gitignore`, `LICENSE` (GPL-3.0).
  Vérif : `npm run build` et `cargo check` dans `src-tauri` passent.
- [x] **T2 Données** — `scripts/build-lexique.mjs` (ou Python) : télécharger Lexique383.tsv
  depuis lexique.org, garder `ortho, phon, freq = freqfilms2 + freqlivres`, dédoublonner
  (ortho, phon), trier par freq desc, écrire `src-tauri/data/lexique.tsv.gz`.
  `DATA_LICENSE.md` (CC BY-SA 4.0, attribution New & Pallier). Ne pas committer le TSV brut.
- [x] **T3 `lexique`** — charge le gz embarqué (`include_bytes!` + `flate2`) en `Vec<Entree>`
  + index `HashMap<phon, Vec<idx>>`. Test : « photographie » présent avec phon `fotogRafi`.
- [x] **T4 `phonetiseur`** — règles graphème→phonème vers l'alphabet Lexique
  (R, E, O, @, §, 5, 1, 2, 9, °, S, Z, N, G, j, w, 8…). Gérer ph, qu, gu, ch, eau/au/o,
  ai/ei/è/ê, an/en/am/em, on/om, in/im/ain/ein/un, ou, oi, gn, ill, c/g doux, s entre voyelles,
  lettres finales muettes, doubles consonnes, h muet. TDD : ≥ 25 cas.
- [x] **T5 `predicteur`** — `suggerer(saisie, n)` : phonétiser ; candidats = index exact
  + distance de Levenshtein pondérée (voyelles proches e/E/°, o/O, 2/9, a/@ coût 0.5) ≤ 2
  sur phon ; + préfixes orthographiques ; score = distance puis freq ; dédoublonner par ortho.
  Doit rester < 50 ms sur 140k entrées (pré-filtrage par longueur ± 2). Tests : fotografi→
  photographie top 3 ; oto→auto ; ankor→encore ; bato→bateau ; fami→famille.
- [x] **T6 `voix`** — crate `tts` : `parler(texte)`, `arreter()`, choisir une voix fr
  (fr-CA en priorité, sinon fr-*). Commandes Tauri. Erreur claire si aucune voix FR.
- [x] **T7 `pont`** — `lire_selection()` : sauvegarder presse-papier, simuler Ctrl/Cmd+C
  (`enigo`), attendre ~120 ms, lire, restaurer. `coller_mot(mot)` : cacher/défocaliser la
  fenêtre OpenLex, coller via presse-papier, restaurer. Raccourci global
  `CmdOrCtrl+Shift+L` → lire la sélection (`tauri-plugin-global-shortcut`).
- [x] **T8 UI** — fenêtre 380×520, toujours au-dessus, grande police lisible, fort contraste,
  thème clair/sombre. Champ « Écris le mot comme tu l'entends », liste de suggestions
  (chiffres 1–9, bouton 🔊 par mot, clic = coller), zone « Lire » (texte collé ou
  sélection), boutons Lire / Arrêter, messages d'erreur doux. Tout en français.
- [x] **T9 Qualité** — `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt`,
  `npm run build`, `npm run tauri build` (macOS local). README FR (but, captures facultatives,
  installation, raccourcis, vie privée, licences, crédits Lexique, « code majoritairement
  généré par agents IA »).
- [x] **T10 CI** — `.github/workflows/ci.yml` (tests sur ubuntu) et `release.yml`
  (`tauri-apps/tauri-action`, windows-latest + macos-latest, sur tag `v*`, release brouillon
  → publiée).
- [ ] **T11 Publication** — `gh repo create NORD-LABS/openlex --public`, push `main`,
  tag `v0.1.0`, vérifier que le workflow release passe ; corriger jusqu'au vert ; publier la
  release. Notifier Théo.
