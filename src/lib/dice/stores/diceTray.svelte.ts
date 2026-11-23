/**
 * Global dice tray state: one floating tray, one engine, one session history.
 *
 * Any module can open the tray or request a roll:
 *
 *   import { diceTray } from "$lib/dice";
 *   diceTray.open();
 *   const record = await diceTray.roll({ expression: "2d20", source: "gm-console" });
 *
 * Pass `{ show: true }` to open the tray and animate the roll with physics.
 *
 * History lives here, not in the tray component, so closing the tray or
 * switching between compact and expanded keeps it. While a campaign session is
 * active, every roll is also appended to that session's file in the project.
 */
import { DiceEngine } from "../engine/diceEngine";
import { formatDiceNotation } from "../engine/diceNotation";
import { DiceSound } from "../engine/diceSound";
import {
  RendererDisposedError,
  type DiceRenderer,
} from "../engine/diceRenderer";
import { RollHistory } from "../integration/sessionHistory.svelte";
import {
  CampaignSessionSink,
  diceSessionFiles,
  type DiceSessionFiles,
  type DiceSessionInfo,
} from "../integration/campaignSessions";
import type {
  DiceTerm,
  DieType,
  RollMode,
  RollRecord,
  RollRequest,
  RollStatus,
  RollVisibility,
} from "../types/dice";

export type TrayMode = "compact" | "expanded";

export interface TrayGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Dice body colours. Dice Box picks light or dark numbers to suit each one. */
export const DICE_COLORS = {
  amber: { label: "Amber", value: "#a8732e" },
  ivory: { label: "Ivory", value: "#e8e1d0" },
  crimson: { label: "Crimson", value: "#8e2430" },
  sapphire: { label: "Sapphire", value: "#2a4f8f" },
  emerald: { label: "Emerald", value: "#22694a" },
  obsidian: { label: "Obsidian", value: "#26262b" },
} as const;
export type DiceColor = keyof typeof DICE_COLORS;

export const DICE_SIZES = { small: 6, medium: 8, large: 10 } as const;
export type DiceSize = keyof typeof DICE_SIZES;

/** Physics gravity: higher settles faster. */
export const ROLL_SPEEDS = { relaxed: 0.7, normal: 1, quick: 1.8 } as const;
export type RollSpeed = keyof typeof ROLL_SPEEDS;

export const TRAY_SURFACES = ["charcoal", "leather", "walnut", "stone", "felt"] as const;
export type TraySurface = (typeof TRAY_SURFACES)[number];

export interface DiceSettings {
  /** Show the 3D physics roll in expanded mode. Off means instant results. */
  animate: boolean;
  /** No shadows, lighter rendering. */
  lowPerformance: boolean;
  sound: boolean;
  /** 0..1 */
  volume: number;
  diceColor: DiceColor;
  diceSize: DiceSize;
  speed: RollSpeed;
  surface: TraySurface;
}

export type RendererState = "off" | "loading" | "ready" | "unavailable";

export interface RollOptions {
  /** Open the tray (expanded) and animate the roll when 3D is available. */
  show?: boolean;
}

export const TRAY_STORAGE_KEY = "catalyst:diceTray";
export const SETTINGS_STORAGE_KEY = "catalyst:diceSettings";

