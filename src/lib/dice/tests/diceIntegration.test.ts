import { describe, expect, it, vi } from "vitest";
import { DiceEngine } from "../engine/diceEngine";
import { criticalOf, describeRoll } from "../engine/diceResults";
import { DiceSound } from "../engine/diceSound";
import { CampaignSessionSink, type DiceSessionFiles } from "../integration/campaignSessions";
import {
  abilityCheck,
  attackRoll,
  damageRoll,
  encounterRoll,
  initiativeRoll,
  savingThrow,
  skillCheck,
  type RollActor,
} from "../integration/rollRequests";
import { RollHistory } from "../integration/sessionHistory.svelte";
import { canView, visibleTo } from "../integration/visibility";
import type { RollRecord } from "../types/dice";

const hero: RollActor = { kind: "character", id: "character-1", name: "Mira", playerId: "player-1" };
const goblin: RollActor = { kind: "npc", id: "npc-1", name: "Goblin" };

function engine() {
  return new DiceEngine({ random: () => 9, newId: () => "roll-x" });
}

describe("roll request builders", () => {
  it("builds initiative, attack, save, skill and ability checks", () => {
    expect(initiativeRoll(hero, { bonus: 2 })).toMatchObject({
      expression: "1d20+2",
      purpose: "initiative",
      label: "Mira — Initiative",
      characterId: "character-1",
      playerId: "player-1",
      visibility: "public",
      source: "character",
    });
    expect(attackRoll(goblin, { bonus: 4, weapon: "Scimitar" })).toMatchObject({
      expression: "1d20+4",
      purpose: "attack",
      label: "Goblin — Scimitar attack",
      npcId: "npc-1",
      visibility: "gm-private",
      source: "gm-console",
    });
    expect(savingThrow(hero, "Dexterity", { bonus: -1, mode: "disadvantage" })).toMatchObject({
      expression: "1d20-1",
      purpose: "saving-throw",
      mode: "disadvantage",
    });
    expect(skillCheck(hero, "Stealth", { mode: "advantage" }).label).toBe("Mira — Stealth");
    expect(abilityCheck(hero, "Strength").expression).toBe("1d20");
  });

  it("doubles damage dice, not the modifier, on a critical hit", () => {
    expect(damageRoll(hero, "1d8+3").expression).toBe("1d8+3");
    expect(damageRoll(hero, "1d8+2d6+3", { critical: true })).toMatchObject({
      expression: "2d8+4d6+3",
      label: "Mira — Critical damage",
      purpose: "damage",
    });
  });

  it("keeps encounter rolls GM-private by default and validates them early", () => {
    expect(encounterRoll("encounter-1", "1d100", "random-encounter")).toMatchObject({
      encounterId: "encounter-1",
      visibility: "gm-private",
      source: "encounter",
    });
    expect(() => encounterRoll("encounter-1", "2d7", "trap")).toThrow();
  });

  it("produces records the engine resolves with their context", async () => {
    const record = await engine().roll(attackRoll(goblin, { bonus: 4, mode: "advantage", encounterId: "encounter-1" }));
    expect(record).toMatchObject({
      expression: "2d20kh1+4",
      kind: "advantage",
      total: 14,
      npcId: "npc-1",
      encounterId: "encounter-1",
      source: "encounter",
      visibility: "gm-private",
    });
    expect(record.dice.filter((die) => die.dropped)).toHaveLength(1);
  });

  it("rejects a fractional bonus", () => {
    expect(() => initiativeRoll(hero, { bonus: 1.5 })).toThrow(RangeError);
  });
});

describe("visibility", () => {
  const roll = (visibility: RollRecord["visibility"], playerId?: string) =>
    ({ id: visibility + (playerId ?? ""), visibility, playerId }) as RollRecord;
  const records = [roll("public"), roll("gm-private"), roll("player-private", "player-1"), roll("player-private", "player-2")];

  it("shows the GM everything", () => {
    expect(visibleTo(records, { role: "gm" })).toHaveLength(4);
  });

  it("shows a player public rolls and their own private rolls", () => {
    expect(visibleTo(records, { role: "player", playerId: "player-1" }).map((r) => r.id)).toEqual([
      "public",
      "player-privateplayer-1",
    ]);
  });

  it("shows a public display public rolls only", () => {
    expect(visibleTo(records, { role: "public-display" }).map((r) => r.id)).toEqual(["public"]);
    expect(canView(roll("player-private"), { role: "player", playerId: "" })).toBe(false);
  });
});

