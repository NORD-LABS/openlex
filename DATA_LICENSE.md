# Licence des données

Le fichier `src-tauri/data/lexique.tsv.gz` est dérivé de **Lexique 3.83**
(<http://www.lexique.org>) : colonnes `ortho`, `phon` et somme des fréquences
`freqfilms2` + `freqlivres`, dédoublonnées par couple (ortho, phon).

> New, B., Pallier, C., Brysbaert, M., Ferrand, L. (2004). Lexique 2 : A New French
> Lexical Database. *Behavior Research Methods, Instruments, & Computers*, 36(3), 516-524.
>
> New, B., Pallier, C., Ferrand, L., Matos, R. (2001). Une base de données lexicales du
> français contemporain sur internet : LEXIQUE. *L'Année Psychologique*, 101, 447-462.

Lexique est distribué sous licence **Creative Commons Attribution – Partage dans les
mêmes conditions 4.0 International (CC BY-SA 4.0)**
(<https://creativecommons.org/licenses/by-sa/4.0/deed.fr>). Les données dérivées
ci-dessus sont distribuées sous la même licence.

Pour régénérer le fichier : `node scripts/build-lexique.mjs`.

Le code source d'OpenLex est sous licence GPL-3.0 (voir `LICENSE`).
