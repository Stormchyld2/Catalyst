<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ArrowRight, BookOpenText, CalendarDays, Dices, LibraryBig, MapPinned, ScrollText, UsersRound } from "lucide-svelte";
  import type { Project } from "../types";

  type CampaignSummary = {
    ruleset: string;
    edition: string | null;
    starting_level: string | null;
    party_name: string | null;
    expected_players: string | null;
    play_mode: string;
    session_schedule: string | null;
    sessions_planned: number;
    sessions_logged: number;
    adventure_records: number;
    encounters: number;
    npcs: number;
    world_entries: number;
    latest_session_plan: string | null;
  };
  let { project, onOpenWrite, onOpenStoryDevelopment, onOpenAdventures, onOpenCampaignOverview }: {
    project: Project;
    onOpenWrite: () => void;
    onOpenStoryDevelopment: () => void;
    onOpenAdventures: () => void;
    onOpenCampaignOverview: () => void;
  } = $props();

  let summary = $state<CampaignSummary | null>(null);
  let loading = $state(false);
  let error = $state<string | null>(null);
  const modifiedDate = $derived(new Date(project.modified_at).toLocaleDateString("en-US", {
    month: "short", day: "numeric", year: "numeric",
  }));

  $effect(() => {
    const root = project.project_root;
    let cancelled = false;
    summary = null;
    error = null;
    if (!root || project.project_type !== "campaign") {
      error = "This project is not a file-backed RPG campaign. Open or create a campaign to use the campaign dashboard.";
      return;
    }
    loading = true;
    invoke<CampaignSummary>("get_campaign_dashboard_summary", { projectRoot: root })
      .then((data) => { if (!cancelled) summary = data; })
      .catch((e) => { if (!cancelled) error = String(e); })
      .finally(() => { if (!cancelled) loading = false; });
    return () => { cancelled = true; };
  });
</script>

