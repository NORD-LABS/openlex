import * as api from "./api";

const $ = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const saisie = $<HTMLInputElement>("saisie");
const liste = $<HTMLOListElement>("suggestions");
const texte = $<HTMLTextAreaElement>("texte");
const lecture = $<HTMLDivElement>("lecture");
const message = $<HTMLParagraphElement>("message");

let suggestions: api.Suggestion[] = [];
let requete = 0;

function afficherMessage(t: string, erreur = false) {
  message.textContent = t;
  message.classList.toggle("erreur", erreur);
}

async function mettreAJourSuggestions() {
  const n = ++requete;
  const valeur = saisie.value.trim();
  const resultats = valeur ? await api.suggerer(valeur) : [];
  if (n !== requete) return; // une frappe plus récente a déjà répondu
  suggestions = resultats;
  liste.replaceChildren(
    ...resultats.map((s, i) => {
      const li = document.createElement("li");
      const choisir = document.createElement("button");
      choisir.className = "mot";
      choisir.innerHTML = `<span class="chiffre">${i + 1}</span>`;
      choisir.append(s.mot);
      choisir.title = "Écrire ce mot dans mon texte";
      choisir.onclick = () => choisirMot(s.mot);
      const ecouter = document.createElement("button");
      ecouter.className = "ecouter";
      ecouter.textContent = "🔊";
      ecouter.setAttribute("aria-label", `Écouter « ${s.mot} »`);
      ecouter.onclick = () => api.lire(s.mot);
      li.append(choisir, ecouter);
      return li;
    }),
  );
  if (valeur && !resultats.length) afficherMessage("Aucun mot trouvé. Essaie de l'écrire autrement.");
  else afficherMessage("");
}

async function choisirMot(mot: string) {
  try {
    await api.inserer(mot);
    afficherMessage(`« ${mot} » ajouté à ton texte.`);
    saisie.value = "";
    await mettreAJourSuggestions();
  } catch (e) {
    afficherMessage(String(e), true);
  }
}

function montrerPhrases(phrases: string[]) {
  lecture.replaceChildren(
    ...phrases.map((p) => {
      const span = document.createElement("span");
      span.textContent = p + " ";
      return span;
    }),
  );
  lecture.hidden = phrases.length === 0;
}

saisie.addEventListener("input", mettreAJourSuggestions);
saisie.addEventListener("keydown", (e) => {
  const chiffre = Number(e.key);
  if (chiffre >= 1 && chiffre <= suggestions.length) {
    e.preventDefault();
    choisirMot(suggestions[chiffre - 1].mot);
  } else if (e.key === "Enter" && suggestions[0]) {
    choisirMot(suggestions[0].mot);
  }
});

$("lire").onclick = async () => {
  if (!texte.value.trim()) {
    afficherMessage("Écris ou colle un texte à lire.");
    return;
  }
  afficherMessage("");
  montrerPhrases(await api.lire(texte.value));
};
$("arreter").onclick = () => api.arreter();

api.surSelection(montrerPhrases);
api.surVoix((e) => {
  const spans = [...lecture.children];
  spans.forEach((s) => s.classList.remove("en-cours"));
  if (e.etat === "phrase") {
    spans[e.index]?.classList.add("en-cours");
    spans[e.index]?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  } else if (e.etat === "erreur") {
    afficherMessage(e.message, true);
  }
});

const auto = $<HTMLInputElement>("auto");
const autoErreur = $<HTMLParagraphElement>("auto-erreur");
function montrerErreurAuto(m: string | null) {
  autoErreur.textContent = m ?? "";
  autoErreur.hidden = !m;
}
auto.onchange = () => api.autoActiver(auto.checked);
api.autoEtat().then((e) => {
  if (e.erreur) {
    auto.checked = false;
    auto.disabled = true;
    montrerErreurAuto(e.erreur);
  }
});
api.surErreurAuto(montrerErreurAuto);

if (api.estMac) $("raccourci").textContent = "⌘";
saisie.focus();
