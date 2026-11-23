// New Campaign Wizard (docs/architecture/020-CAMPAIGN-WIZARD-ARCHITECTURE.md):
// option lists, temporary draft state, validation, the planned-files preview
// and the payload sent to `create_new_campaign`. IDs must match
// src-tauri/src/campaign_wizard.rs, which validates them again.

export const RULES_PACKS = [
  { id: "system-neutral", name: "System-Neutral", description: "No rules assumptions. Works with any game." },
  { id: "custom-homebrew", name: "Custom / Homebrew", description: "Your own or another system, named by you." },
] as const;

export const WORLD_CATEGORIES = [
  { id: "geography", label: "Geography & Climate" },
  { id: "government", label: "Governments & Politics" },
  { id: "nations", label: "Kingdoms & Nations" },
  { id: "settlements", label: "Settlements & Locations" },
  { id: "history", label: "History & Timelines" },
  { id: "lore", label: "Lore & Legends" },
  { id: "cultures", label: "Cultures & Species" },
  { id: "religions", label: "Religions & Deities" },
  { id: "magic", label: "Magic & Supernatural" },
  { id: "languages", label: "Languages" },
  { id: "economy", label: "Economy & Trade" },
  { id: "technology", label: "Technology & Crafting" },
  { id: "factions", label: "Organizations & Factions" },
  { id: "laws", label: "Laws & Customs" },
  { id: "natural", label: "Natural World" },
] as const;

export const GUIDED_STEPS = [
  "Campaign Basics",
  "Rules & Game System",
  "World & Setting",
  "Worldbuilding Foundations",
  "Players & Characters",
  "NPCs, Creatures & Factions",
  "Adventure & Campaign Story",
  "Locations & Maps",
  "Session Setup",
  "Review & Create",
] as const;

export type SetupMode = "quick-start" | "guided";

export interface PlayerDraft { player_name: string; character_name: string }
export interface NpcDraft { name: string; disposition: "ally" | "neutral" | "villain"; role: string; motivation: string; description: string }
export interface EntryDraft { name: string; kind: string; description: string }
export interface CategoryDraft { enabled: boolean; names: string; notes: string }

export interface CampaignDraft {
  mode: SetupMode;
  projectRoot: string;
  name: string;
  subtitle: string;
  premise: string;
  genre: string;
  tone: string;
  gmName: string;
  startDate: string;
  rulesPackId: string;
  customSystemName: string;
  advancement: string;
  startingLevel: string;
  difficulty: string;
  houseRules: string;
  worldMode: "new" | "blank";
  worldName: string;
  era: string;
  calendar: string;
  geography: string;
  worldNotes: string;
  categories: Record<string, CategoryDraft>;
  partyName: string;
  startingProgression: string;
  players: PlayerDraft[];
  npcs: NpcDraft[];
  creatures: EntryDraft[];
  factions: EntryDraft[];
  centralConflict: string;
  startingAdventure: string;
  startingAdventureSummary: string;
  arcs: string;
  sideQuests: string;
  startingLocation: string;
  locations: EntryDraft[];
  mapReferences: string;
  firstSessionTitle: string;
  firstSessionNotes: string;
  gmNotes: string;
  playMode: "physical" | "hybrid" | "digital";
  dice: "physical" | "digital";
  playerDisplay: boolean;
}

export function emptyDraft(): CampaignDraft {
  return {
    mode: "quick-start",
    projectRoot: "",
    name: "",
    subtitle: "",
    premise: "",
    genre: "",
    tone: "",
    gmName: "",
    startDate: "",
    rulesPackId: "system-neutral",
    customSystemName: "",
    advancement: "",
    startingLevel: "",
    difficulty: "",
    houseRules: "",
    worldMode: "new",
    worldName: "",
    era: "",
    calendar: "",
    geography: "",
    worldNotes: "",
    categories: Object.fromEntries(WORLD_CATEGORIES.map((c) => [c.id, { enabled: false, names: "", notes: "" }])),
    partyName: "",
    startingProgression: "",
    players: [],
    npcs: [],
    creatures: [],
    factions: [],
    centralConflict: "",
    startingAdventure: "",
    startingAdventureSummary: "",
    arcs: "",
    sideQuests: "",
    startingLocation: "",
    locations: [],
    mapReferences: "",
    firstSessionTitle: "",
    firstSessionNotes: "",
    gmNotes: "",
    playMode: "physical",
    dice: "physical",
    playerDisplay: false,
  };
}