<section class="command-center" aria-labelledby="dashboard-title">
  <header class="hero">
    <div>
      <p class="eyebrow">Campaign Dashboard</p>
      <h1 id="dashboard-title">{project.name}</h1>
      <p class="meta">{project.genre ?? "Tabletop RPG campaign"} · Updated {modifiedDate}</p>
    </div>
    <div class="hero-actions">
      <button class="primary" type="button" onclick={onOpenCampaignOverview}>
        Edit Campaign Overview <ArrowRight size={17} aria-hidden="true" />
      </button>
      <button class="link-button" type="button" onclick={onOpenWrite}>
        Open Campaign Notes <ArrowRight size={17} aria-hidden="true" />
      </button>
    </div>
  </header>

  {#if loading}
    <p role="status" class="notice">Reading campaign files…</p>
  {:else if error}
    <p role="alert" class="notice">{error}</p>
  {:else if summary}
    <section class="dashboard-card" aria-label="Campaign details">
      <div class="section-heading"><ScrollText size={21} /><h2>At the table</h2></div>
      <div class="details">
        <div><span>Ruleset</span><strong>{summary.ruleset}{summary.edition ? ` · ${summary.edition}` : ""}</strong></div>
        <div><span>Starting level</span><strong>{summary.starting_level || "Not specified"}</strong></div>
        <div><span>Party</span><strong>{summary.party_name || "Not named"}{summary.expected_players ? ` · ${summary.expected_players} expected` : ""}</strong></div>
        <div><span>Play mode</span><strong>{summary.play_mode || "Not specified"}</strong></div>
        <div><span>Schedule</span><strong>{summary.session_schedule || "Not scheduled"}</strong></div>
      </div>
    </section>

    <div class="metrics-grid" aria-label="Campaign records">
      <div class="stat"><CalendarDays size={19} /><strong>{summary.sessions_planned}</strong><span>Session Plans</span></div>
      <div class="stat"><BookOpenText size={19} /><strong>{summary.sessions_logged}</strong><span>Session Logs</span></div>
      <div class="stat"><ScrollText size={19} /><strong>{summary.adventure_records}</strong><span>Adventure Documents</span></div>
      <div class="stat"><Dices size={19} /><strong>{summary.encounters}</strong><span>Encounters</span></div>
      <div class="stat"><UsersRound size={19} /><strong>{summary.npcs}</strong><span>NPC Records</span></div>
      <div class="stat"><MapPinned size={19} /><strong>{summary.world_entries}</strong><span>Worldbuilding Records</span></div>
    </div>

    <div class="actions-grid">
      <section class="dashboard-card">
        <div class="section-heading"><CalendarDays size={21} /><h2>Session preparation</h2></div>
        <p>{summary.latest_session_plan ? `Latest saved plan: ${summary.latest_session_plan}` : "No session plan has been created yet."}</p>
        <button class="link-button" type="button" onclick={onOpenWrite}>Open session notes <ArrowRight size={15} /></button>
      </section>
      <section class="dashboard-card">
        <div class="section-heading"><LibraryBig size={21} /><h2>World Builder</h2></div>
        <p>Explore and edit campaign locations, factions, NPCs and lore.</p>
        <button class="link-button" type="button" onclick={onOpenStoryDevelopment}>Open World Builder <ArrowRight size={15} /></button>
      </section>
      <section class="dashboard-card">
        <div class="section-heading"><ScrollText size={21} /><h2>Adventures</h2></div>
        <p>Prepare quests, adventure arcs, scenes and encounters.</p>
        <button class="link-button" type="button" onclick={onOpenAdventures}>Open Adventures <ArrowRight size={15} /></button>
      </section>
    </div>
  {/if}
</section>

<style>
  .command-center{box-sizing:border-box;width:100%;height:100%;overflow:auto;padding:clamp(24px,4vw,58px);background:var(--color-bg);color:var(--color-text)}
  .hero,.dashboard-card,.metrics-grid,.actions-grid{width:min(1160px,100%);margin-inline:auto}
  .hero{display:flex;align-items:flex-end;justify-content:space-between;gap:24px;margin-bottom:24px}
  .hero-actions{display:flex;gap:10px;flex-wrap:wrap;justify-content:flex-end}
  .eyebrow{margin:0 0 8px;color:var(--color-accent-text);font:700 .72rem var(--font-ui);text-transform:uppercase;letter-spacing:.09em}
  h1,h2{font-family:var(--font-display);margin:0}
  h1{font-size:clamp(2rem,4vw,3.1rem)}
  h2{font-size:1.18rem}
  .meta,.notice,.dashboard-card p{color:var(--color-text-muted);font-size:.9rem;line-height:1.6}
  .primary,.link-button{display:inline-flex;align-items:center;gap:9px;padding:11px 15px;border:1px solid var(--color-border);border-radius:var(--radius-m);background:var(--color-surface);color:var(--color-text);font-weight:650;cursor:pointer}
  .primary{border-color:var(--color-accent);background:color-mix(in srgb,var(--color-accent) 18%,var(--color-surface))}
  .dashboard-card{padding:21px;border:1px solid var(--color-border);border-radius:var(--radius-l);background:var(--color-surface)}
  .section-heading{display:flex;gap:10px;align-items:center;margin-bottom:14px}
  .details{display:grid;grid-template-columns:repeat(auto-fit,minmax(170px,1fr));gap:15px}
  .details div{display:flex;flex-direction:column;gap:7px}
  .details span,.stat span{color:var(--color-text-muted);font-size:.75rem}
  .details strong{font-size:.88rem;overflow-wrap:anywhere}
  .metrics-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:14px;margin-top:17px}
  .stat{display:flex;flex-direction:column;gap:8px;padding:19px;border:1px solid var(--color-border);background:var(--color-surface);border-radius:var(--radius-l)}
  .stat strong{font-size:1.6rem;font-family:var(--font-display)}
  .actions-grid{display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px;margin-top:17px}
  .actions-grid .dashboard-card{display:flex;flex-direction:column;align-items:flex-start}
  .actions-grid p{flex:1}
  .link-button{font-size:.82rem;background:transparent}
  .notice{max-width:1160px;margin:20px auto}
  @media(max-width:800px){.hero{align-items:flex-start;flex-direction:column}.metrics-grid,.actions-grid{grid-template-columns:repeat(2,minmax(0,1fr))}}
  @media(max-width:520px){.metrics-grid,.actions-grid{grid-template-columns:1fr}}
</style>
