// New Campaign Wizard (docs/architecture/020-CAMPAIGN-WIZARD-ARCHITECTURE.md):
// question lists, temporary draft state, validation, the planned-files preview
// and the payload sent to `create_new_campaign`. IDs must match
// src-tauri/src/campaign_wizard.rs, which validates them again.

export interface Choice { id: string; label: string }

/** Progression terminology a rules pack can use. `input` labels the value field. */
export interface ProgressionOption extends Choice { input: string; placeholder: string }

/**
 * Installed, verified rules packs. Only packs that actually ship with Catalyst
 * belong here. Published systems appear once a verified pack is installed, so
 * no commercial game is listed as available before then.
 */
export interface RulesPack {
  id: string;
  name: string;
  description: string;
  /** Editions offered by the pack. Empty means the GM types one in. */
  editions: Choice[];
  progressions: ProgressionOption[];
  advancement: Choice[];
}

const ALL_PROGRESSIONS: ProgressionOption[] = [
  { id: "level", label: "Starting level", input: "Starting level", placeholder: "1" },
  { id: "rank", label: "Rank", input: "Starting rank", placeholder: "Novice" },
  { id: "tier", label: "Tier", input: "Starting tier", placeholder: "Tier 1" },
  { id: "experience", label: "Experience", input: "Starting experience", placeholder: "0 XP" },
  { id: "other", label: "Other", input: "Starting progression", placeholder: "Describe how characters start" },
];

const ALL_ADVANCEMENT: Choice[] = [
  { id: "ruleset-default", label: "Ruleset Default" },
  { id: "experience", label: "Experience Points" },
  { id: "milestones", label: "Milestones" },
  { id: "gm-directed", label: "GM-Directed" },
  { id: "custom", label: "Custom" },
];

export const RULES_PACKS: RulesPack[] = [
  {
    id: "system-neutral",
    name: "System-Neutral",
    description: "No rules assumptions. Works with any game.",
    editions: [{ id: "any", label: "Any / not specified" }],
    progressions: ALL_PROGRESSIONS,
    advancement: ALL_ADVANCEMENT,
  },
  {
    id: "custom-homebrew",
    name: "Custom / Homebrew",
    description: "Your own game, or a system Catalyst doesn't have a pack for yet.",
    editions: [],
    progressions: ALL_PROGRESSIONS,
    advancement: ALL_ADVANCEMENT,
  },
];

export const GENRES: Choice[] = [
  { id: "high-fantasy", label: "High Fantasy" },
  { id: "dark-fantasy", label: "Dark Fantasy" },
  { id: "science-fiction", label: "Science Fiction" },
  { id: "horror", label: "Horror" },
  { id: "modern", label: "Modern / Contemporary" },
  { id: "post-apocalyptic", label: "Post-Apocalyptic" },
  { id: "historical", label: "Historical" },
  { id: "steampunk", label: "Steampunk" },
  { id: "superhero", label: "Superhero" },
  { id: "mixed", label: "Mixed Genre" },
  { id: "custom", label: "Custom" },
];

export const TONES: Choice[] = [
  { id: "heroic", label: "Heroic" },
  { id: "dark", label: "Dark / Gritty" },
  { id: "epic", label: "Epic" },
  { id: "mysterious", label: "Mysterious" },
  { id: "political", label: "Political / Intrigue" },
  { id: "survival", label: "Survival" },
  { id: "lighthearted", label: "Lighthearted / Humorous" },
  { id: "horror", label: "Horror / Suspense" },
  { id: "mixed", label: "Mixed" },
  { id: "custom", label: "Custom" },
];

export const HOUSE_RULE_MODES: (Choice & { hint: string })[] = [
  { id: "standard", label: "Standard Rules", hint: "Play the rules as written." },
  { id: "modified", label: "Standard Rules with Modifications", hint: "Mostly as written, with a few changes you'll note." },
  { id: "custom", label: "Custom / Homebrew Rules", hint: "Your own rules, or heavy changes." },
];

