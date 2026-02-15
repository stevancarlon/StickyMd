import { invoke } from "@tauri-apps/api/core";

export interface NoteData {
  content: string;
  color: string;
  pinned: boolean;
  opacity: number;
  border_radius: number;
}

let saveTimeout: ReturnType<typeof setTimeout>;

export async function loadNote(): Promise<NoteData> {
  try {
    return await invoke<NoteData>("load_note");
  } catch {
    return {
      content: "# Welcome to Sticky MD\n\nStart typing your **markdown** here...\n\n- [x] Always on top\n- [x] Draggable\n- [ ] Your notes here",
      color: "yellow",
      pinned: true,
      opacity: 1.0,
      border_radius: 0,
    };
  }
}

export function saveNote(data: NoteData): void {
  clearTimeout(saveTimeout);
  saveTimeout = setTimeout(async () => {
    try {
      await invoke("save_note", { data });
    } catch (e) {
      console.error("Failed to save note:", e);
    }
  }, 500);
}
