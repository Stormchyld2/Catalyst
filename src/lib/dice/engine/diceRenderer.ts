/**
 * 3D presentation layer for dice.
 *
 * The renderer only animates and reports what settled. It never decides
 * modifiers, visibility, or history; the engine and tray do that.
 *
 * Two implementations:
 * - DiceBoxRenderer: @3d-dice/dice-box (Babylon.js + Ammo.js physics). The
 *   settled face of each simulated die is the result.
 * - no renderer: the engine resolves with an unbiased random source and the
 *   tray shows the values without animation (compact mode, reduced motion,
 *   or when WebGL is unavailable).
 *
 * Dice Box 1.1.4 has no destroy(). dispose() releases its resources by
 * terminating the workers it created during init (the physics worker and the
 * OffscreenCanvas render worker, which owns the WebGL context), dropping its
 * resize hook, and removing its canvas. See
 * docs/architecture/DICE-OPEN-SOURCE-INTEGRATION.md.
 */
import type { DiceExpression, DieResult, DieType } from "../types/dice";
import { DIE_TYPES } from "../types/dice";

export interface DiceRenderer {
  readonly kind: "physics";
  /** Create the canvas inside `container` and load physics and theme assets. */
  init(container: HTMLElement): Promise<void>;
  /** Throw the dice and resolve with what settled, in throw order. */
  roll(expression: DiceExpression): Promise<DieResult[]>;
  /** Remove dice from the tray. Any pending roll is abandoned. */
  clear(): void;
  /** Match the canvas to its container after a layout change. */
  resize(): void;
  /** Release GPU, physics workers, listeners and the canvas. Idempotent. */
  dispose(): void;
}

export class RendererUnavailableError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "RendererUnavailableError";
  }
}

export class RollTimeoutError extends Error {
  constructor() {
    super("The dice did not settle in time. Nothing was recorded.");
    this.name = "RollTimeoutError";
  }
}

export class RendererDisposedError extends Error {
  constructor() {
    super("The dice tray was closed before the roll finished.");
    this.name = "RendererDisposedError";
  }
}

/** Where the bundled Dice Box assets are served from (static/assets/dice-box). */
export const DICE_ASSET_PATH = "/assets/dice-box/";

export function hasWebGL(doc: Document = document): boolean {
  try {
    const canvas = doc.createElement("canvas");
    return !!(canvas.getContext("webgl2") || canvas.getContext("webgl"));
  } catch {
    return false;
  }
}

// --- Dice Box adapter -------------------------------------------------------

/** The subset of the Dice Box 1.1 API this adapter relies on. */
export interface DiceBoxDie {
  sides: number | string;
  value: number;
}

export interface DiceBoxInstance {
  canvas?: HTMLCanvasElement;
  init(): Promise<unknown>;
  roll(notation: string[]): Promise<DiceBoxDie[]>;
  clear(): unknown;
  resizeWorld(): void;
}

export type DiceBoxConstructor = new (
  config: Record<string, unknown>,
) => DiceBoxInstance;

export interface DiceBoxRendererOptions {
  /** Loads the Dice Box class. Defaults to a lazy import so the 3D code is code-split. */
  load?: () => Promise<DiceBoxConstructor>;
  assetPath?: string;
  /** Disable shadows and lower the render load. */
  lowPerformance?: boolean;
  /** Body tint for the default theme. */
  themeColor?: string;
  /** Abandon a roll that has not settled after this long. */
  timeoutMs?: number;
  webgl?: () => boolean;
}

const defaultLoad = async (): Promise<DiceBoxConstructor> =>
  (await import("@3d-dice/dice-box")).default as unknown as DiceBoxConstructor;

let instanceCounter = 0;

export function dieTypeFromSides(sides: number | string): DieType {
  const key = (
    typeof sides === "string" && sides.startsWith("d") ? sides : `d${sides}`
  ) as DieType;
  if (!DIE_TYPES.includes(key))
    throw new Error(`Unexpected die from renderer: ${sides}`);
  return key;
}

export class DiceBoxRenderer implements DiceRenderer {
  readonly kind = "physics" as const;
  private box: DiceBoxInstance | null = null;
  private host: HTMLElement | null = null;
  private workers: Worker[] = [];
  private resizeHandler: (() => void) | null = null;
  private disposed = false;
  private abandon: ((error: Error) => void) | null = null;
  private readonly options: Required<
    Omit<DiceBoxRendererOptions, "load" | "webgl">
  > &
    Pick<DiceBoxRendererOptions, "load" | "webgl">;

  constructor(options: DiceBoxRendererOptions = {}) {
    this.options = {
      assetPath: DICE_ASSET_PATH,
      lowPerformance: false,
      themeColor: "#a8732e",
      timeoutMs: 20_000,
      ...options,
    };
  }

