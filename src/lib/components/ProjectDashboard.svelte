<script lang="ts">
  import { ArrowRight, BookOpenText, LibraryBig, Target, UsersRound, MapPinned } from "lucide-svelte";
  import { currentProject } from "../stores/project.svelte";
  import { writing } from "../stores/writing.svelte";
  import type { Project } from "../types";

  let {
    project,
    onOpenWrite,
    onOpenStoryDevelopment,
  }: {
    project: Project;
    onOpenWrite: () => void;
    onOpenStoryDevelopment: () => void;
  } = $props();

  $effect(() => {
    const projectId = project.id;
    writing.open(projectId);
    return () => writing.open(null);
  });

  const modifiedDate = $derived(new Date(project.modified_at).toLocaleDateString("en-US", {
    month: "short", day: "numeric", year: "numeric",
  }));
  const chapters = $derived(currentProject.chapters.filter((c) => !c.archived && !c.is_part));
  const parts = $derived(currentProject.chapters.filter((c) => !c.archived && c.is_part));
  const currentChapter = $derived(currentProject.currentChapter ?? chapters[0] ?? null);
  const chapterScenes = $derived(
    currentChapter
      ? currentProject.scenes.filter((scene) => !scene.archived && scene.chapter_id === currentChapter.id)
      : []
  );
  const chapterBrief = $derived.by(() => {
    const direct = currentChapter?.synopsis?.trim();
    if (direct) return direct;

    const sceneBreaks = chapterScenes
      .map((scene) => scene.synopsis?.trim())
      .filter((value): value is string => !!value);

    if (!sceneBreaks.length) return "No chapter outline or scene synopsis has been written yet.";

    const combined = sceneBreaks.join(" ");
    return combined.length > 320 ? combined.slice(0, 317).trimEnd() + "…" : combined;
  });
  const projectWords = $derived(
    writing.value?.project_id === project.id ? writing.value.project_words : null
  );
  const goalPercent = $derived(
    projectWords !== null && project.word_target && project.word_target > 0
      ? Math.min(100, Math.round((projectWords / project.word_target) * 100))
      : null
  );
  const sessionWords = $derived(
    writing.value?.project_id === project.id ? writing.value.session_words : null
  );
  const todayWords = $derived(
    writing.value?.project_id === project.id ? writing.value.today_words : null
  );
  const streak = $derived(
    writing.value?.project_id === project.id ? writing.value.streak : null
  );
</script>

