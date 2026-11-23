/**
 * Roll history for the running Catalyst session.
 *
 * History is kept in memory for the running app: it survives closing and
 * reopening the tray and switching workspaces. Temporary rolls outside a
 * campaign session are not stored. While a campaign session is active, a
 * RollHistorySink (CampaignSessionSink) also appends each record to the
 * session's file in the project; see docs/architecture/029-DICE-RESULT-PROTOCOL.md.
 */
import type { DieType, RollRecord } from "../types/dice";

export interface RollHistorySink {
  append(record: RollRecord): Promise<void>;
}

export interface RollHistoryFilter {
  sessionId?: string | null;
  dieType?: DieType;
  playerId?: string;
  characterId?: string;
  npcId?: string;
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

  /** Replace the in-memory history, e.g. with a reopened session's saved rolls (newest first). */
  replace(records: RollRecord[]): void {
    this.records = records.slice(0, MAX_SESSION_ROLLS);
  }

  filter(filter: RollHistoryFilter): RollRecord[] {
    return this.records.filter(
      (record) =>
        (filter.sessionId === undefined ||
          record.sessionId === filter.sessionId) &&
        (!filter.dieType ||
          record.dice.some((die) => die.type === filter.dieType)) &&
        (!filter.playerId || record.playerId === filter.playerId) &&
        (!filter.characterId || record.characterId === filter.characterId) &&
        (!filter.npcId || record.npcId === filter.npcId) &&
        (!filter.purpose || record.purpose === filter.purpose),
    );
  }
}
