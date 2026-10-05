# OpenLex v0.2 — Suggestions automatiques partout (design)

Date : 2026-10-05 · Validé par Théo (choix : Ctrl + 1/2/3, overlay en coin fixe)

## But

Pendant que l'élève écrit dans n'importe quelle application, OpenLex détecte le mot en cours
et, s'il est mal écrit, affiche un petit overlay en bas à droite de l'écran :
`Ctrl+1 photographie · Ctrl+2 photographies · Ctrl+3 photographe`.
Ctrl + chiffre remplace le mot tapé par la suggestion.

## Composants

| Module | Rôle |
|---|---|
| `saisie` | Tampon pur du mot en cours : lettres, apostrophe, trait d'union ; Retour arrière enlève une lettre ; tout le reste vide le tampon. Testé unitairement. |
| `clavier` | Écoute globale, une implémentation par OS, qui émet des `Touche` (Caractère, Effacer, Séparateur, Échap, Ignorer). macOS : `CGEventTap` en écoute seule sur un fil dédié (lettres via `CGEventKeyboardGetUnicodeString`, sans API qui exige le fil principal). Windows : hooks `WH_KEYBOARD_LL` + `WH_MOUSE_LL` (lettres via `ToUnicodeEx` sans modifier l'état du clavier). Les événements synthétiques (les nôtres) sont ignorés. Un clic de souris vide le tampon. |
| `auto` | Coordinateur : reçoit les touches, attend 150 ms de pause, appelle le prédicteur (3 mots), montre/cache l'overlay, active Ctrl+1/2/3 seulement quand l'overlay est visible, fait le remplacement (Retour arrière × longueur, puis frappe du mot, majuscule initiale conservée) sur le fil principal. |
| fenêtre `overlay` | Fenêtre sans bordure, transparente, toujours au-dessus, **non focusable** (ne vole jamais le focus), en bas à droite de la zone de travail de l'écran principal. |

## Règles

- Overlay seulement si le mot a ≥ 3 lettres et n'est pas déjà un mot correct.
- Interrupteur « Suggestions automatiques partout » dans la fenêtre principale (actif par défaut).
- Vie privée : seul le mot en cours est gardé en mémoire ; rien n'est écrit sur disque ni envoyé.
  macOS ne transmet pas les frappes des champs de mot de passe (entrée sécurisée).
- macOS : permission « Surveillance de l'entrée » demandée ; message clair si refusée.

## Hors périmètre (v0.3)

Overlay positionné sous le curseur de texte ; choix du nombre de suggestions.

## Tests

Unitaires : `saisie`. Compilation Windows vérifiée par `cargo check --target x86_64-pc-windows-msvc`
et la CI. Test manuel sur macOS (TextEdit) ; test manuel Windows à faire par un humain.