<section class="command-center" aria-labelledby="dashboard-title">
  <header class="hero">
    <div>
      <p class="eyebrow">Project Dashboard</p>
      <h1 id="dashboard-title">{project.name}</h1>
      <p class="meta">{project.genre ?? (project.project_type === "screenplay" ? "Screenplay" : "Novel")} · {project.author_pen_name ?? "Author not set"} · Updated {modifiedDate}</p>
    </div>
    <button class="primary" type="button" onclick={onOpenWrite}>
      Continue Writing <ArrowRight size={17} aria-hidden="true" />
    </button>
  </header>

  <section class="progress-strip" aria-label="Manuscript progress">
    <div class="progress-main">
      <span class="progress-label">Manuscript</span>
      <strong>{projectWords === null ? "—" : projectWords.toLocaleString()} <small>words</small></strong>
      {#if project.word_target}
        <div class="bar" aria-label={goalPercent === null ? "Writing goal" : `${goalPercent}% of writing goal`}>
          <span style:width={`${goalPercent ?? 0}%`}></span>
        </div>
        <p>{goalPercent ?? 0}% of {project.word_target.toLocaleString()} word goal</p>
      {:else}
        <p>No manuscript word target set.</p>
      {/if}
    </div>
    <div class="stat"><strong>{chapters.length}</strong><span>Chapters</span></div>
    <div class="stat"><strong>{parts.length}</strong><span>Parts</span></div>
    <div class="stat"><strong>{todayWords === null ? "—" : todayWords.toLocaleString()}</strong><span>Today</span></div>
    <div class="stat"><strong>{sessionWords === null ? "—" : sessionWords.toLocaleString()}</strong><span>Session</span></div>
  </section>

  <div class="workspace-grid">
    <section class="focus-panel">
      <div class="section-heading">
        <div><p class="eyebrow">Manuscript Map</p><h2>Where you are in the book</h2></div>
        <BookOpenText size={25} aria-hidden="true" />
      </div>
      <div class="manuscript-map">
        <div>
          <span>Chapter</span>
          <strong>{currentChapter?.title ?? "No chapter selected"}</strong>
        </div>
        <div>
          <span>Current scene</span>
          <strong>{currentProject.currentScene?.title ?? "No scene selected"}</strong>
        </div>
        <div>
          <span>Scenes in chapter</span>
          <strong>{chapterScenes.length}</strong>
        </div>
      </div>
      <button class="link-button" type="button" onclick={onOpenWrite}>Open Manuscript <ArrowRight size={15} /></button>
    </section>

    <aside class="today-panel">
      <p class="eyebrow">At a glance</p>
      <h2>Writing Activity</h2>
      <dl>
        <div><dt>Today</dt><dd>{todayWords === null ? "—" : todayWords.toLocaleString()} words</dd></div>
        <div><dt>Session</dt><dd>{sessionWords === null ? "—" : sessionWords.toLocaleString()} words</dd></div>
        <div><dt>Streak</dt><dd>{streak === null ? "—" : `${streak} day${streak === 1 ? "" : "s"}`}</dd></div>
      </dl>
    </aside>

    <section class="development-panel">
      <div class="section-heading">
        <div><p class="eyebrow">Story Development</p><h2>Know the story</h2></div>
        <LibraryBig size={25} aria-hidden="true" />
      </div>
      <div class="knowledge">
        <div><UsersRound size={18} /><strong>{currentProject.characters.length}</strong><span>Characters</span></div>
        <div><MapPinned size={18} /><strong>{currentProject.locations.length}</strong><span>Locations</span></div>
      </div>
      <div class="chapter-context">
        <span>Current chapter</span>
        <strong>{currentChapter?.title ?? "No chapter selected"}</strong>
        <p>{chapterBrief}</p>
      </div>
      <button class="link-button" type="button" onclick={onOpenStoryDevelopment}>Open Story Development <ArrowRight size={15} /></button>
    </section>

    <section class="project-panel">
      <div class="section-heading">
        <div><p class="eyebrow">Project</p><h2>Book brief</h2></div>
        <Target size={25} aria-hidden="true" />
      </div>
      <dl class="book-brief">
        <div><dt>Format</dt><dd>{project.project_type === "screenplay" ? "Screenplay" : "Novel"}</dd></div>
        <div><dt>Genre</dt><dd>{project.genre ?? "Not set"}</dd></div>
        <div><dt>Target</dt><dd>{project.word_target ? `${project.word_target.toLocaleString()} words` : "Not set"}</dd></div>
      </dl>
      {#if project.description}<p class="description">{project.description}</p>{/if}
    </section>
  </div>
</section>

<style>
  .command-center { box-sizing:border-box; width:100%; height:100%; overflow:auto; padding:clamp(30px,4vw,58px); background:var(--color-bg); color:var(--color-text); }
  .hero,.progress-strip,.workspace-grid { width:min(1160px,100%); margin-inline:auto; }
  .hero { display:flex; align-items:flex-end; justify-content:space-between; gap:24px; margin-bottom:26px; }
  .eyebrow { margin:0 0 7px; color:var(--color-accent-text); font:700 .69rem/1.2 var(--font-ui); letter-spacing:.1em; text-transform:uppercase; }
  h1,h2 { font-family:var(--font-display); }
  h1 { margin:0; font-size:clamp(2.2rem,4vw,3.25rem); line-height:1; }
  h2 { margin:0; font-size:1.18rem; }
  .meta { margin:10px 0 0; color:var(--color-text-muted); font-size:.82rem; }
  .primary { display:inline-flex; align-items:center; gap:9px; padding:12px 17px; border:1px solid var(--color-accent); border-radius:var(--radius-m); background:color-mix(in srgb,var(--color-accent) 16%,var(--color-surface)); color:var(--color-text); font:650 .82rem/1 var(--font-ui); cursor:pointer; }
  .progress-strip { display:grid; grid-template-columns:minmax(300px,2.2fr) repeat(4,minmax(90px,.65fr)); align-items:stretch; margin-bottom:18px; border:1px solid var(--color-border); border-radius:var(--radius-l); background:var(--color-surface); overflow:hidden; }
  .progress-main { padding:20px 22px; }
  .progress-label { display:block; margin-bottom:5px; color:var(--color-text-muted); font-size:.7rem; text-transform:uppercase; letter-spacing:.08em; }
  .progress-main strong { font-family:var(--font-display); font-size:1.75rem; }
  .progress-main strong small { font:500 .72rem/1 var(--font-ui); color:var(--color-text-muted); }
  .progress-main p { margin:7px 0 0; color:var(--color-text-muted); font-size:.72rem; }
  .bar { height:5px; margin-top:11px; border-radius:99px; background:var(--color-bg); overflow:hidden; }
  .bar span { display:block; height:100%; background:var(--color-accent-text); }
  .stat { display:grid; place-content:center; text-align:center; border-left:1px solid var(--color-border); padding:16px 10px; }
  .stat strong { font-family:var(--font-display); font-size:1.45rem; }
  .stat span { margin-top:4px; color:var(--color-text-muted); font-size:.67rem; text-transform:uppercase; letter-spacing:.06em; }
  .workspace-grid { display:grid; grid-template-columns:minmax(0,1.6fr) minmax(260px,.75fr); gap:18px; }
  .focus-panel,.today-panel,.development-panel,.project-panel { box-sizing:border-box; padding:20px 22px; border:1px solid var(--color-border); border-radius:var(--radius-l); background:var(--color-surface); }
  .focus-panel { min-height:205px; }
  .section-heading { display:flex; align-items:flex-start; justify-content:space-between; gap:18px; margin-bottom:18px; }
  .section-heading :global(svg) { color:var(--color-accent-text); }
  .manuscript-map { display:grid; grid-template-columns:1.1fr 1.1fr .55fr; gap:14px; margin:2px 0 22px; }
  .manuscript-map div { min-width:0; padding:14px 15px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:color-mix(in srgb,var(--color-bg) 62%,var(--color-surface)); }
  .manuscript-map span,.chapter-context > span { display:block; margin-bottom:5px; color:var(--color-text-muted); font-size:.66rem; text-transform:uppercase; letter-spacing:.07em; }
  .manuscript-map strong { display:block; font-family:var(--font-display); font-size:1rem; overflow-wrap:anywhere; }
  .description { margin:9px 0 19px; color:var(--color-text-muted); font-size:.78rem; line-height:1.5; }
  .link-button { display:inline-flex; align-items:center; gap:7px; padding:0; border:0; background:transparent; color:var(--color-accent-text); font:650 .77rem/1.2 var(--font-ui); cursor:pointer; }
  .today-panel dl,.book-brief { margin:20px 0 0; }
  .today-panel dl div,.book-brief div { display:flex; justify-content:space-between; gap:18px; padding:10px 0; border-top:1px solid var(--color-border); }
  dt { color:var(--color-text-muted); font-size:.75rem; }
  dd { margin:0; font-size:.78rem; font-weight:650; text-align:right; }
  .knowledge { display:flex; gap:42px; margin:5px 0 16px; }
  .chapter-context { margin:0 0 18px; padding-top:14px; border-top:1px solid var(--color-border); }
  .chapter-context strong { display:block; margin-bottom:6px; font-family:var(--font-display); font-size:.95rem; }
  .chapter-context p { margin:0; color:var(--color-text-muted); font-size:.76rem; line-height:1.5; }
  .knowledge div { display:grid; grid-template-columns:auto auto; align-items:center; gap:3px 8px; }
  .knowledge :global(svg) { grid-row:1 / span 2; color:var(--color-accent-text); }
  .knowledge strong { font-family:var(--font-display); font-size:1.45rem; }
  .knowledge span { color:var(--color-text-muted); font-size:.7rem; }
  button:focus-visible { outline:var(--focus-ring); outline-offset:3px; }
  @media(max-width:900px){ .progress-strip{grid-template-columns:repeat(4,1fr)} .progress-main{grid-column:1/-1} .stat:first-of-type{border-left:0} .workspace-grid{grid-template-columns:1fr} }
  @media(max-width:650px){ .command-center{padding:24px}.hero{align-items:flex-start;flex-direction:column}.progress-strip{grid-template-columns:repeat(2,1fr)}.progress-main{grid-column:1/-1}.knowledge{gap:24px}.manuscript-map{grid-template-columns:1fr} }
</style>