export function csv(value: string): string[] {
  return value.split(",").map((part) => part.trim()).filter(Boolean);
}

export function lines(value: string): string[] {
  return value.split(/\r?\n/).map((part) => part.trim()).filter(Boolean);
}

/** Error for the essentials every campaign needs (Quick Start and step 1). */
export function validateEssentials(draft: CampaignDraft): string | null {
  if (!draft.name.trim()) return "Enter a campaign name.";
  if (!draft.projectRoot.trim()) return "Choose a local folder for the campaign.";
  if (draft.rulesPackId === "custom-homebrew" && !draft.customSystemName.trim()) return "Name your custom or homebrew system.";
  return null;
}

/** Returns an error for a guided step, or null when it may advance. All
 *  content is optional; only names of rows the GM added are required. */
export function validateGuidedStep(step: number, draft: CampaignDraft): string | null {
  if (step === 0 && !draft.name.trim()) return "Enter a campaign name.";
  if (step === 0 && !draft.projectRoot.trim()) return "Choose a local folder for the campaign.";
  if (step === 1 && draft.rulesPackId === "custom-homebrew" && !draft.customSystemName.trim()) return "Name your custom or homebrew system.";
  if (step === 5) {
    if (draft.npcs.some((row) => !row.name.trim())) return "Each NPC needs a name. Remove unused rows or enter a name.";
    if (draft.creatures.some((row) => !row.name.trim())) return "Each creature needs a name. Remove unused rows or enter a name.";
    if (draft.factions.some((row) => !row.name.trim())) return "Each faction needs a name. Remove unused rows or enter a name.";
  }
  if (step === 7 && draft.locations.some((row) => !row.name.trim())) return "Each location needs a name. Remove unused rows or enter a name.";
  return null;
}

export function firstInvalidStep(draft: CampaignDraft): { step: number; error: string } | null {
  for (let step = 0; step < GUIDED_STEPS.length - 1; step++) {
    const error = validateGuidedStep(step, draft);
    if (error) return { step, error };
  }
  return null;
}

