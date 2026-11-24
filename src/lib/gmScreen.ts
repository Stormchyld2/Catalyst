/**
 * GM Screen layout (doc 034). The authoritative copy is the campaign's
 * `Game Master/gm-screen.json`, read and written by the Rust backend. Updates
 * spread the existing objects so fields from newer phases are kept.
 */

export type GmPanelId = "soundboard" | "dice" | "rolls";

export interface GmPanel {
  id: GmPanelId;
  visible: boolean;
  /** Soundboard panel only: the pad tag to show; null shows every pad. */
  pad_tag?: string | null;
}

export interface GmScreenLayout {
  format: "catalyst-gm-screen";
  version: number;
  active_layout: string;
  layouts: Record<string, { panels: GmPanel[] }>;
}

export const GM_PANELS: readonly { id: GmPanelId; label: string }[] = [
  { id: "soundboard", label: "Soundboard" },
  { id: "dice", label: "Dice" },
  { id: "rolls", label: "Rolls" },
] as const;

export function defaultGmScreen(): GmScreenLayout {
  return {
    format: "catalyst-gm-screen",
    version: 1,
    active_layout: "default",
    layouts: { default: { panels: GM_PANELS.map(({ id }) => ({ id, visible: true })) } },
  };
}

/** The active layout's panel, or a visible default if the file doesn't list it. */
export function gmPanel(layout: GmScreenLayout, id: GmPanelId): GmPanel {
  const panels = layout.layouts[layout.active_layout]?.panels ?? [];
  return panels.find((p) => p.id === id) ?? { id, visible: true };
}

export function updateGmPanel(layout: GmScreenLayout, id: GmPanelId, patch: Partial<GmPanel>): GmScreenLayout {
  const key = layout.active_layout;
  const current = layout.layouts[key] ?? { panels: [] };
  const exists = current.panels.some((p) => p.id === id);
  const panels = exists
    ? current.panels.map((p) => (p.id === id ? { ...p, ...patch } : p))
    : [...current.panels, { ...gmPanel(layout, id), ...patch }];
  return { ...layout, layouts: { ...layout.layouts, [key]: { ...current, panels } } };
}
