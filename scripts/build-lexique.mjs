// Construit src-tauri/data/lexique.tsv.gz à partir de Lexique 3.83 (CC BY-SA 4.0).
// Usage : node scripts/build-lexique.mjs [chemin/vers/Lexique383.tsv]
// Sans argument, le fichier est téléchargé depuis lexique.org dans .cache/.
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { gzipSync } from "node:zlib";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const racine = join(dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE = "http://www.lexique.org/databases/Lexique383/Lexique383.tsv";

async function chargerSource() {
  const chemin = process.argv[2] ?? join(racine, ".cache", "Lexique383.tsv");
  if (!existsSync(chemin)) {
    console.log(`Téléchargement de ${SOURCE}…`);
    const rep = await fetch(SOURCE);
    if (!rep.ok) throw new Error(`Téléchargement échoué : HTTP ${rep.status}`);
    mkdirSync(dirname(chemin), { recursive: true });
    writeFileSync(chemin, Buffer.from(await rep.arrayBuffer()));
  }
  return readFileSync(chemin, "utf8");
}

const lignes = (await chargerSource()).split("\n");
const entetes = lignes[0].split("\t");
const col = (nom) => {
  const i = entetes.indexOf(nom);
  if (i < 0) throw new Error(`Colonne manquante : ${nom}`);
  return i;
};
const [iOrtho, iPhon, iFilms, iLivres] = ["ortho", "phon", "freqfilms2", "freqlivres"].map(col);

// Une entrée par couple (ortho, phon) ; on additionne les fréquences des homographes
// (ex. « a » nom + « a » verbe).
const entrees = new Map();
for (const ligne of lignes.slice(1)) {
  if (!ligne.trim()) continue;
  const c = ligne.split("\t");
  const ortho = c[iOrtho];
  const phon = c[iPhon];
  if (!ortho || !phon || /\s/.test(ortho)) continue;
  const freq = (parseFloat(c[iFilms]) || 0) + (parseFloat(c[iLivres]) || 0);
  const cle = `${ortho}\t${phon}`;
  entrees.set(cle, (entrees.get(cle) ?? 0) + freq);
}

const tri = [...entrees].sort((a, b) => b[1] - a[1]);
// Fréquence stockée en centièmes d'occurrence par million, entier.
const sortie = tri.map(([cle, f]) => `${cle}\t${Math.round(f * 100)}`).join("\n") + "\n";

const destination = join(racine, "src-tauri", "data", "lexique.tsv.gz");
mkdirSync(dirname(destination), { recursive: true });
writeFileSync(destination, gzipSync(sortie, { level: 9 }));
console.log(`${tri.length} entrées → ${destination}`);
