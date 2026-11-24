<script lang="ts">
  import type { Editor } from "@tiptap/core";
  import { invoke } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import { currentProject } from "../stores/project.svelte";
  import { writing } from "../stores/writing.svelte";
  import { countWordsInHtml } from "../utils/wordCount";
  import NovelEditor from "./NovelEditor.svelte";

  let editor = $state<Editor | null>(null);
  let draft = $state("");
  let saveStatus = $state<"idle" | "saving" | "error">("idle");
  let saveTimer: ReturnType<typeof setTimeout> | null = null;
  let loadedChapterId: string | null = null;

  const project = $derived(currentProject.value);
  const chapter = $derived(currentProject.currentChapter);
  const chapterWords = $derived(countWordsInHtml(draft));

  function chapterContent() {
    const prose = [...currentProject.scenes]
      .sort((a, b) => a.position - b.position)
      .map((scene) => scene.prose?.trim() ?? "")
      .filter(Boolean);
    return prose.join("<hr>");
  }

  // Recovery drafts are supplementary; the Markdown manuscript remains authoritative.
  function recoveryKey(projectId: string, chapterId: string) {
    return `catalyst:unsaved-chapter:${projectId}:${chapterId}`;
  }

  function recover(projectId: string, chapterId: string): string | null {
    try {
      return window.localStorage.getItem(recoveryKey(projectId, chapterId))
        ?? window.localStorage.getItem(`lithomark:unsaved-chapter:${projectId}:${chapterId}`);
    } catch {
      return null;
    }
  }

  function remember(projectId: string, chapterId: string, html: string) {
    try {
      window.localStorage.setItem(recoveryKey(projectId, chapterId), html);
    } catch (error) {
      console.error("Could not store an emergency chapter recovery draft:", error);
    }
  }

  function forget(projectId: string, chapterId: string, savedHtml: string) {
    try {
      for (const key of [recoveryKey(projectId, chapterId), `lithomark:unsaved-chapter:${projectId}:${chapterId}`]) {
        if (window.localStorage.getItem(key) === savedHtml) window.localStorage.removeItem(key);
      }
    } catch (error) {
      console.error("Could not clear chapter recovery draft:", error);
    }
  }

  let revision = 0;
  let persistedRevision = 0;
  let pendingSave: Promise<void> = Promise.resolve();

  function loadChapter() {
    const chapterId = chapter?.id ?? null;
    const projectId = project?.id;
    if (!chapterId || !projectId || chapterId === loadedChapterId) return;
    // Preserve any unsaved content under the chapter it belongs to.
    loadedChapterId = chapterId;
    const stored = recover(projectId, chapterId);
    draft = stored ?? chapterContent();
    revision = stored === null ? 0 : 1;
    persistedRevision = 0;
    saveStatus = stored === null ? "idle" : "error";
  }

  $effect(() => {
    void chapter?.id;
    void currentProject.scenes;
    loadChapter();
  });

  function handleUpdate(html: string) {
    const projectId = project?.id;
    const chapterId = loadedChapterId;
    draft = html;
    revision++;
    if (projectId && chapterId) remember(projectId, chapterId, html);
    saveStatus = "saving";
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => void save(), 900);
  }

  export async function save() {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = null;
    const activeProject = currentProject.value;
    const activeChapter = currentProject.currentChapter;
    if (!activeProject?.project_root || !activeChapter || activeChapter.id !== loadedChapterId) return pendingSave;
    const value = draft;
    const savedRevision = revision;
    const projectId = activeProject.id;
    const chapterId = activeChapter.id;
    const projectRoot = activeProject.project_root;
    if (savedRevision <= persistedRevision) return pendingSave;
    saveStatus = "saving";
    // Serialize native writes so an older save cannot overwrite a newer edit.
    pendingSave = pendingSave.catch(() => {}).then(async () => {
      try {
        const projection = await invoke<import("../types").WriteProjection>("save_file_chapter_prose", {
          projectRoot,
          projectId,
          chapterId,
          proseHtml: value,
        });
        persistedRevision = Math.max(persistedRevision, savedRevision);
        forget(projectId, chapterId, value);
        if (currentProject.currentChapter?.id === chapterId) {
          const refreshed = projection.scenes_by_chapter[chapterId] ?? [];
          currentProject.setScenes(refreshed);
          if (revision === savedRevision) saveStatus = "idle";
        }
        writing.scheduleRefresh(projectId);
      } catch (error) {
        console.error("Failed to save chapter prose:", error);
        if (currentProject.currentChapter?.id === chapterId) saveStatus = "error";
      }
    });
    return pendingSave;
  }

  export async function flush() {
    await save();
    await pendingSave;
    if (revision > persistedRevision && currentProject.currentChapter?.id === loadedChapterId) {
      await save();
    }
  }

  function insertOutlineText(event: Event) {
    const detail = (event as CustomEvent<{ sceneId: string; text: string }>).detail;
    if (!detail?.text || !editor) return;
    const text = detail.text.trim();
    if (!text) return;
    editor.chain().focus().insertContent(`<p>${escapeHtml(text)}</p>`).run();
  }

  function escapeHtml(value: string) {
    return value
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;")
      .replaceAll("'", "&#39;");
  }

  onMount(() => {
    window.addEventListener("catalyst:insert-outline-text", insertOutlineText);
    return () => window.removeEventListener("catalyst:insert-outline-text", insertOutlineText);
  });

  onDestroy(() => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
      void save();
    }
  });
</script>

<section class="chapter-write" aria-labelledby="chapter-write-title">
  <header class="chapter-write-head">
    <div>
      <p class="chapter-kicker">Chapter {chapter ? chapter.position + 1 : ""}</p>
      <h1 id="chapter-write-title">{chapter?.title}</h1>
    </div>
    <span class="chapter-count">{chapterWords.toLocaleString()} words</span>
  </header>

  <div class="chapter-editor">
    <NovelEditor
      projectId={project?.id}
      sceneId={chapter?.id}
      content={draft}
      readonly={!!chapter?.locked}
      {saveStatus}
      placeholder="Write this chapter…"
      onEditorReady={(instance) => (editor = instance)}
      onUpdate={handleUpdate}
    />
  </div>
</section>

<style>
  .chapter-write { display:flex; flex-direction:column; min-width:0; min-height:0; height:100%; background:var(--color-bg); }
  .chapter-write-head { display:flex; align-items:end; justify-content:space-between; gap:var(--space-m); padding:var(--space-xs) var(--space-m); border-bottom:var(--border-hair); }
  .chapter-kicker { margin:0 0 2px; color:var(--color-text-muted); font:600 var(--text-small)/1.3 var(--font-ui); text-transform:uppercase; letter-spacing:.08em; }
  .chapter-write-head h1 { margin:0; color:var(--color-text); font:550 var(--text-h2)/1.2 var(--font-display); }
  .chapter-count { color:var(--color-text-muted); font:var(--text-small)/1.4 var(--font-ui); }
  .chapter-editor { flex:1; min-width:0; min-height:0; overflow:hidden; }
</style>
