// New Campaign Wizard: option lists and the payload sent to `create_new_campaign`.
// IDs here must match src-tauri/src/campaign_wizard.rs, which validates them.

export const GAME_SYSTEMS = [
  "Fifth Edition (5e)",
  "Pathfinder 2e",
  "Call of Cthulhu",
  "Savage Worlds",
  "Fate",
  "Powered by the Apocalypse",
  "Blades in the Dark",
  "Old-School (OSR)",
  "Homebrew",
] as const;

export const CAMPAIGN_STARTERS = [
  { id: "blank", name: "Blank Campaign", description: "Folders and the campaign overview only." },
  { id: "adventure-path", name: "Adventure Path", description: "Three arcs that build to a finale." },
  { id: "sandbox", name: "Sandbox / Hexcrawl", description: "A region, faction clocks and rumors." },
  { id: "mystery", name: "Mystery & Intrigue", description: "A central mystery with clues and suspects." },
  { id: "dungeon", name: "Dungeon Delve", description: "One deep location with levels and denizens." },
  { id: "one-shot", name: "One-Shot", description: "A single three-scene adventure." },
] as const;

export const WORLD_CATEGORIES = [
  { id: "factions", label: "Factions & Governments" },
  { id: "organizations", label: "Organizations & Guilds" },
  { id: "cultures", label: "Cultures & Peoples" },
  { id: "religions", label: "Religions & Pantheons" },
  { id: "history", label: "History & Lore" },
  { id: "magic", label: "Magic & Technology" },
  { id: "bestiary", label: "Bestiary & Creatures" },
  { id: "items", label: "Items & Artifacts" },
  { id: "languages", label: "Languages" },
] as const;

export const GM_TEMPLATES = [
  { id: "session-prep", title: "Session Prep" },
  { id: "session-log", title: "Session Log" },
  { id: "npc-card", title: "NPC Quick Card" },
  { id: "location-card", title: "Location Card" },
  { id: "faction-card", title: "Faction Card" },
  { id: "quest-hook", title: "Quest / Adventure Hook" },
  { id: "encounter", title: "Encounter" },
  { id: "loot-table", title: "Treasure & Loot" },
  { id: "random-table", title: "Random Table" },
  { id: "handout", title: "Player Handout" },
] as const;

export const SAFETY_TOOLS = ["Session Zero", "Lines & Veils", "X-Card", "Open Door", "Stars & Wishes"] as const;

export const GUIDED_STEPS = [
  "Campaign Basics",
  "Players & Table",
  "World Overview",
  "Worldbuilding Categories",
  "Locations",
  "NPCs & Villains",
  "Story & Hooks",
  "First Session",
  "Game Master Toolkit",
  "Review & Create",
] as const;

export type SetupMode = "quick-start" | "guided";

export interface LocationDraft { name: string; location_type: string; description: string }
export interface NpcDraft { name: string; role: string; affiliation: string; description: string; antagonist: boolean }
export interface CategoryDraft { enabled: boolean; notes: string; entries: string }

export interface CampaignDraft {
  mode: SetupMode;
  projectRoot: string;
  name: string;
  gameSystem: string;
  edition: string;
  genre: string;
  tone: string;
  pitch: string;
  playerCount: string;
  startingLevel: string;
  sessionLength: string;
  cadence: string;
  playMode: string;
  safetyTools: string[];
  lines: string;
  veils: string;
  setting: string;
  era: string;
  magicLevel: string;
  technologyLevel: string;
  worldNotes: string;
  categories: Record<string, CategoryDraft>;
  locations: LocationDraft[];
  npcs: NpcDraft[];
  centralConflict: string;
  mainThreat: string;
  themes: string;
  hooks: string;
  starterId: string;
  sessionZeroNotes: string;
  openingScene: string;
  sessionOneNotes: string;
  gmTemplateIds: string[];
  houseRules: string;
}

export function emptyDraft(): CampaignDraft {
  return {
    mode: "quick-start",
    projectRoot: "",
    name: "",
    gameSystem: "",
    edition: "",
    genre: "",
    tone: "",
    pitch: "",
    playerCount: "",
    startingLevel: "",
    sessionLength: "",
    cadence: "",
    playMode: "",
    safetyTools: [],
    lines: "",
    veils: "",
    setting: "",
    era: "",
    magicLevel: "",
    technologyLevel: "",
    worldNotes: "",
    categories: Object.fromEntries(
      WORLD_CATEGORIES.map((category) => [category.id, { enabled: false, notes: "", entries: "" }])
    ),
    locations: [],
    npcs: [],
    centralConflict: "",
    mainThreat: "",
    themes: "",
    hooks: "",
    starterId: "adventure-path",
    sessionZeroNotes: "",
    openingScene: "",
    sessionOneNotes: "",
    gmTemplateIds: GM_TEMPLATES.map((template) => template.id),
    houseRules: "",
  };
}

