# OpenLex

**Aide libre et gratuite pour lire et écrire**, pensée pour les élèves dyslexiques et
dysorthographiques du Québec. OpenLex est une petite fenêtre qui reste par-dessus Word,
Google Docs ou n'importe quelle application.

> Projet de NORD LABS, en version prototype (v0.2). Ce n'est pas un produit de Haylem et il
> n'est pas affilié à Lexibar (Lexibar est une marque de Haylem).

## Ce que ça fait

- **✨ Suggestions automatiques partout** (nouveau en v0.2) : écris dans Word, Google Docs,
  Messages… Quand un mot est mal écrit, une petite bulle apparaît en bas à droite de l'écran :
  `Ctrl 1 photographie · Ctrl 2 photographies · Ctrl 3 photographe`. Fais
  <kbd>Ctrl</kbd> + <kbd>1</kbd>, <kbd>2</kbd> ou <kbd>3</kbd> et ton mot est corrigé. Tu peux
  désactiver la fonction dans la fenêtre OpenLex.
- **🔮 Prédiction phonétique** : tu écris le mot comme tu l'entends (`fotografi`, `ankor`,
  `ojourdui`) et OpenLex propose la bonne orthographe (*photographie*, *encore*,
  *aujourd'hui*). Un clic, ou le chiffre du mot, l'écrit directement dans ton texte.
- **🔊 Synthèse vocale** : OpenLex lit ton texte à voix haute et surligne la phrase en cours.
  Tu peux coller le texte dans OpenLex, ou **sélectionner du texte dans Word ou Docs** et faire
  <kbd>Ctrl</kbd> + <kbd>Maj</kbd> + <kbd>L</kbd> (<kbd>⌘</kbd> + <kbd>Maj</kbd> + <kbd>L</kbd> sur Mac).
- **🔒 100 % hors ligne** : aucun compte, aucun réseau, aucune donnée envoyée. Ça marche en mode
  avion. Les voix utilisées sont celles déjà installées sur l'ordinateur.

### Vie privée des suggestions automatiques

Pour voir le mot que tu écris, OpenLex écoute le clavier de tout l'ordinateur. Il garde
**seulement le mot en cours, en mémoire** : rien n'est enregistré sur le disque, rien n'est
envoyé. Le code est ouvert, donc tout le monde peut le vérifier (`src-tauri/src/clavier/`).
Sur macOS, les champs de mot de passe ne transmettent pas les frappes. Certains antivirus
peuvent quand même signaler ce type d'écoute.

## Installer

Télécharge la dernière version dans
[Releases](https://github.com/NORD-LABS/openlex/releases) :

| Système | Fichier |
|---|---|
| Windows 10/11 | `OpenLex_x.y.z_x64-setup.exe` ou `.msi` |
| macOS (Intel et Apple Silicon) | `OpenLex_x.y.z_universal.dmg` |

L'app n'est **pas encore signée** :
- **Windows** : si « Windows a protégé votre ordinateur » s'affiche, clique sur
  *Informations complémentaires* › *Exécuter quand même*.
- **macOS** : la première fois, fais clic droit › *Ouvrir*. Ensuite, dans *Réglages Système ›
  Confidentialité et sécurité*, autorise OpenLex dans **Accessibilité** (pour écrire les mots)
  et dans **Surveillance de l'entrée** (pour les suggestions automatiques), puis relance OpenLex.

**À l'école** : un élève n'installe pas de logiciel lui-même. Montre OpenLex à ton
orthopédagogue ou à ton enseignant·e. L'installation se fait avec le soutien technique de l'école.

**Voix française** : Windows inclut des voix françaises. Si OpenLex dit qu'il n'en trouve pas,
ajoute le français dans *Paramètres › Heure et langue › Voix*.

## Comment ça marche

```
Fenêtre + overlay (TypeScript + Vite)
        │ commandes Tauri
Cœur Rust
 ├─ clavier      écoute globale (CGEventTap / hooks Windows) → touches
 ├─ saisie       mot en cours (rien d'autre n'est gardé)
 ├─ auto         pause de 150 ms → 3 suggestions → overlay → Ctrl+1/2/3 remplace le mot
 ├─ lexique      125 000 mots : orthographe, phonétique, fréquence (Lexique 3.83)
 ├─ phonetiseur  « fotografi » → /fotogRafi/ (règles du français)
 ├─ predicteur   distance phonétique pondérée + début de mot, triés par fréquence
 ├─ voix         voix du système, phrase par phrase
 └─ pont         sélection et collage par le presse-papier (toujours restauré)
```

Le pont passe par le presse-papier et des raccourcis clavier simulés. OpenLex marche donc
dans n'importe quelle application (Word, Google Docs, navigateur…) sans s'y intégrer.

## Développer

Prérequis : [Rust](https://rustup.rs), Node.js 20+, et les
[prérequis Tauri](https://tauri.app/start/prerequisites/).

```bash
npm install
npm run tauri dev        # lancer l'app
cd src-tauri && cargo test --release   # tests du cœur
npm run tauri build      # installateur pour ton système
```

Régénérer les données du lexique : `node scripts/build-lexique.mjs`.

## Feuille de route

- Overlay positionné sous le curseur de texte
- Surlignage mot à mot pendant la lecture
- Réglages : vitesse de lecture, choix de la voix, taille du texte
- Illustrations des mots (pictogrammes libres)
- Version Chromebook
- Tests avec des élèves et des orthopédagogues

## Licences et crédits

- **Code** : [GPL-3.0](LICENSE).
- **Données** : dérivées de [Lexique 3.83](http://www.lexique.org) (New, Pallier et coll.),
  sous licence [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/deed.fr).
  Voir [DATA_LICENSE.md](DATA_LICENSE.md).
- **Transparence** : la majorité du code d'implémentation a été générée par des agents IA
  (Claude), sous la direction de Théodore Beaupré, qui a fait la conception et la revue.