export const WORLD_MODES: (Choice & { hint: string; available: boolean })[] = [
  { id: "new", label: "Create a New World", hint: "Build the world inside this campaign.", available: true },
  { id: "existing", label: "Use an Existing World", hint: "Not available yet. Sharing one world between campaigns needs the shared world library, which isn't built.", available: false },
  { id: "import", label: "Import a World", hint: "Not available yet. Catalyst can't import world files in this version.", available: false },
  { id: "blank", label: "Start Without a Defined World", hint: "Leave Worldbuilding empty and decide later.", available: true },
];

export const SETTING_ORIGINS: (Choice & { hint: string })[] = [
  { id: "original", label: "Original / Homebrew", hint: "A world you're creating." },
  { id: "published", label: "Published Setting", hint: "Recorded by name only. Catalyst doesn't import published content." },
  { id: "adapted", label: "Adapted Setting", hint: "A published or familiar world with your own changes." },
  { id: "undecided", label: "Undecided", hint: "Decide later." },
];

export const SETTING_SCALES: Choice[] = [
  { id: "settlement", label: "Single Settlement" },
  { id: "region", label: "Region" },
  { id: "kingdom", label: "Kingdom / Nation" },
  { id: "continent", label: "Continent" },
  { id: "world", label: "Entire World" },
  { id: "planes", label: "Multiple Worlds / Planes" },
  { id: "space", label: "Star System / Space Sector" },
  { id: "custom", label: "Custom" },
];

export const ERAS: Choice[] = [
  { id: "ancient", label: "Ancient" },
  { id: "medieval", label: "Medieval" },
  { id: "renaissance", label: "Renaissance" },
  { id: "industrial", label: "Industrial" },
  { id: "modern", label: "Modern" },
  { id: "futuristic", label: "Futuristic" },
  { id: "mixed", label: "Mixed / Anachronistic" },
  { id: "custom", label: "Custom" },
];

export interface CategoryQuestion { key: string; label: string; placeholder?: string }
export interface EntryField { key: string; label: string; options?: string[] }
export interface CategorySpec {
  id: string;
  label: string;
  questions: CategoryQuestion[];
  /** Named records the GM can add, each saved as its own wiki page. */
  entries?: { noun: string; prompt: string; fields: EntryField[] };
}

