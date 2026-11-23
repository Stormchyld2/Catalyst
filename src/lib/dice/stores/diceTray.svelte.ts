/**
 * Global dice tray state: one floating tray, one engine, one session history.
 *
 * Any module can open the tray or request a roll:
 *
 *   import { diceTray } from "$lib/dice";
 *   diceTray.open();
 *   const record = await diceTray.roll({ expression: "2d20", source: "gm-console" });
 *
 * History lives here, not in the tray component, so closing the tray or
 * switching between compact and expanded keeps it.
 */
import { DiceEngine } from "../engine/diceEngine";
import { formatDiceNotation } from "../engine/diceNotation";
import {
  RendererDisposedError,
  type DiceRenderer,
} from "../engine/diceRenderer";
import { RollHistory } from "../integration/sessionHistory.svelte";
import type {
  DieType,
  RollRecord,
  RollRequest,
  RollStatus,
} from "../types/dice";

export type TrayMode = "compact" | "expanded";

export interface TrayGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DiceSettings {
  /** Show the 3D physics roll in expanded mode. Off means instant results. */
  animate: boolean;
  /** No shadows, lighter rendering. */
  lowPerformance: boolean;
}

export const TRAY_STORAGE_KEY = "catalyst:diceTray";
export const SETTINGS_STORAGE_KEY = "catalyst:diceSettings";

export const TRAY_LIMITS = {
  expanded: { minWidth: 560, minHeight: 400, width: 820, height: 560 },
  compact: { minWidth: 300, minHeight: 0, width: 320, height: 0 },
} as const;

const EDGE = 8;

function storage(): Storage | null {
  try {
    return typeof localStorage === "undefined" ? null : localStorage;
  } catch {
    return null;
  }
}

function readJson<T>(key: string): Partial<T> | null {
  try {
    const raw = storage()?.getItem(key);
    const value = raw ? JSON.parse(raw) : null;
    return value && typeof value === "object" ? value : null;
  } catch {
    return null;
  }
}

function writeJson(key: string, value: unknown) {
  try {
    storage()?.setItem(key, JSON.stringify(value));
  } catch {
    // Storage full or blocked: preferences are a convenience.
  }
}

function prefersReducedMotion(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}

function viewport() {
  return {
    width: typeof window === "undefined" ? 1280 : window.innerWidth,
    height: typeof window === "undefined" ? 800 : window.innerHeight,
  };
}

