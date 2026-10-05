import { listen } from "@tauri-apps/api/event";

type Affichage = { saisie: string; mots: string[] };

const bulle = document.getElementById("bulle") as HTMLDivElement;

function afficher({ mots }: Affichage) {
  bulle.replaceChildren(
    ...mots.map((mot, i) => {
      const choix = document.createElement("span");
      choix.className = "choix";
      const touche = document.createElement("kbd");
      touche.textContent = `Ctrl ${i + 1}`;
      choix.append(touche, mot);
      return choix;
    }),
  );
}

if ("__TAURI_INTERNALS__" in window) {
  listen<Affichage>("overlay", (e) => afficher(e.payload));
} else {
  afficher({ saisie: "fotografi", mots: ["photographie", "photographies", "photographe"] });
}