/** Worldbuilding categories and their follow-up questions (020 §5). */
export const WORLD_CATEGORIES: CategorySpec[] = [
  {
    id: "geography",
    label: "Geography & Climate",
    questions: [
      { key: "terrain", label: "What are the major terrain types?", placeholder: "Mountains, salt flats, river deltas…" },
      { key: "climate", label: "What is the general climate?", placeholder: "Cold north, monsoon south…" },
      { key: "features", label: "Are there important natural hazards or unusual geographical features?" },
    ],
  },
  {
    id: "government",
    label: "Governments & Politics",
    questions: [
      { key: "forms", label: "What forms of government exist?", placeholder: "Monarchy, merchant council, theocracy…" },
      { key: "power", label: "Who holds power?" },
      { key: "conflicts", label: "Are there important political conflicts?" },
    ],
  },
  {
    id: "nations",
    label: "Kingdoms & Nations",
    questions: [],
    entries: {
      noun: "Nation",
      prompt: "What major nations or territories exist?",
      fields: [
        { key: "ruler", label: "Who controls it?" },
        { key: "relations", label: "Allies, rivals or enemies?" },
      ],
    },
  },
  {
    id: "settlements",
    label: "Settlements & Locations",
    questions: [],
    entries: {
      noun: "Settlement",
      prompt: "What important settlements exist?",
      fields: [
        { key: "purpose", label: "What purpose does it serve?" },
        { key: "location", label: "Where is it located?" },
      ],
    },
  },
  {
    id: "history",
    label: "History & Timelines",
    questions: [
      { key: "events", label: "What major historical events shaped the world?" },
      { key: "eras", label: "Are there important dates, eras or conflicts?" },
    ],
  },
  {
    id: "lore",
    label: "Lore & Legends",
    questions: [],
    entries: {
      noun: "Legend",
      prompt: "What legends, myths or mysteries are important?",
      fields: [
        { key: "truth", label: "Is it fact, rumor or unknown truth?", options: ["Established fact", "Rumor", "Unknown truth"] },
        { key: "summary", label: "What do people say?" },
      ],
    },
  },
  {
    id: "cultures",
    label: "Cultures & Species",
    questions: [],
    entries: {
      noun: "Culture",
      prompt: "What cultures or peoples are present?",
      fields: [{ key: "traits", label: "What languages, customs or relationships distinguish them?" }],
    },
  },
  {
    id: "religions",
    label: "Religions & Deities",
    questions: [{ key: "importance", label: "Are religions or supernatural beliefs important?" }],
    entries: {
      noun: "Faith",
      prompt: "Which organizations, deities or philosophies are influential?",
      fields: [{ key: "influence", label: "Why does it matter?" }],
    },
  },
  {
    id: "magic",
    label: "Magic & Supernatural",
    questions: [
      { key: "exists", label: "Does magic or the supernatural exist?" },
      { key: "common", label: "How common is it?" },
      { key: "limits", label: "Are there restrictions, institutions or consequences?" },
    ],
  },
  {
    id: "languages",
    label: "Languages",
    questions: [],
    entries: {
      noun: "Language",
      prompt: "What major languages exist?",
      fields: [{ key: "speakers", label: "Which cultures or regions use it?" }],
    },
  },
  {
    id: "economy",
    label: "Economy & Trade",
    questions: [
      { key: "currency", label: "What forms of currency or trade exist?" },
      { key: "resources", label: "What resources are valuable or scarce?" },
      { key: "trade", label: "Who controls trade, and where does it flow?" },
    ],
  },
  {
    id: "technology",
    label: "Technology & Crafting",
    questions: [
      { key: "level", label: "What technology is common in daily life?" },
      { key: "crafting", label: "Who makes important goods, and how?" },
      { key: "rare", label: "Is any technology rare, forbidden or lost?" },
    ],
  },
  {
    id: "factions",
    label: "Organizations & Factions",
    questions: [],
    entries: {
      noun: "Organization",
      prompt: "Which guilds, orders, cults or companies matter?",
      fields: [
        { key: "goal", label: "What do they want?" },
        { key: "influence", label: "Where do they have influence?" },
      ],
    },
  },
  {
    id: "laws",
    label: "Laws & Customs",
    questions: [
      { key: "laws", label: "What laws matter most to adventurers?" },
      { key: "customs", label: "What customs, taboos or traditions should players know?" },
      { key: "enforcement", label: "Who enforces the law, and how strictly?" },
    ],
  },
  {
    id: "natural",
    label: "Natural World",
    questions: [
      { key: "life", label: "What notable plants and animals live here?" },
      { key: "wonders", label: "Are there natural wonders or strange phenomena?" },
      { key: "dangers", label: "What natural dangers threaten travelers?" },
    ],
  },
];

export const PARTY_ORIGINS: (Choice & { hint: string })[] = [
  { id: "together", label: "Already together", hint: "The characters know each other when play begins." },
  { id: "first-session", label: "Meet in the first session", hint: "Their first meeting is part of the story." },
  { id: "mixed", label: "Some know each other", hint: "A few ties, a few strangers." },
];

export const NPC_DISPOSITIONS: Choice[] = [
  { id: "ally", label: "Ally" },
  { id: "neutral", label: "Neutral" },
  { id: "rival", label: "Rival" },
  { id: "villain", label: "Villain" },
  { id: "unknown", label: "Unknown" },
];

export const NPC_ROLES = ["Patron", "Quest giver", "Mentor", "Merchant", "Authority figure", "Informant", "Rival", "Main villain", "Henchman", "Other"];
export const CREATURE_TYPES = ["Beast", "Monster", "Undead", "Humanoid", "Spirit", "Construct", "Aberration", "Dragon", "Other"];
export const THREAT_LEVELS = ["Minion", "Standard", "Elite", "Boss", "Legendary"];
export const FACTION_ATTITUDES = ["Friendly", "Neutral", "Suspicious", "Hostile", "Unknown"];
export const LOCATION_TYPES = ["Settlement", "Building", "Dungeon", "Wilderness", "Landmark", "Ruin", "Planar / Otherworldly", "Other"];