  async init(container: HTMLElement): Promise<void> {
    if (this.disposed) throw new RendererDisposedError();
    if (this.box) return;
    if (!(this.options.webgl ?? hasWebGL)())
      throw new RendererUnavailableError(
        "WebGL is not available on this device.",
      );

    const DiceBox = await (this.options.load ?? defaultLoad)();
    if (this.disposed) throw new RendererDisposedError();

    const host = document.createElement("div");
    host.id = `catalyst-dice-${++instanceCounter}`;
    host.className = "dice-renderer-host";
    container.appendChild(host);
    this.host = host;

    const box = new DiceBox({
      container: `#${host.id}`,
      id: `${host.id}-canvas`,
      assetPath: this.options.assetPath,
      origin: window.location.origin,
      theme: "default",
      themeColor: this.options.themeColor,
      scale: 8,
      enableShadows: !this.options.lowPerformance,
      shadowTransparency: 0.7,
      lightIntensity: 1,
      offscreen: true,
      delay: 10,
    });
    this.hookResize(box);

    const restoreWorker = this.trackWorkers();
    try {
      await box.init();
    } catch (error) {
      restoreWorker();
      this.dispose();
      throw new RendererUnavailableError(
        `The 3D dice could not start: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
    restoreWorker();
    if (this.disposed) {
      this.release(box);
      throw new RendererDisposedError();
    }
    this.box = box;
  }

  async roll(expression: DiceExpression): Promise<DieResult[]> {
    const box = this.box;
    if (!box || this.disposed) throw new RendererDisposedError();
    this.abandon?.(new RendererDisposedError());
    const notation = expression.terms.map(
      (term) => `${term.count}${term.type}`,
    );

    let timer: ReturnType<typeof setTimeout> | undefined;
    const cancelled = new Promise<never>((_, reject) => {
      this.abandon = reject;
      timer = setTimeout(
        () => reject(new RollTimeoutError()),
        this.options.timeoutMs,
      );
    });
    try {
      const settled = await Promise.race([box.roll(notation), cancelled]);
      return settled.map((die) => ({
        type: dieTypeFromSides(die.sides),
        value: die.value,
      }));
    } finally {
      clearTimeout(timer);
      this.abandon = null;
    }
  }

  clear(): void {
    this.abandon?.(new RendererDisposedError());
    this.box?.clear();
  }

  resize(): void {
    this.resizeHandler?.();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.abandon?.(new RendererDisposedError());
    this.release(this.box);
    this.box = null;
  }

  private release(box: DiceBoxInstance | null): void {
    const canvas = box?.canvas ?? this.host?.querySelector("canvas") ?? null;
    // Onscreen fallback mode keeps the WebGL context on the main thread.
    try {
      const context =
        canvas?.getContext("webgl2") ?? canvas?.getContext("webgl");
      context?.getExtension("WEBGL_lose_context")?.loseContext();
    } catch {
      // Offscreen canvases cannot hand out a context; terminating the worker frees it.
    }
    for (const worker of this.workers) worker.terminate();
    this.workers = [];
    this.resizeHandler = null;
    this.host?.remove();
    this.host = null;
  }

  /**
   * Dice Box registers an untracked window resize listener in resizeWorld().
   * Capture that handler synchronously instead, so the tray drives resizes from
   * its own ResizeObserver and nothing is left on window after dispose.
   */
  private hookResize(box: DiceBoxInstance): void {
    const original = Object.getPrototypeOf(box).resizeWorld as
      | (() => void)
      | undefined;
    if (typeof original !== "function") return;
    box.resizeWorld = () => {
      const add = window.addEventListener;
      let captured: (() => void) | null = null;
      window.addEventListener = ((
        type: string,
        listener: unknown,
        options?: unknown,
      ) => {
        if (type === "resize" && typeof listener === "function")
          captured = listener as () => void;
        else
          add.call(
            window,
            type,
            listener as EventListener,
            options as AddEventListenerOptions,
          );
      }) as typeof window.addEventListener;
      try {
        original.call(box);
      } finally {
        window.addEventListener = add;
      }
      if (captured) this.resizeHandler = captured;
    };
  }

  /**
   * Record the workers Dice Box spawns during init so dispose() can terminate
   * them. Catalyst creates no other workers, so nothing else is captured.
   */
  private trackWorkers(): () => void {
    const Original = globalThis.Worker;
    if (typeof Original !== "function") return () => {};
    const workers = this.workers;
    class TrackedWorker extends Original {
      constructor(url: string | URL, options?: WorkerOptions) {
        super(url, options);
        workers.push(this);
      }
    }
    globalThis.Worker = TrackedWorker;
    return () => {
      if (globalThis.Worker === TrackedWorker) globalThis.Worker = Original;
    };
  }
}