describe("campaign session sink", () => {
  function files(append = vi.fn(async () => {})): DiceSessionFiles {
    return { start: vi.fn(), append, read: vi.fn(), list: vi.fn() };
  }

  it("appends only rolls that belong to its session", async () => {
    const f = files();
    const sink = new CampaignSessionSink(f, "/campaign", "session-1");
    await sink.append({ sessionId: "session-1" } as RollRecord);
    await sink.append({ sessionId: "session-2" } as RollRecord);
    await sink.append({ sessionId: null } as RollRecord);
    expect(f.append).toHaveBeenCalledTimes(1);
  });

  it("reports a failed save and keeps the roll in memory", async () => {
    const onError = vi.fn();
    const sink = new CampaignSessionSink(files(vi.fn(async () => Promise.reject(new Error("disk full")))), "/c", "s");
    const history = new RollHistory();
    history.setSink(sink);
    const quiet = vi.spyOn(console, "error").mockImplementation(() => {});
    const sinkWithError = new CampaignSessionSink(
      files(vi.fn(async () => Promise.reject(new Error("disk full")))),
      "/c",
      "s",
      onError
    );
    history.setSink(sinkWithError);
    history.add({ id: "roll-1", sessionId: "s", dice: [] } as unknown as RollRecord);
    await vi.waitFor(() => expect(onError).toHaveBeenCalled());
    expect(history.all).toHaveLength(1);
    await expect(sink.append({ sessionId: "s" } as RollRecord)).rejects.toThrow("disk full");
    quiet.mockRestore();
  });
});

describe("session history filters", () => {
  it("filters by player, NPC, die type and purpose", () => {
    const history = new RollHistory();
    const base = { sessionId: null, dice: [{ type: "d20", value: 3 }] };
    history.add({ ...base, id: "a", playerId: "p1", purpose: "attack" } as unknown as RollRecord);
    history.add({ ...base, id: "b", npcId: "n1", dice: [{ type: "d6", value: 2 }] } as unknown as RollRecord);
    expect(history.filter({ playerId: "p1" }).map((r) => r.id)).toEqual(["a"]);
    expect(history.filter({ npcId: "n1" }).map((r) => r.id)).toEqual(["b"]);
    expect(history.filter({ dieType: "d6" }).map((r) => r.id)).toEqual(["b"]);
    expect(history.filter({ purpose: "attack" }).map((r) => r.id)).toEqual(["a"]);
  });
});

describe("criticals and summaries", () => {
  it("counts a critical only when exactly one d20 is kept", () => {
    expect(criticalOf([{ type: "d20", value: 20 }])).toBe("success");
    expect(criticalOf([{ type: "d20", value: 1 }])).toBe("failure");
    expect(criticalOf([{ type: "d20", value: 20 }, { type: "d20", value: 5 }])).toBeUndefined();
    expect(criticalOf([{ type: "d20", value: 1, dropped: true }, { type: "d20", value: 20 }])).toBe("success");
    expect(criticalOf([{ type: "d6", value: 6 }])).toBeUndefined();
  });

  it("describes kind, dropped dice and criticals in plain text", async () => {
    const e = new DiceEngine({ random: (() => { const v = [19, 4]; let i = 0; return () => v[i++]; })(), newId: () => "roll-x" });
    const text = describeRoll(await e.roll({ expression: "1d20+3", mode: "advantage" }));
    expect(text).toContain("Roll: 2d20kh1+3 (advantage)");
    expect(text).toContain("Die 2 (d20): 5 (dropped)");
    expect(text).toContain("Total: 23");
    expect(text).toContain("Critical success");
  });
});

describe("DiceSound", () => {
  it("does nothing without Web Audio, when disabled, or at zero volume", () => {
    const sound = new DiceSound();
    expect(() => {
      sound.playThrow(4);
      sound.playSettle();
    }).not.toThrow();
    sound.enabled = false;
    expect(() => sound.playThrow(2)).not.toThrow();
    sound.dispose();
  });

  it("schedules knocks through Web Audio when it is available", () => {
    const start = vi.fn();
    const node = () => ({ connect: vi.fn((next: unknown) => next), start, frequency: { value: 0 }, Q: { value: 0 }, gain: { value: 0 } });
    class FakeContext {
      state = "running";
      currentTime = 0;
      sampleRate = 8000;
      destination = {};
      createBuffer = (_c: number, length: number) => ({ getChannelData: () => new Float32Array(length) });
      createBufferSource = node;
      createBiquadFilter = node;
      createGain = node;
      close = vi.fn(async () => {});
    }
    vi.stubGlobal("AudioContext", FakeContext);
    try {
      const sound = new DiceSound();
      sound.playThrow(2);
      expect(start).toHaveBeenCalledTimes(8);
      sound.volume = 0;
      sound.playSettle();
      expect(start).toHaveBeenCalledTimes(8);
      sound.dispose();
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
