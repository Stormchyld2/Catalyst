/**
 * Roll history for the running Catalyst session.
 *
 * Phase 1 keeps history in memory: it survives closing and reopening the tray
 * and switching workspaces, and ends when Catalyst quits. Catalyst has no
 * campaign session model yet, and the spec says temporary rolls outside an
 * active campaign need no storage. When campaign sessions land (Phase 3), a
 * RollHistorySink writes each record to the campaign's project files through
 * the project services; see docs/architecture/DICE-RESULT-PROTOCOL.md.
 */
import type { DieType, RollRecord } from "../types/dice";

export interface RollHistorySink {
  append(record: RollRecord): Promise<void>;
}

export interface RollHistoryFilter {
  sessionId?: string | null;
  dieType?: DieType;
  characterId?: string;
  purpose?: string;
}

export const MAX_SESSION_ROLLS = 500;

export class RollHistory {
  private records = $state.raw<RollRecord[]>([]);
  private sink: RollHistorySink | null = null;

  /** Newest first. */
  get all(): readonly RollRecord[] {
    return this.records;
  }

  get latest(): RollRecord | null {
    return this.records[0] ?? null;
  }

  setSink(sink: RollHistorySink | null) {
    this.sink = sink;
  }

  add(record: RollRecord): void {
    this.records = [record, ...this.records].slice(0, MAX_SESSION_ROLLS);
    void this.sink?.append(record).catch((error) => {
      console.error("Failed to save dice roll:", error);
    });
  }

  clear(): void {
    this.records = [];
  }

  filter(filter: RollHistoryFilter): RollRecord[] {
    return this.records.filter(
      (record) =>
        (filter.sessionId === undefined ||
          record.sessionId === filter.sessionId) &&
        (!filter.dieType ||
          record.dice.some((die) => die.type === filter.dieType)) &&
        (!filter.characterId || record.characterId === filter.characterId) &&
        (!filter.purpose || record.purpose === filter.purpose),
    );
  }
}
