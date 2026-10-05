// Pont vers le cœur Rust. Hors de Tauri (aperçu dans un navigateur), on simule des
// réponses pour pouvoir travailler l'interface.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type Suggestion = { mot: string; score: number };
export type EvenementVoix =
  | { etat: "phrase"; index: number }
  | { etat: "fin" }
  | { etat: "erreur"; message: string };

const dansTauri = "__TAURI_INTERNALS__" in window;

const demo: Record<string, string[]> = {
  fotografi: ["photographie", "photographies", "photographe"],
  oto: ["auto", "autos", "otto"],
};

export async function suggerer(saisie: string): Promise<Suggestion[]> {
  if (dansTauri) return invoke("suggerer", { saisie });
  return (demo[saisie] ?? []).map((mot, i) => ({ mot, score: i * 0.5 }));
}

export async function lire(texte: string): Promise<string[]> {
  if (dansTauri) return invoke("lire", { texte });
  return texte.split(/(?<=[.!?])\s+/).filter((p) => p.trim());
}

export async function arreter(): Promise<void> {
  if (dansTauri) await invoke("arreter");
}

export async function inserer(mot: string): Promise<void> {
  if (dansTauri) await invoke("inserer", { mot });
}

export function surVoix(rappel: (e: EvenementVoix) => void): void {
  if (dansTauri) listen<EvenementVoix>("voix", (e) => rappel(e.payload));
}

export function surSelection(rappel: (phrases: string[]) => void): void {
  if (dansTauri) listen<{ phrases: string[] }>("selection", (e) => rappel(e.payload.phrases));
}

export const estMac = navigator.userAgent.includes("Mac");