export const TRAY_LIMITS = {
  expanded: { minWidth: 560, minHeight: 400, width: 820, height: 560 },
  compact: { minWidth: 340, minHeight: 0, width: 360, height: 0 },
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
  compactHeight = 240,
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
  const occupied = mode === "compact" ? compactHeight : height;
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

/** Tray selections read largest die first (`2d20+1d6`), percentile last. */
const SELECTION_ORDER: readonly DieType[] = ["d20", "d12", "d10", "d8", "d6", "d4", "d100"];

export class DiceTrayStore {
  readonly engine: DiceEngine;
  readonly history = new RollHistory();

  isOpen = $state(false);
  mode = $state<TrayMode>("expanded");
  geometry = $state<Record<TrayMode, TrayGeometry>>({
    expanded: { x: 0, y: 0, width: 0, height: 0 },
    compact: { x: 0, y: 0, width: 0, height: 0 },
  });
  settings = $state<DiceSettings>(defaultSettings());

  /** Dice queued for the next roll from the tray's own controls. */
  selectedType = $state<DieType>("d20");
  counts = $state<Partial<Record<DieType, number>>>({ d20: 2 });
  modifier = $state(0);
  /** The expanded tray's +/- adjust the modifier instead of a die when true. */
  modifierSelected = $state(false);
  /** ADV / DIS toggles: turn the selection's d20s into one advantage/disadvantage check. */
  rollMode = $state<RollMode>("normal");
  /** Who may see rolls made from the tray. */
  visibility = $state<RollVisibility>("public");
  /** The active campaign session, if any; rolls are saved to its file. */
  campaign = $state.raw<{ projectRoot: string; session: DiceSessionInfo } | null>(null);
  rendererState = $state<RendererState>("off");

  status = $state<RollStatus>("idle");
  error = $state<string | null>(null);
  /** The roll currently shown in the tray, until Clear. */
  current = $state.raw<RollRecord | null>(null);

  readonly sound = new DiceSound();
  /** Measured height of the content-sized compact tray. */
  private compactHeight = 240;
  private renderer: DiceRenderer | null = null;
  private rendererReady = false;
  private generation = 0;
  private readyWaiters: Array<() => void> = [];
  private listeners = new Set<(record: RollRecord) => void>();
  private files: DiceSessionFiles;

  constructor(engine = new DiceEngine(), files: DiceSessionFiles = diceSessionFiles) {
    this.engine = engine;
    this.files = files;
    const tray = readJson<StoredTray>(TRAY_STORAGE_KEY);
    this.mode = tray?.mode === "compact" ? "compact" : "expanded";
    this.geometry = {
      expanded: storedGeometry(tray?.expanded, "expanded"),
      compact: storedGeometry(tray?.compact, "compact"),
    };
    this.settings = readSettings(readJson<DiceSettings>(SETTINGS_STORAGE_KEY));
    this.applySound();
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
      undefined,
      this.compactHeight,
    );
  }

  resizeTo(width: number, height: number) {
    this.geometry[this.mode] = clampGeometry(
      { ...this.activeGeometry, width, height },
      this.mode,
      undefined,
      this.compactHeight,
    );
  }

  /** Re-fit after the Catalyst window changes size. */
  refit() {
    this.geometry = {
      expanded: clampGeometry(this.geometry.expanded, "expanded"),
      compact: clampGeometry(this.geometry.compact, "compact", undefined, this.compactHeight),
    };
  }

  /** The compact tray sizes to its content; it reports its height so it stays on screen. */
  setCompactHeight(height: number) {
    const next = Math.round(height);
    if (next <= 0 || next === this.compactHeight) return;
    this.compactHeight = next;
    if (this.mode === "compact") this.refit();
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
    this.settings = readSettings({ ...this.settings, ...patch });
    writeJson(SETTINGS_STORAGE_KEY, this.settings);
    this.applySound();
  }

  /** The renderer options that need a fresh 3D renderer when they change. */
  get rendererOptions() {
    return {
      lowPerformance: this.settings.lowPerformance,
      themeColor: DICE_COLORS[this.settings.diceColor].value,
      scale: DICE_SIZES[this.settings.diceSize],
      gravity: ROLL_SPEEDS[this.settings.speed],
    };
  }

  private applySound() {
    this.sound.enabled = this.settings.sound;
    this.sound.volume = this.settings.volume;
  }

  // --- dice selection -------------------------------------------------------

  /**
   * The tray's dice as notation, e.g. `2d20+1d6+3`. With ADV or DIS on, the
   * d20s become one `2d20kh1` / `2d20kl1` check.
   */
  get selection(): string {
    const terms: DiceTerm[] = SELECTION_ORDER.filter((type) => (this.counts[type] ?? 0) > 0).map(
      (type) => ({ type, count: this.counts[type]! })
    );
    if (this.rollMode !== "normal") {
      const others = terms.filter((term) => term.type !== "d20");
      const keep = { mode: this.rollMode === "advantage" ? "highest" : "lowest", count: 1 } as const;
      return formatDiceNotation([{ type: "d20", count: 2, keep }, ...others], this.modifier);
    }
    if (!terms.length) return "";
    return formatDiceNotation(terms, this.modifier);
  }

  setRollMode(mode: RollMode) {
    this.rollMode = this.rollMode === mode ? "normal" : mode;
  }

  adjustModifier(delta: number) {
    this.modifier = Math.max(-99, Math.min(99, this.modifier + delta));
  }

  /** Reset the dice selection to a single d20. */
  resetSelection() {
    this.counts = { d20: 1 };
    this.selectedType = "d20";
    this.modifierSelected = false;
    this.modifier = 0;
    this.rollMode = "normal";
  }

  canRoll(type: DieType): boolean {
    return this.engine.capabilities.dice.includes(type);
  }

  /** Make `type` the die that + and − adjust. Adds one if none are selected yet. */
  selectType(type: DieType) {
    if (!this.canRoll(type)) return;
    this.selectedType = type;
    this.modifierSelected = false;
    if (!this.counts[type]) this.counts = { ...this.counts, [type]: 1 };
  }

  /** Die buttons: the first click selects the die, later clicks add one more. */
  pickDie(type: DieType) {
    if (!this.modifierSelected && this.selectedType === type) this.adjustCount(1, type);
    else this.selectType(type);
  }

  /** Make + and − adjust the modifier. */
  selectModifier() {
    this.modifierSelected = true;
  }

  /** Change the count of `type`, or of the selected target (die or modifier). */
  adjustCount(delta: number, type?: DieType) {
    if (!type && this.modifierSelected) return this.adjustModifier(delta);
    const die = type ?? this.selectedType;
    if (!this.canRoll(die)) return;
    const next = Math.min(20, Math.max(0, (this.counts[die] ?? 0) + delta));
    this.counts = { ...this.counts, [die]: next };
  }

  // --- renderer -------------------------------------------------------------

  /** The tray's canvas registers its renderer here once it is ready. */
  attachRenderer(renderer: DiceRenderer) {
    this.renderer = renderer;
    this.rendererReady = true;
    this.rendererState = "ready";
    this.flushReady();
  }

  /** The canvas is starting a renderer. */
  rendererLoading() {
    this.rendererState = "loading";
  }

  /** 3D could not start; rolls continue without animation. */
  rendererUnavailable() {
    this.rendererState = "unavailable";
    this.flushReady();
  }

  detachRenderer(renderer: DiceRenderer) {
    if (this.renderer !== renderer) return;
    this.cancelPending();
    this.renderer = null;
    this.rendererReady = false;
    this.rendererState = "off";
  }

  private flushReady() {
    const waiters = this.readyWaiters;
    this.readyWaiters = [];
    waiters.forEach((resolve) => resolve());
  }

  /** Resolve once the renderer is ready or known to be unavailable (or after `ms`). */
  private waitForRenderer(ms = 8000): Promise<void> {
    if (this.rendererState === "ready" || this.rendererState === "unavailable") return Promise.resolve();
    return new Promise((resolve) => {
      const timer = setTimeout(done, ms);
      const waiters = this.readyWaiters;
      function done() {
        clearTimeout(timer);
        const index = waiters.indexOf(done);
        if (index >= 0) waiters.splice(index, 1);
        resolve();
      }
      waiters.push(done);
    });
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
  async roll(request: RollRequest, options: RollOptions = {}): Promise<RollRecord | null> {
    if (options.show) {
      this.open(this.settings.animate ? "expanded" : undefined);
      if (this.mode === "expanded" && this.settings.animate) await this.waitForRenderer();
    }
    const generation = ++this.generation;
    const renderer = this.physicsActive ? this.renderer : null;
    const sessionId = request.sessionId === undefined ? this.campaign?.session.id ?? null : request.sessionId;
    this.status = renderer ? "rolling" : "idle";
    this.error = null;
    this.current = null;
    try {
      const expression = this.engine.parse(request.expression, request.mode);
      this.sound.playThrow(expression.terms.reduce((sum, term) => sum + term.count, 0));
      const record = await this.engine.roll({ ...request, sessionId }, renderer);
      if (generation !== this.generation) return null;
      if (!renderer) this.sound.playSettle();
      this.history.add(record);
      this.current = record;
      this.status = "idle";
      this.listeners.forEach((listener) => listener(record));
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
    return this.roll({
      expression: this.selection,
      source: "dice-tray",
      visibility: this.visibility,
    });
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
      playerId: last.playerId,
      characterId: last.characterId,
      npcId: last.npcId,
      encounterId: last.encounterId,
      purpose: last.purpose,
      label: last.label,
    });
  }

  /**
   * Be told about every recorded roll, e.g. to forward public rolls to a
   * player display. Check `canView` before sending a record anywhere.
   */
  onRecorded(listener: (record: RollRecord) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  // --- campaign sessions ------------------------------------------------------

  /** Start a campaign session in `projectRoot`; rolls are saved to its file until it ends. */
  async startCampaignSession(projectRoot: string, title = ""): Promise<DiceSessionInfo> {
    const session = await this.files.start(projectRoot, title);
    this.useSession(projectRoot, session, []);
    return session;
  }

  /** Reopen an earlier session and load its saved rolls into history. */
  async resumeCampaignSession(projectRoot: string, sessionId: string) {
    const history = await this.files.read(projectRoot, sessionId);
    this.useSession(projectRoot, history.session, history.rolls);
    return history;
  }

  endCampaignSession() {
    this.campaign = null;
    this.history.setSink(null);
  }

  private useSession(projectRoot: string, session: DiceSessionInfo, rolls: RollRecord[]) {
    this.campaign = { projectRoot, session };
    this.history.replace([...rolls].reverse());
    this.history.setSink(
      new CampaignSessionSink(this.files, projectRoot, session.id, (error) => {
        this.error = `Roll not saved to the session file: ${messageOf(error)}`;
      })
    );
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

function defaultSettings(): DiceSettings {
  return {
    animate: !prefersReducedMotion(),
    lowPerformance: false,
    sound: true,
    volume: 0.5,
    diceColor: "amber",
    diceSize: "medium",
    speed: "normal",
    surface: "charcoal",
  };
}

/** Accept stored settings field by field, falling back to defaults. */
function readSettings(stored: Partial<DiceSettings> | null): DiceSettings {
  const d = defaultSettings();
  const s = stored ?? {};
  const pick = <T>(value: unknown, ok: boolean, fallback: T): T => (ok ? (value as T) : fallback);
  return {
    animate: pick(s.animate, typeof s.animate === "boolean", d.animate),
    lowPerformance: pick(s.lowPerformance, typeof s.lowPerformance === "boolean", d.lowPerformance),
    sound: pick(s.sound, typeof s.sound === "boolean", d.sound),
    volume: pick(
      s.volume,
      typeof s.volume === "number" && s.volume >= 0 && s.volume <= 1,
      d.volume
    ),
    diceColor: pick(s.diceColor, typeof s.diceColor === "string" && s.diceColor in DICE_COLORS, d.diceColor),
    diceSize: pick(s.diceSize, typeof s.diceSize === "string" && s.diceSize in DICE_SIZES, d.diceSize),
    speed: pick(s.speed, typeof s.speed === "string" && s.speed in ROLL_SPEEDS, d.speed),
    surface: pick(s.surface, TRAY_SURFACES.includes(s.surface as TraySurface), d.surface),
  };
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export const diceTray = new DiceTrayStore();