export const CAMPAIGN_STRUCTURES: (Choice & { hint: string })[] = [
  { id: "sandbox", label: "Sandbox", hint: "Players choose where to go; the world reacts." },
  { id: "episodic", label: "Episodic", hint: "Self-contained adventures, loosely connected." },
  { id: "linear", label: "Story-driven", hint: "A planned storyline with clear chapters." },
  { id: "mixed", label: "Mixed", hint: "A main story with room to wander." },
];

export const CAMPAIGN_LENGTHS: Choice[] = [
  { id: "one-shot", label: "One-shot" },
  { id: "short", label: "Short (2–6 sessions)" },
  { id: "medium", label: "Medium (7–20 sessions)" },
  { id: "long", label: "Long (20+ sessions)" },
  { id: "ongoing", label: "Ongoing / open-ended" },
];

export const PLAY_MODES: (Choice & { hint: string })[] = [
  { id: "physical", label: "In person", hint: "Around a table with books, minis or a battle map." },
  { id: "hybrid", label: "Hybrid", hint: "Some players at the table, some online, or screens at the table." },
  { id: "digital", label: "Online", hint: "Voice or video with a virtual tabletop." },
];

export const SCHEDULES: Choice[] = [
  { id: "Weekly", label: "Weekly" },
  { id: "Every two weeks", label: "Every two weeks" },
  { id: "Monthly", label: "Monthly" },
  { id: "Irregular", label: "When we can" },
];

export const GUIDED_STEPS = [
  "Campaign Identity",
  "RPG System & Rules",
  "World & Setting",
  "Worldbuilding Foundations",
  "Players & Characters",
  "NPCs, Creatures & Factions",
  "Adventure & Campaign Story",
  "Locations & Maps",
  "Session Setup",
  "Review & Create",
] as const;

export type SetupMode = "quick-start" | "guided" | "blank";

export interface PlayerDraft { player_name: string; character_name: string; concept: string }
export interface NpcDraft { name: string; disposition: string; role: string; motivation: string; description: string }
export interface EntryDraft { name: string; kind: string; description: string; detail: string }
export interface WorldEntryDraft { name: string; fields: Record<string, string> }
export interface CategoryDraft { enabled: boolean; answers: Record<string, string>; entries: WorldEntryDraft[]; notes: string }

export interface CampaignDraft {
  mode: SetupMode;
  projectRoot: string;
  name: string;
  premise: string;
  genre: string;
  genreCustom: string;
  tones: string[];
  toneCustom: string;
  gmName: string;
  startDate: string;
  rulesPackId: string;
  customSystemName: string;
  edition: string;
  progressionType: string;
  startingLevel: string;
  advancement: string;
  advancementCustom: string;
  houseRulesMode: string;
  houseRules: string;
  allowedContent: string;
  creationNotes: string;
  worldMode: "new" | "existing" | "import" | "blank";
  worldName: string;
  worldOrigin: string;
  worldScale: string;
  worldScaleCustom: string;
  era: string;
  eraCustom: string;
  worldNotes: string;
  categories: Record<string, CategoryDraft>;
  partyName: string;
  expectedPlayers: string;
  partyOrigin: string;
  players: PlayerDraft[];
  npcs: NpcDraft[];
  creatures: EntryDraft[];
  factions: EntryDraft[];
  storyStructure: string;
  storyLength: string;
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
  schedule: string;
  sessionLength: string;
  sessionZero: boolean;
}

export function emptyCategory(): CategoryDraft {
  return { enabled: false, answers: {}, entries: [], notes: "" };
}

export function emptyDraft(): CampaignDraft {
  return {
    mode: "quick-start",
    projectRoot: "",
    name: "",
    premise: "",
    genre: "",
    genreCustom: "",
    tones: [],
    toneCustom: "",
    gmName: "",
    startDate: "",
    rulesPackId: "system-neutral",
    customSystemName: "",
    edition: "",
    progressionType: "",
    startingLevel: "",
    advancement: "",
    advancementCustom: "",
    houseRulesMode: "",
    houseRules: "",
    allowedContent: "",
    creationNotes: "",
    worldMode: "new",
    worldName: "",
    worldOrigin: "",
    worldScale: "",
    worldScaleCustom: "",
    era: "",
    eraCustom: "",
    worldNotes: "",
    categories: Object.fromEntries(WORLD_CATEGORIES.map((c) => [c.id, emptyCategory()])),
    partyName: "",
    expectedPlayers: "",
    partyOrigin: "",
    players: [],
    npcs: [],
    creatures: [],
    factions: [],
    storyStructure: "",
    storyLength: "",
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
    schedule: "",
    sessionLength: "",
    sessionZero: false,
  };
}