export function csv(value: string): string[] {
  return value.split(",").map((part) => part.trim()).filter(Boolean);
}

export function lines(value: string): string[] {
  return value.split(/\r?\n/).map((part) => part.trim()).filter(Boolean);
}

/** Returns an error message for a guided step, or null when it may advance. */
export function validateGuidedStep(step: number, draft: CampaignDraft): string | null {
  if (step === 0) return validateBasics(draft);
  if (step === 1 && draft.playerCount.trim()) {
    const count = Number(draft.playerCount);
    if (!Number.isInteger(count) || count < 1 || count > 50) return "Player count must be a whole number from 1 to 50.";
  }
  if (step === 2 && ![draft.setting, draft.era, draft.magicLevel, draft.technologyLevel, draft.worldNotes].some((value) => value.trim())) {
    return "Enter at least one world detail. You can expand it later.";
  }
  if (step === 4 && draft.locations.some((location) => !location.name.trim())) {
    return "Each location needs a name. Remove unused rows or enter a name.";
  }
  if (step === 5 && draft.npcs.some((npc) => !npc.name.trim())) {
    return "Each NPC needs a name. Remove unused rows or enter a name.";
  }
  return null;
}

export function validateBasics(draft: CampaignDraft): string | null {
  if (!draft.name.trim()) return "Enter a campaign name.";
  if (!draft.projectRoot.trim()) return "Choose an empty local folder for the campaign.";
  return null;
}

/** Every guided step must pass before Create; returns the first failing step. */
export function firstInvalidStep(draft: CampaignDraft): { step: number; error: string } | null {
  for (let step = 0; step < GUIDED_STEPS.length - 1; step++) {
    const error = validateGuidedStep(step, draft);
    if (error) return { step, error };
  }
  return null;
}

export function buildCampaignInput(draft: CampaignDraft) {
  const base = {
    project_root: draft.projectRoot.trim(),
    name: draft.name.trim(),
    setup_mode: draft.mode,
    game_system: draft.gameSystem.trim(),
    pitch: draft.pitch.trim(),
    starter_id: draft.starterId,
  };
  if (draft.mode === "quick-start") {
    // Quick Start: the backend installs every GM template and seeds defaults.
    return { ...base, gm_template_ids: null };
  }
  return {
    ...base,
    edition: draft.edition.trim(),
    genre: draft.genre.trim(),
    tone: draft.tone.trim(),
    table: {
      player_count: draft.playerCount.trim() ? Number(draft.playerCount) : null,
      starting_level: draft.startingLevel.trim(),
      session_length: draft.sessionLength.trim(),
      cadence: draft.cadence.trim(),
      play_mode: draft.playMode.trim(),
      safety_tools: draft.safetyTools,
      lines: csv(draft.lines),
      veils: csv(draft.veils),
    },
    world: {
      setting: draft.setting.trim(),
      era: draft.era.trim(),
      magic_level: draft.magicLevel.trim(),
      technology_level: draft.technologyLevel.trim(),
      notes: draft.worldNotes.trim(),
    },
    world_categories: WORLD_CATEGORIES.filter((category) => draft.categories[category.id]?.enabled).map((category) => ({
      id: category.id,
      notes: draft.categories[category.id].notes.trim(),
      entries: csv(draft.categories[category.id].entries),
    })),
    locations: draft.locations.map((location) => ({
      name: location.name.trim(),
      location_type: location.location_type.trim(),
      description: location.description.trim(),
    })),
    npcs: draft.npcs.map((npc) => ({
      name: npc.name.trim(),
      role: npc.role.trim(),
      affiliation: npc.affiliation.trim(),
      description: npc.description.trim(),
      antagonist: npc.antagonist,
    })),
    story: {
      central_conflict: draft.centralConflict.trim(),
      main_threat: draft.mainThreat.trim(),
      themes: csv(draft.themes),
      hooks: lines(draft.hooks),
    },
    first_session: {
      session_zero_notes: draft.sessionZeroNotes.trim(),
      opening_scene: draft.openingScene.trim(),
      session_one_notes: draft.sessionOneNotes.trim(),
    },
    gm_template_ids: [...draft.gmTemplateIds],
    house_rules: draft.houseRules.trim(),
  };
}
