/**
 * The open campaign's soundboard, shared by the Soundboard workspace and the
 * GM Screen so audio keeps playing when you switch between them (doc 034).
 * `Assets/Audio/soundboard.json` stays the only copy of the layout.
 */

import { invoke } from "@tauri-apps/api/core";
import { SoundboardEngine, defaultLayout, type AudioFile, type SoundboardLayout } from "./soundboard.svelte";

const SAVE_DELAY_MS = 400;

export class SoundboardBoard {
  layout = $state<SoundboardLayout>({ ...defaultLayout(), pads: [] });
  files = $state<AudioFile[]>([]);
  loading = $state(false);
  error = $state<string | null>(null);
  saveState = $state<"idle" | "saving" | "saved" | "error">("idle");
  root = $state("");
  readonly engine: SoundboardEngine;

  #saveTimer: ReturnType<typeof setTimeout> | null = null;
  #invoke: typeof invoke;

  constructor(call: typeof invoke = invoke) {
    this.#invoke = call;
    this.engine = new SoundboardEngine(defaultLayout(), (file) =>
      this.#invoke<ArrayBuffer>("read_soundboard_audio", { projectRoot: this.root, path: file }),
    );
  }

  /** Load a campaign's board. Re-opening the same campaign keeps playback going. */
  async open(projectRoot: string) {
    if (!projectRoot || projectRoot === this.root) return;
    await this.flush();
    this.engine.stopAll();
    for (const pad of this.layout.pads) this.engine.forget(pad.file);
    this.root = projectRoot;
    this.loading = true;
    this.error = null;
    try {
      const [layout, files] = await Promise.all([
        this.#invoke<SoundboardLayout | null>("load_soundboard_layout", { projectRoot }),
        this.#invoke<AudioFile[]>("list_soundboard_audio", { projectRoot }),
      ]);
      if (projectRoot !== this.root) return;
      // No soundboard.json yet: start with the built-in pads (doc 033).
      this.layout = layout ?? defaultLayout();
      this.files = files;
      this.engine.setLayout(layout);
      this.saveState = "idle";
    } catch (e) {
      if (projectRoot === this.root) this.error = String(e);
    } finally {
      if (projectRoot === this.root) this.loading = false;
    }
  }

  /** The campaign closed: save, silence and forget it. */
  async close() {
    await this.flush();
    this.engine.stopAll();
    for (const pad of this.layout.pads) this.engine.forget(pad.file);
    this.root = "";
    this.layout = { ...defaultLayout(), pads: [] };
    this.files = [];
    this.error = null;
    this.saveState = "idle";
  }

  async refreshFiles() {
    if (!this.root) return;
    try {
      this.files = await this.#invoke<AudioFile[]>("list_soundboard_audio", { projectRoot: this.root });
      for (const pad of this.layout.pads) this.engine.forget(pad.file);
    } catch (e) {
      this.error = String(e);
    }
  }

  /** Apply a change now and save it shortly after. */
  commit(next: SoundboardLayout) {
    this.layout = next;
    this.engine.setLayout(next);
    this.saveState = "saving";
    if (this.#saveTimer) clearTimeout(this.#saveTimer);
    this.#saveTimer = setTimeout(() => void this.#persist(), SAVE_DELAY_MS);
  }

  /** Write any pending change immediately. */
  async flush() {
    if (!this.#saveTimer) return;
    clearTimeout(this.#saveTimer);
    await this.#persist();
  }

  async #persist() {
    this.#saveTimer = null;
    const projectRoot = this.root;
    if (!projectRoot) return;
    try {
      await this.#invoke("save_soundboard_layout", { projectRoot, layout: $state.snapshot(this.layout) });
      if (projectRoot === this.root && !this.#saveTimer) this.saveState = "saved";
    } catch (e) {
      this.saveState = "error";
      this.error = `Could not save the soundboard: ${String(e)}`;
    }
  }
}

export const soundboard = new SoundboardBoard();