export function csv(value: string): string[] {
  return value.split(",").map((part) => part.trim()).filter(Boolean);
}

export function lines(value: string): string[] {
  return value.split(/\r?\n/).map((part) => part.trim()).filter(Boolean);
}

export function rulesPack(id: string): RulesPack {
  return RULES_PACKS.find((pack) => pack.id === id) ?? RULES_PACKS[0];
}

function label(list: Choice[], id: string, custom = ""): string {
  if (id === "custom") return custom.trim() || "Custom";
  return list.find((choice) => choice.id === id)?.label ?? "";
}

export function genreLabel(draft: CampaignDraft): string {
  return draft.genre ? label(GENRES, draft.genre, draft.genreCustom) : "";
}

export function toneLabel(draft: CampaignDraft): string {
  return TONES.filter((tone) => draft.tones.includes(tone.id))
    .map((tone) => label(TONES, tone.id, draft.toneCustom))
    .join(", ");
}

export function rulesName(draft: CampaignDraft): string {
  return draft.rulesPackId === "custom-homebrew"
    ? draft.customSystemName.trim() || "Custom / Homebrew"
    : rulesPack(draft.rulesPackId).name;
}

/** Error for the essentials every campaign needs (Quick Start, Blank, step 1). */
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
  if (step === 3) {
    for (const category of WORLD_CATEGORIES) {
      const value = draft.categories[category.id];
      if (value.enabled && category.entries && value.entries.some((row) => !row.name.trim())) {
        return `Each ${category.entries.noun.toLowerCase()} in ${category.label} needs a name. Remove unused rows or enter a name.`;
      }
    }
  }
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

function categoryAnswers(spec: CategorySpec, value: CategoryDraft) {
  return spec.questions
    .map((q) => ({ question: q.label, answer: (value.answers[q.key] ?? "").trim() }))
    .filter((a) => a.answer);
}

function entryDescription(spec: CategorySpec, entry: WorldEntryDraft): string {
  return (spec.entries?.fields ?? [])
    .map((field) => ({ field, value: (entry.fields[field.key] ?? "").trim() }))
    .filter(({ value }) => value)
    .map(({ field, value }) => `**${field.label}** ${value}`)
    .join("\n\n");
}

const enabledCategories = (draft: CampaignDraft) => WORLD_CATEGORIES.filter((c) => draft.categories[c.id]?.enabled);

/** The documents Create will write, for the Review step's preview. Folders
 *  are always the full approved tree. */
