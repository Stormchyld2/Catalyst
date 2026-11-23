/**
 * Dice sounds, synthesized with the Web Audio API.
 *
 * Nothing is downloaded or bundled: each sound is a short burst of filtered
 * noise generated on the spot, so it works offline and has no licensing
 * question. Dice Box 1.1.4 ships no sounds of its own.
 */

type AudioContextConstructor = new () => AudioContext;

function audioContextClass(): AudioContextConstructor | null {
  if (typeof window === "undefined") return null;
  const w = window as unknown as {
    AudioContext?: AudioContextConstructor;
    webkitAudioContext?: AudioContextConstructor;
  };
  return w.AudioContext ?? w.webkitAudioContext ?? null;
}

export class DiceSound {
  private context: AudioContext | null = null;
  enabled = true;
  /** 0..1 */
  volume = 0.5;

  private ctx(): AudioContext | null {
    if (!this.enabled || this.volume <= 0) return null;
    if (!this.context) {
      const Context = audioContextClass();
      if (!Context) return null;
      try {
        this.context = new Context();
      } catch {
        return null;
      }
    }
    if (this.context.state === "suspended") void this.context.resume().catch(() => {});
    return this.context;
  }

  /** One hard knock: a die hitting the tray or landing. */
  private knock(ctx: AudioContext, at: number, loudness: number, pitch: number) {
    const length = Math.floor(ctx.sampleRate * 0.06);
    const buffer = ctx.createBuffer(1, length, ctx.sampleRate);
    const data = buffer.getChannelData(0);
    for (let i = 0; i < length; i++) data[i] = (Math.random() * 2 - 1) * (1 - i / length) ** 3;
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    const filter = ctx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.value = pitch;
    filter.Q.value = 4;
    const gain = ctx.createGain();
    gain.gain.value = this.volume * loudness;
    source.connect(filter).connect(gain).connect(ctx.destination);
    source.start(at);
  }

  /** The rattle of dice leaving the hand and hitting the tray. */
  playThrow(diceCount: number) {
    const ctx = this.ctx();
    if (!ctx) return;
    const hits = Math.min(14, 4 + diceCount * 2);
    for (let i = 0; i < hits; i++) {
      const at = ctx.currentTime + 0.05 + i * (0.04 + Math.random() * 0.06);
      this.knock(ctx, at, 0.9 * (1 - i / (hits + 2)), 1800 + Math.random() * 1600);
    }
  }

  /** A single die coming to rest. */
  playSettle() {
    const ctx = this.ctx();
    if (!ctx) return;
    this.knock(ctx, ctx.currentTime, 0.45, 1200 + Math.random() * 500);
  }

  dispose() {
    void this.context?.close().catch(() => {});
    this.context = null;
  }
}
