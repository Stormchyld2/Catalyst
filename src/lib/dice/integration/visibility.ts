/**
 * Who may see a roll. Anything that sends rolls to another screen (a player
 * display, a LAN client) must filter with `canView` before sending: privacy is
 * enforced by not transmitting a record, never by hiding it in the UI.
 */
import type { RollRecord } from "../types/dice";

export type RollViewer = { role: "gm" } | { role: "player"; playerId: string } | { role: "public-display" };

export function canView(record: RollRecord, viewer: RollViewer): boolean {
  if (viewer.role === "gm") return true;
  switch (record.visibility) {
    case "public":
      return true;
    case "gm-private":
      return false;
    case "player-private":
      return viewer.role === "player" && !!record.playerId && record.playerId === viewer.playerId;
  }
}

/** The records `viewer` may receive, in the same order. */
export function visibleTo(records: readonly RollRecord[], viewer: RollViewer): RollRecord[] {
  return records.filter((record) => canView(record, viewer));
}