export function plannedFiles(draft: CampaignDraft): string[] {
  const files = ["project.yaml", "campaign.yaml", "Adventures/Main Campaign/Campaign Overview.md"];
  if (draft.mode === "blank") return files;
  // The offline GM Reference Library is copied into every campaign (020 §9).
  files.push("Game Master/Guides/ (7 starter guides)", "Game Master/Random Tables/ (10 random tables)");
  if (draft.mode === "quick-start") return files;
  // Same-named records in one folder are numbered, as the backend does.
  const used = new Map<string, number>();
  const add = (folder: string, title: string) => {
    const stem = safeName(title);
    const key = `${folder}/${stem}`.toLowerCase();
    const count = (used.get(key) ?? 0) + 1;
    used.set(key, count);
    files.push(`${folder}/${stem}${count === 1 ? "" : ` ${count}`}.md`);
  };
  if (draft.houseRules.trim()) add("Rules & Mechanics/House Rules", "House Rules");
  if (draft.worldMode === "new" && [draft.worldName, draft.worldOrigin, draft.worldScale, draft.era, draft.worldNotes].some((v) => v.trim())) {
    add("Worldbuilding", draft.worldName.trim() || "World Overview");
  }
  for (const category of enabledCategories(draft)) {
    const value = draft.categories[category.id];
    if (categoryAnswers(category, value).length || value.notes.trim()) add(`Worldbuilding/${category.label}`, `${category.label} Overview`);
    for (const entry of value.entries) add(`Worldbuilding/${category.label}`, entry.name);
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
  if (draft.firstSessionTitle.trim() || draft.firstSessionNotes.trim() || draft.sessionZero) {
    add("Sessions/Session Plans", draft.firstSessionTitle.trim() || (draft.sessionZero ? "Session Zero" : "Session 1"));
  }
  if (draft.gmNotes.trim()) add("Game Master/GM Notes", "GM Notes");
  return files;
}

export function buildCampaignInput(draft: CampaignDraft) {
  const base = {
    project_root: draft.projectRoot.trim(),
    name: draft.name.trim(),
    setup_mode: draft.mode,
    basics: { premise: draft.mode === "blank" ? "" : draft.premise.trim() },
    rules: { pack_id: draft.rulesPackId, custom_system_name: draft.customSystemName.trim() },
  };
  if (draft.mode !== "guided") return base;
  const entry = (row: EntryDraft) => ({ name: row.name.trim(), kind: row.kind.trim(), description: row.description.trim(), detail: row.detail.trim() });
  const pack = rulesPack(draft.rulesPackId);
  const editionLabel = pack.editions.find((e) => e.id === draft.edition)?.label ?? draft.edition.trim();
  const newWorld = draft.worldMode === "new";
  const progression = pack.progressions.find((p) => p.id === draft.progressionType);
  const startingProgression = progression && draft.startingLevel.trim() ? `${progression.label}: ${draft.startingLevel.trim()}` : "";
  return {
    ...base,
    basics: {
      premise: draft.premise.trim(),
      genre: genreLabel(draft),
      tone: toneLabel(draft),
      gm_name: draft.gmName.trim(),
      start_date: draft.startDate.trim(),
    },
    rules: {
      pack_id: draft.rulesPackId,
      custom_system_name: draft.customSystemName.trim(),
      edition: editionLabel,
      progression_type: draft.progressionType,
      starting_level: draft.progressionType ? draft.startingLevel.trim() : "",
      advancement: draft.advancement ? label(pack.advancement, draft.advancement, draft.advancementCustom) : "",
      house_rules_mode: draft.houseRulesMode,
      house_rules: draft.houseRulesMode && draft.houseRulesMode !== "standard" ? draft.houseRules.trim() : "",
      allowed_content: lines(draft.allowedContent.replace(/,/g, "\n")),
      creation_notes: draft.creationNotes.trim(),
    },
    world: {
      mode: newWorld ? "new" : "blank",
      name: newWorld ? draft.worldName.trim() : "",
      origin: newWorld ? draft.worldOrigin : "",
      scale: newWorld && draft.worldScale ? label(SETTING_SCALES, draft.worldScale, draft.worldScaleCustom) : "",
      era: newWorld && draft.era ? label(ERAS, draft.era, draft.eraCustom) : "",
      notes: newWorld ? draft.worldNotes.trim() : "",
    },
    worldbuilding: enabledCategories(draft).map((c) => {
      const value = draft.categories[c.id];
      return {
        id: c.id,
        answers: categoryAnswers(c, value),
        entries: value.entries.map((row) => ({ name: row.name.trim(), kind: "", description: entryDescription(c, row) })),
        notes: value.notes.trim(),
      };
    }),
    party: {
      name: draft.partyName.trim(),
      starting_progression: startingProgression,
      expected_players: draft.expectedPlayers.trim(),
      origin: draft.partyOrigin,
      players: draft.players
        .map((p) => ({ player_name: p.player_name.trim(), character_name: p.character_name.trim(), concept: p.concept.trim() }))
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
      structure: draft.storyStructure,
      length: draft.storyLength,
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
      schedule: draft.schedule,
      session_length: draft.sessionLength.trim(),
      session_zero: draft.sessionZero,
    },
  };
}