function safeName(value: string): string {
  const cleaned = value.trim().replace(/[\u0000-\u001f<>:"/\\|?*]/g, "-").trim().replace(/[. ]+$/, "").trim();
  return cleaned || "Untitled";
}

function shortTitle(text: string): string {
  const first = text.split(/\r?\n/)[0].trim();
  return first.length > 60 ? `${first.slice(0, 60).trimEnd()}…` : first || "Untitled";
}

/** The documents Create will write, for the Review step's preview. Folders
 *  are always the full approved tree. */
export function plannedFiles(draft: CampaignDraft): string[] {
  const files = ["project.yaml", "campaign.yaml", "Adventures/Main Campaign/Campaign Overview.md"];
  if (draft.mode === "quick-start") return files;
  const add = (folder: string, title: string) => files.push(`${folder}/${safeName(title)}.md`);
  if (draft.houseRules.trim()) add("Rules & Mechanics/House Rules", "House Rules");
  if ([draft.worldName, draft.era, draft.calendar, draft.geography, draft.worldNotes].some((v) => v.trim())) {
    add("Worldbuilding", draft.worldName.trim() || "World Overview");
  }
  for (const category of WORLD_CATEGORIES) {
    const value = draft.categories[category.id];
    if (!value?.enabled) continue;
    if (value.notes.trim()) add(`Worldbuilding/${category.label}`, `${category.label} Notes`);
    for (const name of csv(value.names)) add(`Worldbuilding/${category.label}`, name);
  }
  for (const player of draft.players) if (player.character_name.trim()) add("Characters/Player Characters", player.character_name);
  for (const npc of draft.npcs) add("Characters/Non-Player Characters", npc.name);
  for (const creature of draft.creatures) add("Creatures & Monsters/Custom Creatures", creature.name);
  for (const faction of draft.factions) add("Worldbuilding/Organizations & Factions", faction.name);
  if (draft.startingAdventure.trim()) add("Adventures/Main Campaign", draft.startingAdventure);
  for (const arc of lines(draft.arcs)) add("Adventures/Main Campaign", shortTitle(arc));
  for (const quest of lines(draft.sideQuests)) add("Adventures/Side Quests", shortTitle(quest));
  for (const location of draft.locations) add("Worldbuilding/Settlements & Locations", location.name);
  const start = draft.startingLocation.trim();
  if (start && !draft.locations.some((l) => l.name.trim().toLowerCase() === start.toLowerCase())) {
    add("Worldbuilding/Settlements & Locations", start);
  }
  if (lines(draft.mapReferences).length) add("Maps", "Map References");
  if (draft.firstSessionTitle.trim() || draft.firstSessionNotes.trim()) add("Sessions/Session Plans", draft.firstSessionTitle.trim() || "Session 1");
  if (draft.gmNotes.trim()) add("Game Master/GM Notes", "GM Notes");
  return files;
}

export function buildCampaignInput(draft: CampaignDraft) {
  const base = {
    project_root: draft.projectRoot.trim(),
    name: draft.name.trim(),
    setup_mode: draft.mode,
    basics: { premise: draft.premise.trim() },
    rules: { pack_id: draft.rulesPackId, custom_system_name: draft.customSystemName.trim() },
  };
  if (draft.mode === "quick-start") return base;
  const entry = (row: EntryDraft) => ({ name: row.name.trim(), kind: row.kind.trim(), description: row.description.trim() });
  return {
    ...base,
    basics: {
      subtitle: draft.subtitle.trim(),
      premise: draft.premise.trim(),
      genre: draft.genre.trim(),
      tone: draft.tone.trim(),
      gm_name: draft.gmName.trim(),
      start_date: draft.startDate.trim(),
    },
    rules: {
      pack_id: draft.rulesPackId,
      custom_system_name: draft.customSystemName.trim(),
      advancement: draft.advancement.trim(),
      starting_level: draft.startingLevel.trim(),
      difficulty: draft.difficulty.trim(),
      house_rules: draft.houseRules.trim(),
    },
    world: {
      mode: draft.worldMode,
      name: draft.worldName.trim(),
      era: draft.era.trim(),
      calendar: draft.calendar.trim(),
      geography: draft.geography.trim(),
      notes: draft.worldNotes.trim(),
    },
    worldbuilding: WORLD_CATEGORIES.filter((c) => draft.categories[c.id]?.enabled).map((c) => ({
      id: c.id,
      names: csv(draft.categories[c.id].names),
      notes: draft.categories[c.id].notes.trim(),
    })),
    party: {
      name: draft.partyName.trim(),
      starting_progression: draft.startingProgression.trim(),
      players: draft.players
        .map((p) => ({ player_name: p.player_name.trim(), character_name: p.character_name.trim() }))
        .filter((p) => p.player_name || p.character_name),
    },
    npcs: draft.npcs.map((n) => ({
      name: n.name.trim(),
      disposition: n.disposition,
      role: n.role.trim(),
      motivation: n.motivation.trim(),
      description: n.description.trim(),
    })),
    creatures: draft.creatures.map(entry),
    factions: draft.factions.map(entry),
    story: {
      central_conflict: draft.centralConflict.trim(),
      starting_adventure: draft.startingAdventure.trim(),
      starting_adventure_summary: draft.startingAdventureSummary.trim(),
      arcs: lines(draft.arcs),
      side_quests: lines(draft.sideQuests),
    },
    locations: {
      starting_location: draft.startingLocation.trim(),
      entries: draft.locations.map(entry),
      map_references: lines(draft.mapReferences),
    },
    session: {
      first_session_title: draft.firstSessionTitle.trim(),
      first_session_notes: draft.firstSessionNotes.trim(),
      gm_notes: draft.gmNotes.trim(),
      play_mode: draft.playMode,
      dice: draft.dice,
      player_display: draft.playerDisplay,
    },
  };
}