function finite(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

/** Keep the tray on screen and within its size limits. */
export function clampGeometry(
  geometry: TrayGeometry,
  mode: TrayMode,
  view = viewport(),
): TrayGeometry {
  const limits = TRAY_LIMITS[mode];
  const maxWidth = Math.max(limits.minWidth, view.width - EDGE * 2);
  const maxHeight = Math.max(limits.minHeight, view.height - EDGE * 2);
  const width = Math.min(Math.max(geometry.width, limits.minWidth), maxWidth);
  const height =
    mode === "compact"
      ? 0
      : Math.min(Math.max(geometry.height, limits.minHeight), maxHeight);
  // Compact height is content-sized; reserve room for it when clamping.
  const occupied = mode === "compact" ? 240 : height;
  const x = Math.min(
    Math.max(geometry.x, EDGE),
    Math.max(EDGE, view.width - width - EDGE),
  );
  const y = Math.min(
    Math.max(geometry.y, EDGE),
    Math.max(EDGE, view.height - occupied - EDGE),
  );
  return {
    x: Math.round(x),
    y: Math.round(y),
    width: Math.round(width),
    height: Math.round(height),
  };
}

function defaultGeometry(mode: TrayMode): TrayGeometry {
  const view = viewport();
  const limits = TRAY_LIMITS[mode];
  return clampGeometry(
    {
      x: (view.width - limits.width) / 2,
      y:
        mode === "compact"
          ? view.height - 300
          : (view.height - limits.height) / 2,
      width: limits.width,
      height: limits.height,
    },
    mode,
    view,
  );
}

function storedGeometry(value: unknown, mode: TrayMode): TrayGeometry {
  const g = value as Partial<TrayGeometry> | undefined;
  if (
    !g ||
    !finite(g.x) ||
    !finite(g.y) ||
    !finite(g.width) ||
    !finite(g.height)
  )
    return defaultGeometry(mode);
  return clampGeometry(g as TrayGeometry, mode);
}

interface StoredTray {
  mode: TrayMode;
  expanded: TrayGeometry;
  compact: TrayGeometry;
}

export class DiceTrayStore {
  readonly engine: DiceEngine;
  readonly history = new RollHistory();

  isOpen = $state(false);
  mode = $state<TrayMode>("expanded");
  geometry = $state<Record<TrayMode, TrayGeometry>>({
    expanded: { x: 0, y: 0, width: 0, height: 0 },
    compact: { x: 0, y: 0, width: 0, height: 0 },
  });
  settings = $state<DiceSettings>({ animate: true, lowPerformance: false });

  /** Dice queued for the next roll from the tray's own controls. */
  selectedType = $state<DieType>("d20");
  counts = $state<Partial<Record<DieType, number>>>({ d20: 2 });

  status = $state<RollStatus>("idle");
  error = $state<string | null>(null);
  /** The roll currently shown in the tray, until Clear. */
  current = $state.raw<RollRecord | null>(null);

  private renderer: DiceRenderer | null = null;
  private rendererReady = false;
  private generation = 0;

  constructor(engine = new DiceEngine()) {
    this.engine = engine;
    const tray = readJson<StoredTray>(TRAY_STORAGE_KEY);
    this.mode = tray?.mode === "compact" ? "compact" : "expanded";
    this.geometry = {
      expanded: storedGeometry(tray?.expanded, "expanded"),
      compact: storedGeometry(tray?.compact, "compact"),
    };
    const settings = readJson<DiceSettings>(SETTINGS_STORAGE_KEY);
    this.settings = {
      animate:
        typeof settings?.animate === "boolean"
          ? settings.animate
          : !prefersReducedMotion(),
      lowPerformance: settings?.lowPerformance === true,
    };
  }

  // --- visibility -----------------------------------------------------------

  open(mode?: TrayMode) {
    if (mode) this.setMode(mode);
    this.isOpen = true;
  }

  close() {
    this.cancelPending();
    this.isOpen = false;
  }

  toggle() {
    if (this.isOpen) this.close();
    else this.open();
  }

  setMode(mode: TrayMode) {
    if (mode === this.mode) return;
    this.cancelPending();
    this.mode = mode;
    this.persistTray();
  }

  // --- geometry -------------------------------------------------------------

  get activeGeometry(): TrayGeometry {
    return this.geometry[this.mode];
  }

  moveTo(x: number, y: number) {
    this.geometry[this.mode] = clampGeometry(
      { ...this.activeGeometry, x, y },
      this.mode,
    );
  }

  resizeTo(width: number, height: number) {
    this.geometry[this.mode] = clampGeometry(
      { ...this.activeGeometry, width, height },
      this.mode,
    );
  }

  /** Re-fit after the Catalyst window changes size. */
  refit() {
    this.geometry = {
      expanded: clampGeometry(this.geometry.expanded, "expanded"),
      compact: clampGeometry(this.geometry.compact, "compact"),
    };
  }

  persistTray() {
    writeJson(TRAY_STORAGE_KEY, {
      mode: this.mode,
      expanded: this.geometry.expanded,
      compact: this.geometry.compact,
    } satisfies StoredTray);
  }

  // --- settings -------------------------------------------------------------

  updateSettings(patch: Partial<DiceSettings>) {
    this.settings = { ...this.settings, ...patch };
    writeJson(SETTINGS_STORAGE_KEY, this.settings);
  }

  // --- dice selection -------------------------------------------------------

  get selection(): string {
    return formatDiceNotation(
      Object.entries(this.counts)
        .filter(([, count]) => (count ?? 0) > 0)
        .map(([type, count]) => ({ type: type as DieType, count: count! })),
    );
  }

  canRoll(type: DieType): boolean {
    return this.engine.capabilities.dice.includes(type);
  }

  selectType(type: DieType) {
    if (!this.canRoll(type)) return;
    this.selectedType = type;
    if (!this.counts[type]) this.counts = { ...this.counts, [type]: 1 };
  }

  adjustCount(delta: number, type: DieType = this.selectedType) {
    if (!this.canRoll(type)) return;
    const next = Math.min(20, Math.max(0, (this.counts[type] ?? 0) + delta));
    this.counts = { ...this.counts, [type]: next };
  }

  // --- renderer -------------------------------------------------------------

  /** The tray's canvas registers its renderer here once it is ready. */
  attachRenderer(renderer: DiceRenderer) {
    this.renderer = renderer;
    this.rendererReady = true;
  }

  detachRenderer(renderer: DiceRenderer) {
    if (this.renderer !== renderer) return;
    this.cancelPending();
    this.renderer = null;
    this.rendererReady = false;
  }

  /** True when the next tray roll will be animated by physics. */
  get physicsActive(): boolean {
    return (
      this.isOpen &&
      this.mode === "expanded" &&
      this.settings.animate &&
      this.rendererReady
    );
  }

  // --- rolling --------------------------------------------------------------

  /**
   * Roll and record. Resolves with the record, or null if the roll was
   * superseded, cleared, or the tray closed first. Errors are shown in the
   * tray and nothing is recorded.
   */
  async roll(request: RollRequest): Promise<RollRecord | null> {
    const generation = ++this.generation;
    const renderer = this.physicsActive ? this.renderer : null;
    this.status = renderer ? "rolling" : "idle";
    this.error = null;
    this.current = null;
    try {
      const record = await this.engine.roll(request, renderer);
      if (generation !== this.generation) return null;
      this.history.add(record);
      this.current = record;
      this.status = "idle";
      return record;
    } catch (error) {
      if (generation !== this.generation) return null;
      this.status = error instanceof RendererDisposedError ? "idle" : "error";
      this.error =
        error instanceof RendererDisposedError ? null : messageOf(error);
      return null;
    }
  }

  rollSelection() {
    return this.roll({ expression: this.selection, source: "dice-tray" });
  }

  /** Roll the last expression again with the same context. */
  reroll() {
    const last = this.current ?? this.history.latest;
    if (!last) return this.rollSelection();
    return this.roll({
      expression: last.expression,
      source: last.source,
      visibility: last.visibility,
      sessionId: last.sessionId,
      characterId: last.characterId,
      encounterId: last.encounterId,
      purpose: last.purpose,
    });
  }

  /** Clear the dice and the shown result. History is kept. */
  clearTray() {
    this.cancelPending();
    this.renderer?.clear();
    this.current = null;
    this.error = null;
  }

  private cancelPending() {
    this.generation++;
    if (this.status === "rolling") this.status = "idle";
  }
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const diceTray = new DiceTrayStore();
