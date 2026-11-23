/**
 * Campaign session history in project files, through the Rust project service
 * (src-tauri/src/dice_sessions.rs):
 *
 *   <campaign>/Sessions/Roll Logs/<session-id>/session.json
 *   <campaign>/Sessions/Roll Logs/<session-id>/rolls.jsonl
 *
 * (Book-layout projects use Story Development/Sessions/Roll Logs/ instead.)
 *
 * The files are authoritative and readable without Catalyst. The service
 * validates every record before writing it. No database is involved.
 */
import { invoke } from "@tauri-apps/api/core";
import type { RollRecord } from "../types/dice";
import type { RollHistorySink } from "./sessionHistory.svelte";

export interface DiceSessionInfo {
  id: string;
  title: string;
  startedAt: string;
}

export interface DiceSessionHistory {
  session: DiceSessionInfo;
  /** Oldest first, as stored. */
  rolls: RollRecord[];
  /** Lines in the file that could not be read back. */
  skippedLines: number;
}

/** The project-service calls the dice roller needs. Swappable in tests. */
export interface DiceSessionFiles {
  start(projectRoot: string, title: string): Promise<DiceSessionInfo>;
  append(projectRoot: string, sessionId: string, record: RollRecord): Promise<void>;
  read(projectRoot: string, sessionId: string): Promise<DiceSessionHistory>;
  list(projectRoot: string): Promise<DiceSessionInfo[]>;
}

export const diceSessionFiles: DiceSessionFiles = {
  start: (projectRoot, title) => invoke("start_dice_session", { projectRoot, title }),
  append: (projectRoot, sessionId, record) =>
    invoke("append_dice_roll", { projectRoot, sessionId, record }),
  read: (projectRoot, sessionId) => invoke("get_dice_session_history", { projectRoot, sessionId }),
  list: (projectRoot) => invoke("list_dice_sessions", { projectRoot }),
};

/** Appends each recorded roll to the active session's file. */
export class CampaignSessionSink implements RollHistorySink {
  constructor(
    private readonly files: DiceSessionFiles,
    private readonly projectRoot: string,
    private readonly sessionId: string,
    private readonly onError: (error: unknown) => void = () => {}
  ) {}

  async append(record: RollRecord): Promise<void> {
    // Rolls made outside this session (an explicit other sessionId) are not ours to save.
    if (record.sessionId !== this.sessionId) return;
    try {
      await this.files.append(this.projectRoot, this.sessionId, record);
    } catch (error) {
      this.onError(error);
      throw error;
    }
  }
}
