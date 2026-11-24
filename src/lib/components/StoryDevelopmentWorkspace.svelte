<script module lang="ts">
  interface StoryDevelopmentSessionState {
    selectedPath: string | null;
    openTabPaths: string[];
    closedTabPaths: string[];
    expandedPaths: string[];
    viewMode: "read" | "edit" | "split";
    searchQuery: string;
    contentsOpen: boolean;
  }

  // Disposable UI session state only. Authoritative project content remains on disk.
  // Keeping this outside the workspace component lets Write/Story Dev switches
  // unmount the view without losing the user's working context.
  const storyDevelopmentSessions = new Map<string, StoryDevelopmentSessionState>();
</script>

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    AlertTriangle,
    Bold,
    ChevronDown,
    ChevronRight,
    FileText,
    FilePlus,
    FolderPlus,
    LayoutTemplate,
    Folder,
    Image as ImageIcon,
    Heading1,
    Heading2,
    Heading3,
    Italic,
    ListTree,
    Loader2,
    RefreshCw,
    RotateCcw,
    Save,
    Trash2,
    ArchiveRestore,
    Strikethrough,
  } from "lucide-svelte";
  import { onDestroy, onMount, tick, untrack } from "svelte";
  import { currentProject } from "../stores/project.svelte";
  import { markdownHeadingId, renderMarkdown } from "../utils/markdownPreview";
  import EntityTemplateManager from "./EntityTemplateManager.svelte";

  // Reuse the file-authoritative wiki for Adventures and Campaign Notes,
  // selecting only their actual campaign folders rather than novel outlines.
  let { focusRoot = null, initialDocumentPath = null }: { focusRoot?: "Adventures" | "Sessions" | null; initialDocumentPath?: string | null } = $props();
  const workspaceLabel = $derived(focusRoot === "Adventures" ? "Adventures" : focusRoot === "Sessions" ? "Campaign Notes" : "World Builder");
  const withinFocus = (relativePath: string) => !focusRoot || relativePath === focusRoot || relativePath.startsWith(focusRoot + "/");
  const sessionKey = (id: string) => id + ":" + (focusRoot ?? "world");

  interface StoryDevelopmentEntry {
    id: string;
    entry_kind?: "document" | "image";
    document_type: string;
    title: string;
    relative_path: string;
    section: string;
  }

  interface StoryDevelopmentIssue {
    relative_path: string;
    message: string;
  }

  interface StoryDevelopmentIndex {
    entries: StoryDevelopmentEntry[];
    issues: StoryDevelopmentIssue[];
  }

  interface StoryDevelopmentMetadataField {
    key: string;
    value: unknown;
  }

  interface StoryDevelopmentAsset {
    relative_path: string;
    mime_type: string;
    bytes: number[];
  }

  interface EntityTemplateSummary {
    id: string;
    name: string;
    category: string;
    scope: string;
    document_type: string;
    subtype: string | null;
    overview_fields: string[];
    image_field: string;
    source: string;
    fields: { key: string; label: string; type: string; required: boolean; options: string[] }[];
  }

  interface EntityTemplateRegistry {
    templates: EntityTemplateSummary[];
    issues: { relative_path: string; message: string }[];
    user_templates_dir: string;
  }

  interface StoryDevelopmentDocument {
    id: string;
    document_type: string;
    title: string | null;
    relative_path: string;
    body: string;
    source: string;
    metadata_yaml: string;
    metadata: StoryDevelopmentMetadataField[];
    source_hash: string;
  }

  type ViewMode = "read" | "edit" | "split";

  interface LinkTarget {
    id: string;
    document_type: string;
    title: string;
    aliases: string[];
    relative_path: string;
  }

  type LinkResolution =
    | { status: "resolved"; target: LinkTarget }
    | { status: "ambiguous"; query: string; candidates: LinkTarget[] }
    | { status: "unresolved"; query: string };

  interface DocumentHeading {
    level: number;
    text: string;
    id: string;
  }

  interface TreeFile {
    kind: "file";
    entry: StoryDevelopmentEntry;
  }

  interface TreeFolder {
    kind: "folder";
    name: string;
    path: string;
    children: TreeNode[];
  }

  type TreeNode = TreeFile | TreeFolder;

  let index = $state<StoryDevelopmentIndex>({ entries: [], issues: [] });
  let document = $state<StoryDevelopmentDocument | null>(null);
  let loadingIndex = $state(false);
  let loadingDocument = $state(false);
  let error = $state<string | null>(null);
  let loadedProjectId = $state<string | null>(null);
  let selectedPath = $state<string | null>(null);
  let openTabs = $state<StoryDevelopmentEntry[]>([]);
  let closedTabs = $state<StoryDevelopmentEntry[]>([]);
  let imageDataUrl = $state<string | null>(null);
  let overviewImageDataUrl = $state<string | null>(null);
  let overviewImageError = $state<string | null>(null);
  let viewMode = $state<ViewMode>("read");
  let contentsOpen = $state(false);
  let searchQuery = $state("");
  let markdownEditor = $state<HTMLTextAreaElement | null>(null);
  let expanded = $state<Set<string>>(new Set());
  let bodyDraft = $state("");
  let savingBody = $state(false);
  let bodyError = $state<string | null>(null);
  let entityTemplates = $state<EntityTemplateRegistry>({ templates: [], issues: [], user_templates_dir: "" });
  let templateManagerOpen = $state(false);
  let templateActionsOpen = $state(false);
  let assignTemplateOpen = $state(false);
  let assignTemplateId = $state("");
  let assignTemplatePreview = $state<{template_name: string; proposed_body: string; added_headings: string[]} | null>(null);
  let assignTemplateError = $state<string | null>(null);
  let assigningTemplate = $state(false);
  let linkMessage = $state<string | null>(null);
  let ambiguousLink = $state<{ query: string; candidates: LinkTarget[] } | null>(null);

  const bodyDirty = $derived(document ? bodyDraft !== document.source : false);
  const selectedEntry = $derived(
    selectedPath ? index.entries.find((entry) => entry.relative_path === selectedPath) ?? null : null
  );
  const selectedIsImage = $derived(selectedEntry?.entry_kind === "image");
  function sourceBody(source: string): string {
    if (!source.startsWith("---")) return source;
    const lines = source.replace(/\r\n/g, "\n").replace(/\r/g, "\n").split("\n");
    if (lines[0]?.trim() !== "---") return source;
    const closing = lines.findIndex((line, index) => index > 0 && line.trim() === "---");
    return closing < 0 ? "" : lines.slice(closing + 1).join("\n").replace(/^\n/, "");
  }

  const renderedBody = $derived(
    document ? renderMarkdown(sourceBody(bodyDraft)) : ""
  );
  const normalizedSearch = $derived(searchQuery.trim().toLowerCase());
  const visibleEntries = $derived(
    index.entries.filter((entry) => withinFocus(entry.relative_path) &&
      (!normalizedSearch || `${entry.title} ${entry.relative_path} ${entry.document_type}`
        .toLowerCase().includes(normalizedSearch)))
  );
  let storyFolders = $state<string[]>(["Story Development"]);
  const tree = $derived(buildTree(visibleEntries, normalizedSearch ? [] : storyFolders.filter(withinFocus)));
  const contentsHeadings = $derived(
    document ? extractHeadings(bodyDraft) : []
  );
  const storyTemplateGroups = $derived.by(() => {
    const labels: Record<string, string> = { character: "Characters", location: "Locations", worldbuilding: "Worldbuilding", organization: "Organizations", object: "Objects", item: "Items & Equipment", campaign: "Campaign", adventure: "Adventures", creature: "Creatures", session: "Sessions", gm: "Game Master", event: "Events", research: "Research", general: "General" };
    const order = ["campaign", "adventure", "session", "gm", "character", "creature", "location", "worldbuilding", "organization", "item", "object", "event", "research", "general"];
    // Adventures and Campaign Notes offer the templates that belong there; World
    // Builder offers the whole catalog. Custom categories always appear.
    const focusCategories: Record<string, string[]> = {
      Adventures: ["adventure", "campaign", "gm", "creature", "location", "event"],
      Sessions: ["session", "campaign", "gm"],
    };
    const known = new Set(order);
    const allowed = focusRoot ? focusCategories[focusRoot] : null;
    const grouped = new Map<string, EntityTemplateSummary[]>();
    for (const template of entityTemplates.templates.filter((item) => (!item.scope || ["story-development", "worldbuilding", "campaign", "adventure", "encounter", "session", "gm"].includes(item.scope))
      && (!allowed || allowed.includes(item.category || "general") || !known.has(item.category || "general")))) {
      const key = template.category || "general";
      if (!grouped.has(key)) grouped.set(key, []);
      grouped.get(key)!.push(template);
    }
    return [...grouped.entries()]
      .sort(([a], [b]) => {
        const ai = order.indexOf(a), bi = order.indexOf(b);
        return (ai < 0 ? order.length : ai) - (bi < 0 ? order.length : bi) || a.localeCompare(b);
      })
      .map(([category, templates]) => ({ category, label: labels[category] ?? category, templates: templates.sort((a, b) => a.name.localeCompare(b.name)) }));
  });

  const activeEntityTemplate = $derived(resolveEntityTemplate(document, entityTemplates.templates));
  const activeImageField = $derived(activeEntityTemplate?.image_field || "image");
  const overviewMetadata = $derived.by(() => {
    const currentDocument = document;
    if (!currentDocument) return [];

    if (activeEntityTemplate?.overview_fields?.length) {
      return activeEntityTemplate.overview_fields
        .map((key) => currentDocument.metadata.find((field) => field.key === key))
        .filter((field): field is StoryDevelopmentMetadataField => Boolean(field));
    }

    return currentDocument.metadata.filter((field) => field.key !== activeImageField);
  });

  function metadataString(doc: StoryDevelopmentDocument, key: string): string | null {
    const value = doc.metadata.find((field) => field.key === key)?.value;
    return typeof value === "string" && value.trim() ? value.trim() : null;
  }

  function metadataValue(doc: StoryDevelopmentDocument, key: string): unknown {
    return doc.metadata.find((field) => field.key === key)?.value;
  }

  function metadataList(doc: StoryDevelopmentDocument, key: string): string[] {
    const value = metadataValue(doc, key);
    if (Array.isArray(value)) return value.map(String).map((item) => item.trim()).filter(Boolean);
    if (typeof value === "string" && value.trim()) {
      return value.split(",").map((item) => item.trim()).filter(Boolean);
    }
    return [];
  }

  const characterPanel = $derived.by(() => {
    const doc = document;
    if (!doc || activeEntityTemplate?.id !== "character-profile") return null;
    return {
      subtitle: metadataString(doc, "subtitle"),
      role: metadataString(doc, "role"),
      pov: metadataString(doc, "pov") ?? metadataString(doc, "point_of_view"),
      status: metadataString(doc, "status"),
      age: metadataString(doc, "age"),
      occupation: metadataString(doc, "occupation"),
      affiliation: metadataString(doc, "affiliation"),
      location: metadataString(doc, "location") ?? metadataString(doc, "current_location"),
      narrativeRole: metadataString(doc, "narrative_role"),
      aliases: metadataList(doc, "aliases"),
      relationships: metadataList(doc, "relationships"),
      tags: metadataList(doc, "tags"),
      firstAppearance: metadataString(doc, "first_appearance"),
      lastAppearance: metadataString(doc, "last_appearance"),
      birthYear: metadataString(doc, "birth_year"),
    };
  });

  function resolveEntityTemplate(
    doc: StoryDevelopmentDocument | null,
    templates: EntityTemplateSummary[]
  ): EntityTemplateSummary | null {
    if (!doc) return null;

    const explicitId = metadataString(doc, "template_id");
    if (explicitId) {
      const explicit = templates.find((template) => template.id === explicitId);
      if (explicit) return explicit;
    }

    const subtype = metadataString(doc, "subtype");
    const compatible = templates.filter((template) => template.document_type === doc.document_type);
    if (subtype) {
      const subtypeMatch = compatible.find((template) => template.subtype === subtype);
      if (subtypeMatch) return subtypeMatch;
    }

    return compatible.find((template) => template.subtype === null && template.source === "built-in")
      ?? compatible.find((template) => template.subtype === null)
      ?? compatible[0]
      ?? null;
  }

  let newDocumentOpen = $state(false);
  let newDocumentFolder = $state(untrack(() => focusRoot) ?? "Story Development");
  let newDocumentMode = $state<"template" | "empty">("template");
  let folderDialogOpen = $state(false);
  let folderName = $state("");
  let folderParent = $state(untrack(() => focusRoot) ?? "Story Development");
  let folderError = $state<string | null>(null);
  let creatingFolder = $state(false);
  let selectedFolder = $state<string | null>(null);
  let newDocumentTemplate = $state("");
  let newDocumentTitle = $state("");
  let newDocumentFields = $state<Record<string, string>>({});
  let newDocumentError = $state<string | null>(null);
  let creatingDocument = $state(false);
  let relocateOpen = $state(false);
  let relocateDestination = $state("");
  let relocateError = $state<string | null>(null);
  let relocating = $state(false);
  interface TrashedStoryDocument {
    entry_id: string;
    original_path: string;
    document_id: string;
    source_hash: string;
  }
  let trashOpen = $state(false);
  let trashLoading = $state(false);
  let trashRestoring = $state(false);
  let trashItems = $state<TrashedStoryDocument[]>([]);
  let trashError = $state<string | null>(null);
  let deleteOpen = $state(false);
  let deleteError = $state<string | null>(null);
  let deleting = $state(false);
  const selectedCreationTemplate = $derived(entityTemplates.templates.find(t => t.id === newDocumentTemplate));

  async function startNewDocument() {
    try { await refreshStoryFolders(); } catch (cause) { newDocumentError = String(cause); return; }
    newDocumentFolder = selectedFolder && storyFolders.includes(selectedFolder) ? selectedFolder : storyFolders.find(withinFocus) ?? focusRoot ?? "Story Development";
    newDocumentMode = "template";
    const preferredTemplate = focusRoot === "Adventures" ? "quest-adventure"
      : focusRoot === "Sessions" ? "session-plan" : "character-profile";
    newDocumentTemplate = entityTemplates.templates.find(t => t.id === preferredTemplate)?.id
      ?? entityTemplates.templates[0]?.id ?? "";
    newDocumentTitle = "";
    newDocumentFields = {};
    newDocumentError = null;
    newDocumentOpen = true;
  }

  async function submitNewDocument() {
    const root = currentProject.value?.project_root;
    const template = selectedCreationTemplate;
    if (!root || (newDocumentMode === "template" && !template) || !newDocumentTitle.trim() || creatingDocument) return;
    if (!(await saveBeforeNavigation())) {
      newDocumentError = "Save the current document before creating another.";
      return;
    }
    creatingDocument = true;
    newDocumentError = null;
    try {
      const fields: Record<string, unknown> = {};
      for (const field of newDocumentMode === "template" ? (template?.fields ?? []) : []) {
        const raw = newDocumentFields[field.key]?.trim();
        if (!raw) {
          if (field.required) throw new Error(field.label + " is required.");
          continue;
        }
        fields[field.key] = field.type === "list"
          ? raw.split(",").map(x => x.trim()).filter(Boolean)
          : field.type === "number" ? Number(raw)
          : field.type === "boolean" ? raw === "true" : raw;
        if (field.type === "number" && !Number.isFinite(fields[field.key] as number)) {
          throw new Error(field.label + " must be a number.");
        }
      }
      const created = await invoke<StoryDevelopmentDocument>("create_story_document_from_template", {
        input: { project_root: root, template_id: newDocumentMode === "empty" ? "empty" : template?.id, title: newDocumentTitle.trim(), destination_folder: newDocumentFolder, fields },
      });
      if (currentProject.value?.project_root !== root) return;
      newDocumentOpen = false;
      expanded = new Set([...expanded, ...newDocumentFolder.split("/").map((_, i, parts) => parts.slice(0, i + 1).join("/"))]);
      await loadIndex(currentProject.value.id);
      const entry = index.entries.find(item => item.relative_path === created.relative_path);
      if (entry) await activateTab(entry);
    } catch (cause) {
      newDocumentError = String(cause);
    } finally {
      creatingDocument = false;
    }
  }

  function openRelocateDocument() {
    if (!document || selectedIsImage || savingBody) return;
    relocateDestination = document.relative_path;
    relocateError = null;
    relocateOpen = true;
  }

  async function submitRelocateDocument() {
    const projectId = currentProject.value?.id;
    const projectRoot = currentProject.value?.project_root;
    if (!projectId || !projectRoot || !document || relocating) return;
    if (!(await saveBeforeNavigation())) {
      relocateError = "The document could not be saved. Resolve the save error before moving it.";
      return;
    }
    // A pending edit may have been saved above; use the refreshed hash.
    const active = document;
    if (!active) return;
    const destinationPath = relocateDestination.trim().replace(/\\/g, "/");
    if (!destinationPath || destinationPath === active.relative_path) {
      relocateError = "Enter a different project-relative Markdown path.";
      return;
    }
    // Book roots plus the campaign roots (see campaign_wizard.rs CAMPAIGN_ROOTS).
    const storyRoots = ["Story Development", "Characters", "Locations", "Worldbuilding",
      "Campaign", "World", "Adventures", "Sessions", "Game Master", "Maps",
      "Creatures & Monsters", "Encounters", "Items & Equipment", "Rules & Mechanics"];
    if (!storyRoots.some((root) => destinationPath.startsWith(`${root}/`))) {
      relocateError = "Choose a path inside one of the campaign folders.";
      return;
    }
    relocating = true;
    relocateError = null;
    try {
      const moved = await invoke<StoryDevelopmentDocument>("move_story_development_document", {
        projectRoot, relativePath: active.relative_path,
        destinationPath, expectedSourceHash: active.source_hash,
      });
      if (currentProject.value?.id !== projectId || selectedPath !== active.relative_path) return;
      const refreshed = await invoke<StoryDevelopmentIndex>("get_story_development_index", { projectRoot });
      if (currentProject.value?.id !== projectId || selectedPath !== active.relative_path) return;
      const entry = refreshed.entries.find((item) => item.id === active.id && item.relative_path === moved.relative_path);
      if (!entry) throw new Error("Document moved, but its stable ID was not found in the refreshed index. Refresh the Wiki.");
      index = refreshed;
      openTabs = openTabs.map((tab) => tab.id === active.id ? entry : tab);
      closedTabs = closedTabs.map((tab) => tab.id === active.id ? entry : tab);
      selectedPath = moved.relative_path;
      document = moved;
      bodyDraft = moved.source;
      expanded = new Set([...expanded, ...moved.relative_path.split("/").slice(0, -1).map((_, i, parts) => parts.slice(0, i + 1).join("/"))]);
      relocateOpen = false;
    } catch (cause) {
      relocateError = String(cause);
    } finally {
      relocating = false;
    }
  }

  async function openTrash() {
    const root = currentProject.value?.project_root;
    if (!root) return;
    trashOpen = true;
    trashLoading = true;
    trashError = null;
    trashItems = [];
    try {
      const items = await invoke<TrashedStoryDocument[]>("list_story_development_trash", { projectRoot: root });
      if (currentProject.value?.project_root === root) trashItems = items;
    } catch (cause) {
      if (currentProject.value?.project_root === root) trashError = String(cause);
    } finally {
      trashLoading = false;
    }
  }

  async function restoreTrashItem(item: TrashedStoryDocument) {
    const projectId = currentProject.value?.id;
    const root = currentProject.value?.project_root;
    if (!projectId || !root || trashRestoring) return;
    if (!(await saveBeforeNavigation())) {
      trashError = "Save the current document before restoring another.";
      return;
    }
    trashRestoring = true;
    trashError = null;
    try {
      const restored = await invoke<StoryDevelopmentDocument>("restore_story_development_trash", {
        projectRoot: root, entryId: item.entry_id, expectedSourceHash: item.source_hash,
      });
      if (currentProject.value?.id !== projectId) return;
      if (restored.id !== item.document_id || restored.relative_path !== item.original_path) {
        throw new Error("Restoration completed, but returned unexpected identity or path. Refresh Story Development.");
      }
      const refreshed = await invoke<StoryDevelopmentIndex>("get_story_development_index", { projectRoot: root });
      if (currentProject.value?.id !== projectId) return;
      index = refreshed;
      const entry = refreshed.entries.find((candidate) => candidate.id === restored.id && candidate.relative_path === restored.relative_path);
      if (!entry) throw new Error("Restored document is not in the Wiki index. Refresh Story Development.");
      trashItems = trashItems.filter((candidate) => candidate.entry_id !== item.entry_id);
      expanded = new Set([...expanded, ...restored.relative_path.split("/").slice(0, -1).map((_, i, parts) => parts.slice(0, i + 1).join("/"))]);
      trashOpen = false;
      openEntry(entry);
    } catch (cause) {
      trashError = String(cause);
    } finally {
      trashRestoring = false;
    }
  }

  function openDeleteDocument() {
    if (!document || savingBody || relocating) return;
    deleteError = null;
    deleteOpen = true;
  }

  async function confirmDeleteDocument() {
    const projectId = currentProject.value?.id;
    const projectRoot = currentProject.value?.project_root;
    const active = document;
    if (!projectId || !projectRoot || !active || deleting) return;
    if (savingBody || bodyDirty) {
      deleteError = "Save your changes before moving this document to trash.";
      return;
    }
    deleting = true;
    deleteError = null;
    try {
      const deletedId = await invoke<string>("delete_story_development_document", {
        projectRoot, relativePath: active.relative_path, expectedSourceHash: active.source_hash,
      });
      if (deletedId !== active.id) throw new Error("Deletion returned an unexpected document ID. Refresh the Wiki.");
      if (currentProject.value?.id !== projectId) return;
      // The file is already in recoverable trash. Clear its tab even if index
      // refresh fails, so a stale editor cannot accidentally recreate it.
      openTabs = openTabs.filter((tab) => tab.id !== active.id);
      closedTabs = closedTabs.filter((tab) => tab.id !== active.id);
      index = { ...index, entries: index.entries.filter((entry) => entry.id !== active.id) };
      if (selectedPath === active.relative_path) {
        selectedPath = null;
        document = null;
        bodyDraft = "";
        bodyError = null;
        overviewImageDataUrl = null;
        overviewImageError = null;
        const next = openTabs[0] ?? index.entries[0];
        if (next) openEntry(next);
      }
      deleteOpen = false;
      try {
        const refreshed = await invoke<StoryDevelopmentIndex>("get_story_development_index", { projectRoot });
        if (currentProject.value?.id === projectId) index = refreshed;
      } catch (cause) {
        if (currentProject.value?.id === projectId) {
          error = "Document moved to recoverable trash, but the file tree could not refresh: " + String(cause);
        }
      }
    } catch (cause) {
      deleteError = String(cause);
    } finally {
      deleting = false;
    }
  }

  function openTemplateAssignment() {
    if (!document) return;
    assignTemplateId = metadataString(document, "template_id")
      ?? activeEntityTemplate?.id ?? entityTemplates.templates[0]?.id ?? "";
    assignTemplatePreview = null;
    assignTemplateError = null;
    assignTemplateOpen = true;
  }

  async function previewTemplateAssignment() {
    const root = currentProject.value?.project_root;
    if (!root || !document || !assignTemplateId) return;
    if (bodyDirty || savingBody) {
      assignTemplateError = "Save your Markdown edits before previewing a template.";
      return;
    }
    assignTemplateError = null;
    assignTemplatePreview = null;
    try {
      assignTemplatePreview = await invoke("preview_story_document_template", {
        projectRoot: root, relativePath: document.relative_path, templateId: assignTemplateId,
      });
    } catch (cause) { assignTemplateError = String(cause); }
  }

  async function confirmTemplateAssignment() {
    const root = currentProject.value?.project_root;
    if (!root || !document || !assignTemplatePreview || assigningTemplate) return;
    assigningTemplate = true;
    assignTemplateError = null;
    try {
      const originalPath = document.relative_path;
      const updated = await invoke<StoryDevelopmentDocument>("apply_story_document_template", {
        projectRoot: root, relativePath: originalPath, templateId: assignTemplateId,
        expectedSourceHash: document.source_hash,
      });
      if (document?.relative_path === originalPath) {
        document = updated;
        bodyDraft = updated.source;
        assignTemplateOpen = false;
      }
    } catch (cause) {
      assignTemplateError = String(cause);
      assignTemplatePreview = null;
    } finally { assigningTemplate = false; }
  }

  async function refreshStoryFolders() {
    const root = currentProject.value?.project_root;
    if (!root) return;
    const folders = await invoke<string[]>("get_story_development_folders", { projectRoot: root });
    if (!Array.isArray(folders) || !folders.every((folder) => typeof folder === "string")) {
      throw new Error("Story Development folder discovery returned an invalid response.");
    }
    storyFolders = folders;
  }

  async function startNewFolder() {
    folderError = null;
    folderName = "";
    try { await refreshStoryFolders(); }
    catch (cause) { folderError = String(cause); return; }
    folderParent = selectedFolder && storyFolders.includes(selectedFolder)
      ? selectedFolder : storyFolders[0] ?? "Story Development";
    folderDialogOpen = true;
  }

  async function submitNewFolder() {
    const root = currentProject.value?.project_root;
    if (!root || !folderName.trim() || creatingFolder) return;
    creatingFolder = true;
    folderError = null;
    try {
      const created = await invoke<string>("create_story_development_folder", {
        projectRoot: root, parentFolder: folderParent, name: folderName.trim(),
      });
      if (currentProject.value?.project_root !== root) return;
      selectedFolder = created;
      expanded = new Set([...expanded, folderParent, created]);
      await refreshStoryFolders();
      folderDialogOpen = false;
    } catch (cause) { folderError = String(cause); }
    finally { creatingFolder = false; }
  }

  async function loadEntityTemplates() {
    try {
      entityTemplates = await invoke<EntityTemplateRegistry>("get_entity_template_registry");
    } catch {
      entityTemplates = { templates: [], issues: [], user_templates_dir: "" };
    }
  }

  function extractHeadings(markdown: string): DocumentHeading[] {
    return markdown
      .replace(/\r\n?/g, "\n")
      .split("\n")
      .map((line) => /^(#{1,6})\s+(.+?)\s*$/.exec(line))
      .filter((match): match is RegExpExecArray => Boolean(match))
      .map((match) => {
        const text = match[2].replace(/[*_~`]/g, "");
        return { level: match[1].length, text, id: markdownHeadingId(text) };
      });
  }

  function folderIsExpanded(path: string): boolean {
    return Boolean(normalizedSearch) || expanded.has(path);
  }

  function jumpToHeading(id: string) {
    window.document.getElementById(id)?.scrollIntoView({ behavior: "smooth", block: "start" });
  }

  async function wrapSelection(prefix: string, suffix = prefix) {
    const editor = markdownEditor;
    if (!editor) return;

    const start = editor.selectionStart;
    const end = editor.selectionEnd;
    const selection = bodyDraft.slice(start, end);
    bodyDraft = bodyDraft.slice(0, start) + prefix + selection + suffix + bodyDraft.slice(end);

    await tick();
    editor.focus();
    editor.setSelectionRange(start + prefix.length, end + prefix.length);
  }

  async function applyHeading(level: 1 | 2 | 3) {
    const editor = markdownEditor;
    if (!editor) return;

    const start = editor.selectionStart;
    const end = editor.selectionEnd;
    const lineStart = bodyDraft.lastIndexOf("\n", Math.max(0, start - 1)) + 1;
    const nextBreak = bodyDraft.indexOf("\n", end);
    const lineEnd = nextBreak === -1 ? bodyDraft.length : nextBreak;
    const selectedLines = bodyDraft.slice(lineStart, lineEnd);
    const prefix = `${"#".repeat(level)} `;
    const replacement = selectedLines
      .split("\n")
      .map((line) => `${prefix}${line.replace(/^#{1,6}\s+/, "")}`)
      .join("\n");

    bodyDraft = bodyDraft.slice(0, lineStart) + replacement + bodyDraft.slice(lineEnd);

    await tick();
    editor.focus();
    editor.setSelectionRange(lineStart, lineStart + replacement.length);
  }

  function entryKind(entry: StoryDevelopmentEntry): "document" | "image" {
    return entry.entry_kind === "image" ? "image" : "document";
  }

  function ensureTab(entry: StoryDevelopmentEntry) {
    if (!openTabs.some((tab) => tab.relative_path === entry.relative_path)) {
      openTabs = [...openTabs, entry];
    }
  }

  function bytesToDataUrl(asset: StoryDevelopmentAsset): string {
    let binary = "";
    const chunkSize = 0x8000;
    for (let offset = 0; offset < asset.bytes.length; offset += chunkSize) {
      binary += String.fromCharCode(...asset.bytes.slice(offset, offset + chunkSize));
    }
    return `data:${asset.mime_type};base64,${btoa(binary)}`;
  }

  function buildTree(entries: StoryDevelopmentEntry[], folders: string[]): TreeNode[] {
    type MutableFolder = { name: string; path: string; folders: Map<string, MutableFolder>; files: TreeFile[] };
    const root: MutableFolder = { name: "", path: "", folders: new Map(), files: [] };

    for (const folderPath of folders) {
      let folder = root;
      for (const segment of folderPath.split("/")) {
        const path = folder.path ? folder.path + "/" + segment : segment;
        let next = folder.folders.get(segment);
        if (!next) {
          next = { name: segment, path, folders: new Map(), files: [] };
          folder.folders.set(segment, next);
        }
        folder = next;
      }
    }

    for (const entry of entries) {
      const parts = entry.relative_path.split("/");
      let folder = root;
      for (const segment of parts.slice(0, -1)) {
        const path = folder.path ? `${folder.path}/${segment}` : segment;
        let next = folder.folders.get(segment);
        if (!next) {
          next = { name: segment, path, folders: new Map(), files: [] };
          folder.folders.set(segment, next);
        }
        folder = next;
      }
      folder.files.push({ kind: "file", entry });
    }

    const finalize = (folder: MutableFolder): TreeNode[] => [
      ...[...folder.folders.values()]
        .sort((a, b) => a.name.localeCompare(b.name))
        .map<TreeFolder>((child) => ({
          kind: "folder",
          name: child.name,
          path: child.path,
          children: finalize(child),
        })),
      ...folder.files.sort((a, b) => a.entry.title.localeCompare(b.entry.title)),
    ];

    return finalize(root);
  }

  function displayMetadataValue(value: unknown): string {
    if (value === null || value === undefined || value === "") return "—";
    if (Array.isArray(value)) return value.length ? value.map(displayMetadataValue).join(", ") : "—";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  }

  function metadataLabel(key: string): string {
    return key
      .replace(/[_-]+/g, " ")
      .replace(/\b\w/g, (letter) => letter.toUpperCase());
  }

  async function saveBody(): Promise<boolean> {
    if (savingBody) return false;
    if (!bodyDirty) return true;
    if (!document || !currentProject.value?.id) return false;

    const projectRoot = currentProject.value.project_root;
    if (!projectRoot) {
      bodyError = "This project does not have a local Catalyst project folder yet.";
      return false;
    }

    savingBody = true;
    bodyError = null;
    const projectId = currentProject.value.id;
    const relativePath = document.relative_path;
    const expectedSourceHash = document.source_hash;
    const savingDraft = bodyDraft;

    try {
      const updated = await invoke<StoryDevelopmentDocument>("update_story_development_source", {
        projectRoot,
        relativePath,
        source: savingDraft,
        expectedSourceHash,
      });

      if (currentProject.value?.id !== projectId || selectedPath !== relativePath) return false;
      document = updated;
      if (bodyDraft === savingDraft) bodyDraft = updated.source;

      // The image reference lives in YAML. A full-source edit can therefore
      // change the portrait without reopening the document. Refresh it from
      // the newly parsed/saved metadata after every successful source save.
      const template = resolveEntityTemplate(updated, entityTemplates.templates);
      const imageKey = template?.image_field || "image";
      const imageField = updated.metadata.find((field) => field.key === imageKey);
      overviewImageDataUrl = null;
      overviewImageError = null;
      if (typeof imageField?.value === "string" && imageField.value.trim()) {
        void loadOverviewImage(projectId, imageField.value.trim(), relativePath);
      }
      return true;
    } catch (e) {
      bodyError = e instanceof Error ? e.message : String(e);
      return false;
    } finally {
      if (currentProject.value?.id === projectId && selectedPath === relativePath) {
        savingBody = false;
      }
    }
  }

  // Autosave after a short idle period. The live preview always reads the in-memory draft.
  // Timer cleanup prevents stale edits from being saved after switching documents.
  $effect(() => {
    const dirty = bodyDirty;
    const activePath = selectedPath;
    const activeProject = currentProject.value?.id;
    const draft = bodyDraft;
    if (!dirty || savingBody || !activePath || !activeProject || !draft && !document) return;
    const timer = window.setTimeout(() => {
      if (selectedPath === activePath && currentProject.value?.id === activeProject && bodyDirty && !savingBody) {
        void saveBody();
      }
    }, 3000);
    return () => window.clearTimeout(timer);
  });

  // Never discard a dirty document while navigating. A failed or in-flight save
  // keeps the current document visible so the author can retry.
  async function saveBeforeNavigation(): Promise<boolean> {
    if (savingBody) return false;
    if (!bodyDirty) return true;
    const saved = await saveBody();
    return saved && !bodyDirty;
  }

  function toggleFolder(path: string) {
    const next = new Set(expanded);
    if (next.has(path)) next.delete(path);
    else next.add(path);
    expanded = next;
  }

  async function loadIndex(projectId: string) {
    loadingIndex = true;
    error = null;
    const requestProjectId = projectId;

    try {
      const projectRoot = currentProject.value?.project_root;
      if (!projectRoot) {
        throw new Error("This project does not have a local Catalyst project folder yet.");
      }

      const result = await invoke<StoryDevelopmentIndex>("get_story_development_index", {
        projectRoot,
      });
      if (currentProject.value?.id !== requestProjectId) return;

      // Folder discovery is supplementary to the authoritative document index.
      // A folder read failure must not hide existing Wiki documents.
      index = result;
      try {
        await refreshStoryFolders();
      } catch {
        storyFolders = ["Story Development"];
      }
      if (currentProject.value?.id !== requestProjectId) return;
      loadedProjectId = requestProjectId;

      const saved = storyDevelopmentSessions.get(sessionKey(requestProjectId));
      if (saved) {
        const byPath = new Map(result.entries.map((entry) => [entry.relative_path, entry]));
        openTabs = saved.openTabPaths.filter(withinFocus)
          .map((path) => byPath.get(path))
          .filter((entry): entry is StoryDevelopmentEntry => Boolean(entry));
        closedTabs = saved.closedTabPaths.filter(withinFocus)
          .map((path) => byPath.get(path))
          .filter((entry): entry is StoryDevelopmentEntry => Boolean(entry));
        expanded = new Set(saved.expandedPaths);
        viewMode = saved.viewMode;
        searchQuery = saved.searchQuery;
        contentsOpen = saved.contentsOpen;
        selectedPath =
          saved.selectedPath && byPath.has(saved.selectedPath) && withinFocus(saved.selectedPath) ? saved.selectedPath : null;
      }

      const requested = initialDocumentPath && withinFocus(initialDocumentPath) &&
        result.entries.some((entry) => entry.relative_path === initialDocumentPath)
          ? initialDocumentPath : null;
      const first = requested ??
        (selectedPath && result.entries.some((entry) => entry.relative_path === selectedPath && withinFocus(entry.relative_path))
          ? selectedPath
          : result.entries.find((entry) => withinFocus(entry.relative_path))?.relative_path ?? null);

      selectedPath = first;
      if (!saved) {
        expanded = new Set(result.entries.map((entry) => entry.relative_path.split("/")[0]));
      }
      if (first) {
        const firstEntry = result.entries.find((entry) => entry.relative_path === first);
        if (firstEntry) {
          ensureTab(firstEntry);
          if (entryKind(firstEntry) === "image") await loadAsset(requestProjectId, first);
          else await loadDocument(requestProjectId, first);
        }
      } else {
        document = null;
        imageDataUrl = null;
      }
    } catch (e) {
      if (currentProject.value?.id !== requestProjectId) return;
      error = e instanceof Error ? e.message : String(e);
      index = { entries: [], issues: [] };
      document = null;
      selectedPath = null;
    } finally {
      if (currentProject.value?.id === requestProjectId) loadingIndex = false;
    }
  }

  async function loadDocument(projectId: string, relativePath: string) {
    loadingDocument = true;
    error = null;

    try {
      const projectRoot = currentProject.value?.project_root;
      if (!projectRoot) {
        throw new Error("This project does not have a local Catalyst project folder yet.");
      }

      const result = await invoke<StoryDevelopmentDocument>("get_story_development_document", {
        projectRoot,
        relativePath,
      });
      if (currentProject.value?.id !== projectId || selectedPath !== relativePath) return;
      document = result;
      imageDataUrl = null;
      overviewImageDataUrl = null;
      overviewImageError = null;
      bodyDraft = result.source;
      bodyError = null;
      const template = resolveEntityTemplate(result, entityTemplates.templates);
      const imageKey = template?.image_field || "image";
      const imageField = result.metadata.find((field) => field.key === imageKey);
      if (typeof imageField?.value === "string" && imageField.value.trim()) {
        void loadOverviewImage(projectId, imageField.value.trim(), relativePath);
      }
    } catch (e) {
      if (currentProject.value?.id !== projectId || selectedPath !== relativePath) return;
      error = e instanceof Error ? e.message : String(e);
      document = null;
    } finally {
      if (currentProject.value?.id === projectId && selectedPath === relativePath) {
        loadingDocument = false;
      }
    }
  }

  async function loadAsset(projectId: string, relativePath: string) {
    loadingDocument = true;
    error = null;

    try {
      const projectRoot = currentProject.value?.project_root;
      if (!projectRoot) {
        throw new Error("This project does not have a local Catalyst project folder yet.");
      }
      const asset = await invoke<StoryDevelopmentAsset>("get_story_development_asset", {
        projectRoot,
        relativePath,
      });
      if (currentProject.value?.id !== projectId || selectedPath !== relativePath) return;

      document = null;
      bodyDraft = "";
      overviewImageDataUrl = null;
      overviewImageError = null;
      imageDataUrl = bytesToDataUrl(asset);
    } catch (e) {
      if (currentProject.value?.id !== projectId || selectedPath !== relativePath) return;
      error = e instanceof Error ? e.message : String(e);
      imageDataUrl = null;
    } finally {
      if (currentProject.value?.id === projectId && selectedPath === relativePath) {
        loadingDocument = false;
      }
    }
  }

  async function loadOverviewImage(projectId: string, imagePath: string, documentPath: string) {
    try {
      const projectRoot = currentProject.value?.project_root;
      if (!projectRoot) return;
      const asset = await invoke<StoryDevelopmentAsset>("get_story_development_asset", {
        projectRoot,
        relativePath: imagePath,
      });
      if (
        currentProject.value?.id !== projectId ||
        selectedPath !== documentPath ||
        document?.relative_path !== documentPath
      ) {
        return;
      }
      overviewImageDataUrl = bytesToDataUrl(asset);
      overviewImageError = null;
    } catch (e) {
      overviewImageDataUrl = null;
      overviewImageError = e instanceof Error ? e.message : String(e);
    }
  }

  function openEntry(entry: StoryDevelopmentEntry) {
    const projectId = currentProject.value?.id;
    if (!projectId) return;

    ensureTab(entry);
    selectedPath = entry.relative_path;
    error = null;
    linkMessage = null;
    ambiguousLink = null;

    if (entryKind(entry) === "image") {
      document = null;
      bodyDraft = "";
      void loadAsset(projectId, entry.relative_path);
    } else {
      imageDataUrl = null;
      void loadDocument(projectId, entry.relative_path);
    }
  }

  function openResolvedTarget(target: LinkTarget): boolean {
    const entry = index.entries.find((candidate) => candidate.id === target.id);
    if (!entry) {
      linkMessage = `Linked document is outside Story Development: ${target.title}`;
      return false;
    }

    bodyDraft = "";
    bodyError = null;
    openEntry(entry);
    linkMessage = null;
    return true;
  }

  async function openWikilink(query: string) {
    const projectId = currentProject.value?.id;
    const projectRoot = currentProject.value?.project_root;
    if (!projectId || !projectRoot || !query.trim()) return;
    if (!(await saveBeforeNavigation())) return;

    linkMessage = null;
    ambiguousLink = null;
    try {
      const resolution = await invoke<LinkResolution>("resolve_project_link", {
        projectRoot,
        query,
      });
      if (currentProject.value?.id !== projectId) return;

      if (resolution.status === "unresolved") {
        linkMessage = `Unresolved link: ${resolution.query}`;
        return;
      }
      if (resolution.status === "ambiguous") {
        ambiguousLink = { query: resolution.query, candidates: resolution.candidates };
        return;
      }

      openResolvedTarget(resolution.target);
    } catch (e) {
      linkMessage = e instanceof Error ? e.message : String(e);
    }
  }

  async function selectAmbiguousLink(candidate: LinkTarget) {
    const projectId = currentProject.value?.id;
    const projectRoot = currentProject.value?.project_root;
    if (!projectId || !projectRoot) return;
    if (!(await saveBeforeNavigation())) return;

    linkMessage = null;
    try {
      const resolution = await invoke<LinkResolution>("resolve_project_link", {
        projectRoot,
        query: candidate.id,
      });
      if (currentProject.value?.id !== projectId) return;

      if (resolution.status !== "resolved" || resolution.target.id !== candidate.id) {
        linkMessage = `Selected link could not be resolved by stable ID: ${candidate.title}`;
        return;
      }

      if (openResolvedTarget(resolution.target)) ambiguousLink = null;
    } catch (e) {
      linkMessage = e instanceof Error ? e.message : String(e);
    }
  }

  function dismissAmbiguousLink() {
    ambiguousLink = null;
  }

  function handlePreviewClick(event: MouseEvent) {
    const element = event.target instanceof Element
      ? event.target.closest<HTMLElement>("[data-wikilink]")
      : null;
    const query = element?.dataset.wikilink;
    if (!query) return;
    event.preventDefault();
    void openWikilink(query);
  }


  function handlePreviewKeydown(event: KeyboardEvent) {
    if (event.key !== "Enter" && event.key !== " ") return;
    const target = event.target as HTMLElement | null;
    if (!target?.closest("button.wikilink")) return;
    event.preventDefault();
    handlePreviewClick(event as unknown as MouseEvent);
  }
  async function activateTab(entry: StoryDevelopmentEntry) {
    if (entry.relative_path === selectedPath) return;
    if (!(await saveBeforeNavigation())) return;
    bodyDraft = "";
    bodyError = null;
    openEntry(entry);
  }

  async function closeTab(event: MouseEvent, entry: StoryDevelopmentEntry) {
    event.stopPropagation();
    const indexToClose = openTabs.findIndex((tab) => tab.relative_path === entry.relative_path);
    if (indexToClose < 0) return;
    const active = entry.relative_path === selectedPath;
    if (active && !(await saveBeforeNavigation())) return;

    const remaining = openTabs.filter((tab) => tab.relative_path !== entry.relative_path);
    openTabs = remaining;
    closedTabs = [entry, ...closedTabs.filter((tab) => tab.relative_path !== entry.relative_path)].slice(0, 10);

    if (!active) return;
    const next = remaining[indexToClose] ?? remaining[indexToClose - 1] ?? null;
    if (next) {
      bodyDraft = "";
      bodyError = null;
      openEntry(next);
    } else {
      selectedPath = null;
      document = null;
      imageDataUrl = null;
      overviewImageDataUrl = null;
      overviewImageError = null;
      bodyDraft = "";
    }
  }

  async function reopenClosedTab() {
    const entry = closedTabs[0];
    if (!entry) return;
    if (!(await saveBeforeNavigation())) return;

    closedTabs = closedTabs.slice(1);
    bodyDraft = "";
    bodyError = null;
    openEntry(entry);
  }

  async function selectEntry(entry: StoryDevelopmentEntry) {
    if (entry.relative_path === selectedPath) {
      ensureTab(entry);
      return;
    }
    if (!(await saveBeforeNavigation())) return;
    bodyDraft = "";
    bodyError = null;
    openEntry(entry);
  }

  async function refresh() {
    const projectId = currentProject.value?.id;
    if (!projectId || !(await saveBeforeNavigation())) return;
    void loadIndex(projectId);
  }

  function saveSessionState(projectId: string | null) {
    if (!projectId) return;
    storyDevelopmentSessions.set(sessionKey(projectId), {
      selectedPath,
      openTabPaths: openTabs.map((entry) => entry.relative_path),
      closedTabPaths: closedTabs.map((entry) => entry.relative_path),
      expandedPaths: [...expanded],
      viewMode,
      searchQuery,
      contentsOpen,
    });
  }

  onDestroy(() => {
    saveSessionState(loadedProjectId ?? currentProject.value?.id ?? null);
  });

  onMount(() => {
    void loadEntityTemplates();

    const handleBeforeUnload = (event: BeforeUnloadEvent) => {
      if (!bodyDirty) return;
      event.preventDefault();
      event.returnValue = "";
    };

    const handleWorkspaceChange = (event: Event) => {
      const request = event as CustomEvent<{ pendingDocumentSaves?: Promise<boolean>[] }>;
      if (request.detail?.pendingDocumentSaves) {
        request.detail.pendingDocumentSaves.push(saveBeforeNavigation());
      } else if (bodyDirty || savingBody) {
        // Legacy synchronous callers must not discard unsaved content.
        event.preventDefault();
      }
    };

    const handleSaveShortcut = (event: KeyboardEvent) => {
      if (
        !bodyDirty ||
        savingBody ||
        event.defaultPrevented ||
        event.isComposing ||
        !(event.ctrlKey || event.metaKey) ||
        event.altKey ||
        event.key.toLowerCase() !== "s"
      ) {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      void saveBody();
    };

    const flushBeforeClose = (event: Event) => {
      const request = event as CustomEvent<{ pendingDocumentSaves: Promise<boolean>[] }>;
      request.detail?.pendingDocumentSaves.push(saveBeforeNavigation());
    };

    const handleRecoveryRestored = (event: Event) => {
      const detail = (event as CustomEvent<{ projectRoot: string; relativePath: string }>).detail;
      if (detail?.projectRoot !== currentProject.value?.project_root) return;
      if (selectedPath === detail.relativePath && currentProject.value?.id) {
        document = null;
        bodyDraft = "";
        void loadDocument(currentProject.value.id, detail.relativePath);
      }
    };
    window.addEventListener("catalyst:recovery-restored", handleRecoveryRestored);
    window.addEventListener("catalyst:flush-documents-before-close", flushBeforeClose);
    window.addEventListener("beforeunload", handleBeforeUnload);
    window.addEventListener("catalyst:before-workspace-change", handleWorkspaceChange);
    window.addEventListener("catalyst:before-project-change", handleWorkspaceChange);
    window.addEventListener("keydown", handleSaveShortcut, true);

    return () => {
      window.removeEventListener("catalyst:recovery-restored", handleRecoveryRestored);
      window.removeEventListener("catalyst:flush-documents-before-close", flushBeforeClose);
      window.removeEventListener("beforeunload", handleBeforeUnload);
      window.removeEventListener("catalyst:before-workspace-change", handleWorkspaceChange);
      window.removeEventListener("catalyst:before-project-change", handleWorkspaceChange);
      window.removeEventListener("keydown", handleSaveShortcut, true);
    };
  });

  $effect(() => {
    const projectId = currentProject.value?.id ?? null;

    if (!projectId) {
      loadedProjectId = null;
      index = { entries: [], issues: [] };
      document = null;
      openTabs = [];
      closedTabs = [];
      imageDataUrl = null;
      overviewImageDataUrl = null;
      overviewImageError = null;
      bodyDraft = "";
      bodyError = null;
      selectedPath = null;
      searchQuery = "";
      contentsOpen = false;
      error = null;
      linkMessage = null;
      ambiguousLink = null;
      return;
    }

    if (projectId !== loadedProjectId && !loadingIndex) {
      linkMessage = null;
      ambiguousLink = null;
      void loadIndex(projectId);
    }
  });
</script>

{#snippet treeNodes(nodes: TreeNode[], depth = 0)}
  <ul class="tree-list" style:--tree-depth={depth}>
    {#each nodes as node (node.kind === "folder" ? node.path : node.entry.id)}
      <li>
        {#if node.kind === "folder"}
          <button
            type="button"
            class="tree-row tree-folder"
            aria-expanded={folderIsExpanded(node.path)}
            aria-label={`Folder ${node.path}`}
            onclick={() => { selectedFolder = node.path; toggleFolder(node.path); }}
          >
            {#if folderIsExpanded(node.path)}
              <ChevronDown size={14} aria-hidden="true" />
            {:else}
              <ChevronRight size={14} aria-hidden="true" />
            {/if}
            <Folder size={15} aria-hidden="true" />
            <span>{node.name}</span>
          </button>
          {#if folderIsExpanded(node.path)}
            {@render treeNodes(node.children, depth + 1)}
          {/if}
        {:else}
          <button
            type="button"
            class="tree-row tree-file"
            class:tree-file--active={selectedPath === node.entry.relative_path}
            aria-current={selectedPath === node.entry.relative_path ? "page" : undefined}
            aria-label={entryKind(node.entry) === "image"
              ? `Project image ${node.entry.title}`
              : `Project file ${node.entry.title}`}
            onclick={() => selectEntry(node.entry)}
          >
            <span class="tree-spacer"></span>
            {#if entryKind(node.entry) === "image"}
              <ImageIcon size={14} aria-hidden="true" />
            {:else}
              <FileText size={14} aria-hidden="true" />
            {/if}
            <span>{node.entry.title}</span>
          </button>
        {/if}
      </li>
    {/each}
  </ul>
{/snippet}

<section class="story-development" aria-label="Story Development workspace">
  <aside class="story-development__nav">
    <header class="story-development__nav-header">
      <div>
        <p class="eyebrow">{workspaceLabel}</p>
        <h2>{currentProject.value?.name ?? "Project"}</h2>
      </div>
      <button
        type="button"
        class="ka-icon-button"
        aria-label="Refresh Story Development"
        title="Refresh"
        disabled={loadingIndex}
        onclick={refresh}
      >
        <span class:spin={loadingIndex}><RefreshCw size={18} aria-hidden="true" /></span>
      </button>
    </header>

    <div class="story-development__search">
      <input
        type="search"
        placeholder="Search files…"
        aria-label="Search Story Development files"
        bind:value={searchQuery}
      />
    </div>

    <div class="story-development__tree" aria-label="Project files">
      {#if loadingIndex && index.entries.length === 0}
        <div class="status"><span class="spin"><Loader2 size={18} aria-hidden="true" /></span> Loading files…</div>
      {:else if tree.length === 0}
        <div class="empty">{normalizedSearch ? "No matching files." : `No ${workspaceLabel} files yet.`}</div>
      {:else}
        {@render treeNodes(tree)}
      {/if}
    </div>

    {#if templateActionsOpen}
      <div class="story-development__template-actions" aria-label="Template actions">
        <button type="button" disabled={!document || selectedIsImage} onclick={() => { templateActionsOpen = false; openTemplateAssignment(); }}>Assign / Change Template</button>
        <button type="button" onclick={() => { templateActionsOpen = false; templateManagerOpen = true; }}>Manage Templates</button>
      </div>
    {/if}
    <div class="story-development__create-footer">
      <span title={selectedFolder ?? focusRoot ?? workspaceLabel}>{selectedFolder ?? focusRoot ?? workspaceLabel}</span>
      <button type="button" title="New file" aria-label="New file" onclick={startNewDocument}><FilePlus size={18}/></button>
      <button type="button" title="New folder" aria-label="New folder" onclick={startNewFolder}><FolderPlus size={18}/></button>
      <button type="button" title="Restore deleted documents" aria-label="Restore deleted documents" onclick={openTrash}><ArchiveRestore size={18}/></button>
      <button type="button" title="Templates" aria-label="Templates" aria-expanded={templateActionsOpen} onclick={() => (templateActionsOpen = !templateActionsOpen)}><LayoutTemplate size={18}/></button>
    </div>

    {#if index.issues.length}
      <section class="issues">
        <h3><AlertTriangle size={14} aria-hidden="true" /> File Issues</h3>
        {#each index.issues as issue (issue.relative_path)}
          <div class="issue"><strong>{issue.relative_path}</strong><span>{issue.message}</span></div>
        {/each}
      </section>
    {/if}
  </aside>

  <div class="story-development__workspace">
    <div class="document-tabs" role="tablist" aria-label="Open Story Development files">
      {#each openTabs as tab (tab.relative_path)}
        <div
          role="tab"
          class="document-tab"
          class:document-tab--active={selectedPath === tab.relative_path}
          aria-selected={selectedPath === tab.relative_path}
        >
          <button
            type="button"
            class="document-tab__activate"
            aria-label={tab.title}
            onclick={() => activateTab(tab)}
          >
            {#if entryKind(tab) === "image"}
              <ImageIcon size={13} aria-hidden="true" />
            {:else}
              <FileText size={13} aria-hidden="true" />
            {/if}
            <span>{tab.title}</span>
            {#if selectedPath === tab.relative_path && bodyDirty}
              <span class="document-tab__dirty" aria-label="Unsaved changes">●</span>
            {/if}
          </button>
          <button
            type="button"
            class="document-tab__close"
            aria-label={`Close ${tab.title}`}
            onclick={(event) => closeTab(event, tab)}
          >×</button>
        </div>
      {/each}
      {#if closedTabs.length}
        <button
          type="button"
          class="document-tabs__reopen"
          aria-label="Reopen closed tab"
          title={`Reopen ${closedTabs[0].title}`}
          onclick={reopenClosedTab}
        >
          <RotateCcw size={13} aria-hidden="true" />
        </button>
      {/if}
    </div>

    <header class="document-toolbar">
      <div class="document-title">
        <span>{document?.title ?? selectedEntry?.title ?? selectedPath ?? workspaceLabel}</span>
        {#if loadingDocument}<span class="spin"><Loader2 size={14} aria-hidden="true" /></span>{/if}
      </div>
      {#if !selectedIsImage}
        <div class="document-toolbar__controls">
          {#if document}
            <button type="button" aria-label="Rename or move document" disabled={savingBody || relocating || deleting} onclick={openRelocateDocument}>Rename / Move</button>
            <button type="button" aria-label="Delete document" disabled={savingBody || relocating || deleting} onclick={openDeleteDocument}><Trash2 size={14} aria-hidden="true" /> Delete</button>
          {/if}
          <button
            type="button"
            class="contents-toggle"
            class:active={contentsOpen}
            aria-pressed={contentsOpen}
            onclick={() => (contentsOpen = !contentsOpen)}
          >
            <ListTree size={14} aria-hidden="true" />
            Contents
          </button>
          <div class="view-switcher" aria-label="Document view">
          <button type="button" class:active={viewMode === "read"} onclick={() => (viewMode = "read")}>Read</button>
          <button type="button" class:active={viewMode === "edit"} onclick={() => (viewMode = "edit")}>Edit</button>
            <button type="button" class:active={viewMode === "split"} onclick={() => (viewMode = "split")}>Split</button>
          </div>
        </div>
      {/if}
    </header>

    {#if contentsOpen && document && !selectedIsImage}
      <nav class="contents-panel" aria-label="Document contents">
        <header>
          <span>Contents</span>
          <button type="button" aria-label="Close contents" onclick={() => (contentsOpen = false)}>×</button>
        </header>
        {#if contentsHeadings.length}
          <ol>
            {#each contentsHeadings as heading, index (`${heading.id}-${index}`)}
              <li style:--heading-level={heading.level}>
                <button type="button" onclick={() => jumpToHeading(heading.id)}>{heading.text}</button>
              </li>
            {/each}
          </ol>
        {:else}
          <p>No headings in this document.</p>
        {/if}
      </nav>
    {/if}

    {#if ambiguousLink}
      <div class="link-picker" role="dialog" aria-modal="false" aria-labelledby="ambiguous-link-title">
        <header>
          <div>
            <p class="eyebrow">Ambiguous link</p>
            <h3 id="ambiguous-link-title">Choose “{ambiguousLink.query}”</h3>
          </div>
          <button type="button" aria-label="Dismiss ambiguous link choices" onclick={dismissAmbiguousLink}>×</button>
        </header>
        <p>More than one project document matches this link. Choose the intended document.</p>
        <div class="link-picker__candidates">
          {#each ambiguousLink.candidates as candidate (candidate.id)}
            <button type="button" class="link-picker__candidate" onclick={() => selectAmbiguousLink(candidate)}>
              <strong>{candidate.title}</strong>
              <span>{candidate.document_type} · {candidate.relative_path}</span>
              {#if candidate.aliases.length}<span>Aliases: {candidate.aliases.join(", ")}</span>{/if}
              <code>{candidate.id}</code>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    {#if linkMessage}
      <div class="workspace-error" role="alert"><AlertTriangle size={18} aria-hidden="true" />{linkMessage}</div>
    {/if}

    {#if error}
      <div class="workspace-error" role="alert"><AlertTriangle size={18} aria-hidden="true" />{error}</div>
    {:else if selectedIsImage && imageDataUrl}
      <section class="image-viewer" aria-label="Image viewer">
        <img src={imageDataUrl} alt={selectedEntry?.title ?? "Story Development image"} />
        <div class="image-viewer__caption">
          <strong>{selectedEntry?.title}</strong>
          <span>{selectedEntry?.relative_path}</span>
        </div>
      </section>
    {:else if document}
      <div class:split={viewMode === "split"} class="document-layout">
        {#if viewMode === "edit" || viewMode === "split"}
          <section class="editor-pane" aria-label="Markdown editor">
            {#if viewMode === "split"}
              <div class="split-pane-label">Editor</div>
            {/if}
            <div class="editor-toolbar">
              <div class="markdown-formatting" aria-label="Markdown formatting">
                <button type="button" aria-label="Bold" title="Bold" onclick={() => wrapSelection("**")}><Bold size={14} aria-hidden="true" /></button>
                <button type="button" aria-label="Italic" title="Italic" onclick={() => wrapSelection("*")}><Italic size={14} aria-hidden="true" /></button>
                <button type="button" aria-label="Strikethrough" title="Strikethrough" onclick={() => wrapSelection("~~")}><Strikethrough size={14} aria-hidden="true" /></button>
                <span class="editor-toolbar__divider"></span>
                <button type="button" aria-label="Heading 1" title="Heading 1" onclick={() => applyHeading(1)}><Heading1 size={14} aria-hidden="true" /></button>
                <button type="button" aria-label="Heading 2" title="Heading 2" onclick={() => applyHeading(2)}><Heading2 size={14} aria-hidden="true" /></button>
                <button type="button" aria-label="Heading 3" title="Heading 3" onclick={() => applyHeading(3)}><Heading3 size={14} aria-hidden="true" /></button>
              </div>
              <div class="editor-toolbar__actions">
                {#if bodyDirty}<span class="unsaved-indicator">Unsaved</span>{/if}
                <button
                  type="button"
                  class="editor-save"
                  disabled={!bodyDirty || savingBody}
                  onclick={saveBody}
                >
                  {#if savingBody}
                    <span class="spin"><Loader2 size={14} aria-hidden="true" /></span>
                    Saving…
                  {:else}
                    <Save size={14} aria-hidden="true" />
                    Save
                  {/if}
                </button>
              </div>
            </div>
            <textarea
              class="markdown-editor"
              aria-label="Document Markdown source including YAML frontmatter"
              bind:this={markdownEditor}
              bind:value={bodyDraft}
              spellcheck="true"
            ></textarea>
            {#if bodyError}
              <div class="editor-error" role="alert">{bodyError}</div>
            {/if}
          </section>
        {/if}

        {#if viewMode === "read" || viewMode === "split"}
          <section class="preview-pane" aria-label="Rendered Wiki page">
            {#if viewMode === "split"}
              <div class="split-pane-label">Preview</div>
            {/if}
            <article class="wiki-page">
              <p class="eyebrow">{document.document_type}</p>
              <h1>{document.title ?? document.relative_path.split("/").slice(-1)[0]?.replace(/\.md$/, "")}</h1>
              <div class="wiki-body" role="presentation" onclick={handlePreviewClick} onkeydown={handlePreviewKeydown}>{@html renderedBody}</div>
            </article>
          </section>
        {/if}

        <aside class="metadata-panel" aria-label="Document overview">
          {#if characterPanel}
            <header class="metadata-panel__header character-panel__header">
              <p class="eyebrow">Character Profile</p>
              <h3>{document.title ?? "Character"}</h3>
              {#if characterPanel.subtitle}<small>{characterPanel.subtitle}</small>{/if}
            </header>

            {#if overviewImageDataUrl}
              <img class="metadata-panel__image character-panel__portrait" src={overviewImageDataUrl}
                alt={document.title ? `${document.title} portrait` : "Character portrait"} />
            {:else if overviewImageError}
              <p class="metadata-panel__image-error" role="alert">{overviewImageError}</p>
            {/if}

            {#if characterPanel.role || characterPanel.pov || characterPanel.status}
              <div class="character-panel__badges" aria-label="Character status">
                {#if characterPanel.role}<span>{characterPanel.role}</span>{/if}
                {#if characterPanel.pov}<span>{characterPanel.pov}</span>{/if}
                {#if characterPanel.status}<span>{characterPanel.status}</span>{/if}
              </div>
            {/if}

            <section class="character-panel__section">
              <h4>Quick Facts</h4>
              <dl class="character-panel__facts">
                <dt>Age</dt><dd>{characterPanel.age ?? "—"}{#if characterPanel.birthYear} <span class="character-panel__secondary">(b. {characterPanel.birthYear})</span>{/if}</dd>
                <dt>Occupation</dt><dd>{characterPanel.occupation ?? "—"}</dd>
                <dt>Affiliation</dt><dd>{characterPanel.affiliation ?? "—"}</dd>
                <dt>Location</dt><dd>{characterPanel.location ?? "—"}</dd>
                <dt>Narrative Role</dt><dd>{characterPanel.narrativeRole ?? "—"}</dd>
                <dt>Aliases</dt><dd>{characterPanel.aliases.length ? characterPanel.aliases.join(", ") : "—"}</dd>
              </dl>
            </section>

            <section class="character-panel__section">
              <h4>Relationships</h4>
              {#if characterPanel.relationships.length}
                <div class="character-panel__links">{#each characterPanel.relationships as relationship}<span>{relationship}</span>{/each}</div>
              {:else}<p class="character-panel__empty">No structured relationships yet.</p>{/if}
            </section>

            <section class="character-panel__section">
              <h4>Story Tracking</h4>
              <dl class="character-panel__facts">
                <dt>First Appearance</dt><dd>{characterPanel.firstAppearance ?? "—"}</dd>
                <dt>Last Appearance</dt><dd>{characterPanel.lastAppearance ?? "—"}</dd>
              </dl>
            </section>

            <section class="character-panel__section">
              <h4>Tags</h4>
              {#if characterPanel.tags.length}
                <div class="character-panel__tags">{#each characterPanel.tags as tag}<span>{tag}</span>{/each}</div>
              {:else}<p class="character-panel__empty">No tags.</p>{/if}
            </section>
          {:else}
            <header class="metadata-panel__header">
              <p class="eyebrow">Overview</p>
              <h3>{document.title ?? "Properties"}</h3>
              {#if activeEntityTemplate}<small>{activeEntityTemplate.name}</small>{/if}
            </header>
            {#if overviewImageDataUrl}
              <img class="metadata-panel__image" src={overviewImageDataUrl}
                alt={document.title ? `${document.title} reference` : "Document reference"} />
            {:else if overviewImageError}
              <p class="metadata-panel__image-error" role="alert">{overviewImageError}</p>
            {/if}
            <dl class="metadata-panel__fields">
              <dt>Type</dt><dd>{document.document_type}</dd>
              {#each overviewMetadata as field (field.key)}
                <dt>{metadataLabel(field.key)}</dt>
                <dd>{displayMetadataValue(field.value)}</dd>
              {/each}
            </dl>
          {/if}

          <details class="metadata-panel__technical">
            <summary>Technical details</summary>
            <dl><dt>Stable ID</dt><dd>{document.id}</dd><dt>Path</dt><dd>{document.relative_path}</dd></dl>
          </details>
        </aside>
      </div>
    {:else}
      <div class="empty-document">Select a {workspaceLabel} file.</div>
    {/if}
  </div>
</section>

{#if trashOpen}
  <div class="template-create-backdrop" role="presentation">
    <div class="template-create-dialog" role="dialog" aria-modal="true" aria-label="Restore deleted documents" tabindex="-1">
      <h2>Deleted Documents</h2>
      <p>Restore a document to its original path. Existing files will never be overwritten.</p>
      {#if trashLoading}<p>Loading project trash…</p>
      {:else if trashItems.length === 0 && !trashError}<p>No deleted documents found.</p>{/if}
      {#each trashItems as item (item.entry_id)}
        <div class="template-create-actions">
          <span>{item.original_path}</span>
          <button type="button" disabled={trashRestoring || trashLoading} onclick={() => restoreTrashItem(item)}>Restore</button>
        </div>
      {/each}
      {#if trashError}<p role="alert">{trashError}</p>{/if}
      <div class="template-create-actions">
        <button type="button" disabled={trashRestoring} onclick={() => (trashOpen = false)}>Close</button>
      </div>
    </div>
  </div>
{/if}

{#if deleteOpen}
  <div class="template-create-backdrop" role="presentation">
    <div class="template-create-dialog" role="dialog" aria-modal="true" aria-label="Confirm document deletion" tabindex="-1">
      <h2>Move Document to Trash?</h2>
      <p><strong>{document?.title ?? document?.relative_path}</strong> will be removed from the Wiki and preserved in this project's recoverable trash. Linked references are not changed.</p>
      {#if bodyDirty}<p role="alert">You have unsaved changes. Save them before deleting this document.</p>{/if}
      {#if deleteError}<p role="alert">{deleteError}</p>{/if}
      <div class="template-create-actions">
        <button type="button" onclick={() => (deleteOpen = false)} disabled={deleting}>Cancel</button>
        <button type="button" onclick={confirmDeleteDocument} disabled={deleting || savingBody || bodyDirty || !document}>Move to Trash</button>
      </div>
    </div>
  </div>
{/if}

{#if relocateOpen}
  <div class="template-create-backdrop" role="presentation">
    <div class="template-create-dialog" role="dialog" aria-modal="true" aria-label="Rename or move document" tabindex="-1">
      <h2>Rename or Move Document</h2>
      <p>Enter a project-relative Markdown path. The document's stable ID and contents will be preserved.</p>
      <label for="relocate-path">Destination path</label>
      <input id="relocate-path" bind:value={relocateDestination} disabled={relocating} />
      {#if relocateError}<p role="alert">{relocateError}</p>{/if}
      <div class="template-create-actions">
        <button type="button" onclick={() => (relocateOpen = false)} disabled={relocating}>Cancel</button>
        <button type="button" onclick={submitRelocateDocument} disabled={relocating || !relocateDestination.trim()}>Move Document</button>
      </div>
    </div>
  </div>
{/if}

{#if assignTemplateOpen}
  <div class="template-create-backdrop" role="presentation">
    <div class="template-create-dialog" role="dialog" aria-modal="true" aria-label="Assign document template" tabindex="-1">
      <h2>Assign or Change Template</h2>
      <p>Existing Markdown and metadata are preserved. Only missing template sections are appended.</p>
      <label for="assign-template-select">Template</label>
      <select id="assign-template-select" bind:value={assignTemplateId} onchange={() => (assignTemplatePreview = null)}>
        {#each storyTemplateGroups as group (group.category)}
          <optgroup label={group.label}>
            {#each group.templates as template (template.id)}
              <option value={template.id}>{template.name}</option>
            {/each}
          </optgroup>
        {/each}
      </select>
      <button type="button" onclick={previewTemplateAssignment} disabled={!assignTemplateId || assigningTemplate}>Preview Changes</button>
      {#if assignTemplatePreview}
        <h3>Sections to add</h3>
        {#if assignTemplatePreview.added_headings.length}
          <ul>{#each assignTemplatePreview.added_headings as heading}<li>{heading}</li>{/each}</ul>
        {:else}
          <p>No missing sections. Only template metadata will change.</p>
        {/if}
        <details>
          <summary>Preview merged Markdown</summary>
          <pre class="template-merge-preview">{assignTemplatePreview.proposed_body}</pre>
        </details>
      {/if}
      {#if assignTemplateError}<p role="alert">{assignTemplateError}</p>{/if}
      <div class="template-create-actions">
        <button type="button" onclick={() => (assignTemplateOpen = false)} disabled={assigningTemplate}>Cancel</button>
        <button type="button" onclick={confirmTemplateAssignment} disabled={!assignTemplatePreview || assigningTemplate}>Apply Template</button>
      </div>
    </div>
  </div>
{/if}

{#if templateManagerOpen}
  <EntityTemplateManager
    onClose={() => (templateManagerOpen = false)}
    onRegistryChanged={(registry) => (entityTemplates = registry)}
  />
{/if}

  {#if folderDialogOpen}
    <div class="template-create-backdrop template-create-backdrop--folder" role="presentation">
      <div class="template-create-dialog" role="dialog" aria-modal="true" aria-label="Create folder" tabindex="-1">
        <h2>Create Folder</h2>
        <label for="folder-parent">Parent folder</label>
        <select id="folder-parent" bind:value={folderParent}>
          {#each storyFolders as folder (folder)}<option value={folder}>{folder}</option>{/each}
        </select>
        <label for="folder-name">Folder name</label>
        <input id="folder-name" bind:value={folderName} />
        {#if folderError}<p role="alert">{folderError}</p>{/if}
        <div class="template-create-actions">
          <button type="button" onclick={() => (folderDialogOpen = false)} disabled={creatingFolder}>Cancel</button>
          <button type="button" onclick={async () => { await submitNewFolder(); if (!folderDialogOpen && newDocumentOpen) newDocumentFolder = selectedFolder ?? newDocumentFolder; }} disabled={creatingFolder || !folderName.trim()}>Create Folder</button>
        </div>
      </div>
    </div>
  {/if}
  {#if newDocumentOpen}
    <div class="template-create-backdrop" role="presentation">
      <div class="template-create-dialog" role="dialog" aria-modal="true" aria-label="Create document from template" tabindex="-1">
        <h2>Create Document</h2>
        <label for="new-doc-title">Document name</label>
        <input id="new-doc-title" bind:value={newDocumentTitle} placeholder="Document title" />
        <label for="new-doc-folder">Destination folder</label>
        <select id="new-doc-folder" bind:value={newDocumentFolder}>
          {#each storyFolders as folder (folder)}
            <option value={folder}>{folder}</option>
          {/each}
        </select>
        <button type="button" onclick={() => { folderParent = newDocumentFolder; folderName = ""; folderError = null; folderDialogOpen = true; }}>+ Create folder…</button>
        <label for="new-doc-mode">Document type</label>
        <select id="new-doc-mode" bind:value={newDocumentMode}>
          <option value="template">Create from template</option>
          <option value="empty">Empty Markdown document (assign template later)</option>
        </select>
        {#if newDocumentMode === "template"}
        <label for="new-doc-template">Template</label>
        <select id="new-doc-template" bind:value={newDocumentTemplate} onchange={() => (newDocumentFields = {})}>
          {#each storyTemplateGroups as group (group.category)}
            <optgroup label={group.label}>
              {#each group.templates as template (template.id)}
                <option value={template.id}>{template.name}</option>
              {/each}
            </optgroup>
          {/each}
        </select>
        {#if selectedCreationTemplate}
          {#each selectedCreationTemplate.fields ?? [] as field (field.key)}
            <label for={"new-doc-" + field.key}>{field.label}</label>
            {#if field.type === "multiline"}
              <textarea id={"new-doc-" + field.key} value={newDocumentFields[field.key] ?? ""}
                oninput={e => (newDocumentFields[field.key] = e.currentTarget.value)} rows="3"></textarea>
            {:else if field.type === "select" || field.type === "boolean"}
              <select id={"new-doc-" + field.key} value={newDocumentFields[field.key] ?? ""}
                onchange={e => (newDocumentFields[field.key] = e.currentTarget.value)}>
                <option value="">Not specified</option>
                {#each field.type === "boolean" ? ["true", "false"] : field.options as option}
                  <option value={option}>{option}</option>
                {/each}
              </select>
            {:else}
              <input id={"new-doc-" + field.key} value={newDocumentFields[field.key] ?? ""}
                placeholder={field.type === "list" ? "Comma-separated values" : ""}
                oninput={e => (newDocumentFields[field.key] = e.currentTarget.value)} />
            {/if}
          {/each}
        {/if}
        {/if}
        {#if newDocumentError}<p role="alert">{newDocumentError}</p>{/if}
        <div class="template-create-actions">
          <button type="button" disabled={creatingDocument} onclick={() => (newDocumentOpen = false)}>Cancel</button>
          <button type="button" disabled={creatingDocument || !newDocumentTitle.trim() || (newDocumentMode === "template" && !selectedCreationTemplate)}
            onclick={submitNewDocument}>{creatingDocument ? "Creating…" : "Create Document"}</button>
        </div>
      </div>
    </div>
  {/if}
<style>
  .story-development__template-actions { display:flex; flex-direction:column; gap:4px; padding:8px; border-top:1px solid var(--color-border); background:var(--color-surface); }
  .story-development__template-actions button { text-align:left; border:1px solid var(--color-border); background:var(--color-surface-raised); color:var(--color-text); border-radius:5px; padding:7px 10px; cursor:pointer; }
  .story-development__template-actions button:disabled { opacity:.45; cursor:not-allowed; }
  .template-merge-preview { max-height:300px; overflow:auto; white-space:pre-wrap; font-size:12px; }
  .template-create-backdrop--folder { z-index: 1200 !important; }
  .template-create-backdrop { position: fixed; inset: 0; z-index: 1100; background: #000b; display: grid; place-items: center; padding: 1rem; }
  .template-create-dialog { background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text); border-radius: 8px; padding: 1.5rem; width: min(640px, 95vw); max-height: 85vh; overflow: auto; display: grid; gap: .55rem; }
  .template-create-dialog input, .template-create-dialog select, .template-create-dialog textarea { width: 100%; background: var(--color-control-bg); border: 1px solid var(--color-control-border); color: var(--color-control-text); padding: .55rem; border-radius: 4px; }
  .template-create-actions { display: flex; justify-content: end; gap: .7rem; margin-top: 1rem; }


  .story-development { display:flex; flex:1; min-width:0; min-height:0; background:var(--color-bg); }
  .story-development__create-footer { display:flex; align-items:center; gap:6px; padding:8px; border-top:1px solid var(--color-border); }
  .story-development__create-footer span { flex:1; min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; font-size:11px; color:var(--color-text-muted); }
  .story-development__create-footer button { background:transparent; border:1px solid var(--color-border); color:var(--color-text); padding:6px; border-radius:5px; cursor:pointer; }
  .story-development__nav { flex:0 0 292px; width:292px; min-height:0; display:flex; flex-direction:column; border-right:1px solid var(--color-border); background:var(--color-surface); }
  .story-development__nav-header { display:flex; align-items:center; justify-content:space-between; gap:8px; padding:14px 12px 10px; border-bottom:1px solid var(--color-border); }
  .story-development__nav-header h2 { margin:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:var(--color-text); font:600 17px/1.2 var(--font-display); }
  .eyebrow { margin:0 0 3px; color:var(--color-accent-text); font:600 10px/1.3 var(--font-ui); letter-spacing:.08em; text-transform:uppercase; }
  .story-development__search { padding:10px 8px; border-bottom:1px solid var(--color-border); }
  .story-development__search { display:grid; gap:6px; }
  .story-development__search input { width:100%; min-height:34px; padding:6px 9px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:var(--color-surface-sunken); color:var(--color-text); }
  .story-development__tree { flex:1; min-height:0; overflow:auto; padding:6px 0; }
  .tree-list { margin:0; padding:0; list-style:none; }
  .tree-list .tree-list { padding-left:14px; }
  .tree-row { width:100%; min-height:28px; display:flex; align-items:center; gap:5px; padding:4px 8px; border:0; border-radius:0; background:transparent; color:var(--color-text-muted); text-align:left; cursor:pointer; }
  .tree-row:hover { background:var(--color-surface-raised); color:var(--color-text); }
  .tree-file--active { background:var(--color-surface-raised); color:var(--color-accent-text); }
  .tree-row span:last-child { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .tree-spacer { width:14px; flex:0 0 14px; }
  .status,.empty { padding:12px; display:flex; align-items:center; gap:8px; color:var(--color-text-muted); }
  .issues { max-height:160px; overflow:auto; padding:10px 12px; border-top:1px solid var(--color-border); }
  .issues h3 { margin:0 0 8px; display:flex; align-items:center; gap:5px; color:var(--color-error); font-size:11px; text-transform:uppercase; }
  .issue { display:grid; gap:2px; margin-bottom:8px; color:var(--color-text-muted); font-size:10px; }
  .story-development__workspace { position:relative; flex:1; min-width:0; min-height:0; display:flex; flex-direction:column; }
  .document-tabs { min-height:34px; display:flex; align-items:stretch; overflow-x:auto; border-bottom:1px solid var(--color-border); background:var(--color-surface-sunken); }
  .document-tab { min-width:120px; max-width:240px; display:flex; align-items:stretch; border-right:1px solid var(--color-border); background:transparent; color:var(--color-text-muted); }
  .document-tab:hover { background:var(--color-surface); color:var(--color-text); }
  .document-tab--active { background:var(--color-surface); color:var(--color-text); box-shadow:inset 0 -2px 0 var(--color-accent-text); }
  .document-tab__activate { min-width:0; flex:1; display:flex; align-items:center; gap:6px; padding:7px 4px 7px 8px; border:0; background:transparent; color:inherit; cursor:pointer; }
  .document-tab__activate > span:not(.document-tab__dirty) { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .document-tab__dirty { color:var(--color-accent-text); font-size:9px; }
  .document-tab__close { align-self:center; flex:0 0 22px; height:22px; margin-right:4px; border:0; border-radius:4px; background:transparent; color:var(--color-text-muted); font-size:15px; line-height:1; cursor:pointer; }
  .document-tab__close:hover { background:var(--color-surface-raised); color:var(--color-text); }
  .document-tabs__reopen { flex:0 0 34px; display:grid; place-items:center; border:0; border-right:1px solid var(--color-border); background:transparent; color:var(--color-text-muted); cursor:pointer; }
  .document-tabs__reopen:hover { background:var(--color-surface); color:var(--color-accent-text); }
  .document-toolbar { min-height:46px; display:flex; align-items:center; justify-content:space-between; gap:12px; padding:0 14px; border-bottom:1px solid var(--color-border); background:var(--color-surface); }
  .document-toolbar__controls { display:flex; align-items:center; gap:8px; }
  .contents-toggle { display:inline-flex; align-items:center; gap:5px; padding:5px 9px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:var(--color-surface-sunken); color:var(--color-text-muted); cursor:pointer; }
  .contents-toggle:hover,.contents-toggle.active { background:var(--color-surface-raised); color:var(--color-accent-text); }
  .contents-panel { position:absolute; z-index:8; top:82px; left:12px; width:250px; max-height:calc(100% - 98px); overflow:auto; padding:10px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:var(--color-surface); box-shadow:0 12px 32px rgb(0 0 0 / .28); }
  .contents-panel header { display:flex; align-items:center; justify-content:space-between; gap:8px; padding:2px 4px 8px; border-bottom:1px solid var(--color-border); color:var(--color-text); font-weight:600; }
  .contents-panel header button { border:0; background:transparent; color:var(--color-text-muted); cursor:pointer; font-size:16px; }
  .contents-panel ol { margin:8px 0 0; padding:0; list-style:none; }
  .contents-panel li { padding-left:calc((var(--heading-level) - 1) * 12px); }
  .contents-panel li button { width:100%; padding:5px 6px; border:0; border-radius:4px; background:transparent; color:var(--color-text-muted); text-align:left; cursor:pointer; }
  .contents-panel li button:hover { background:var(--color-surface-raised); color:var(--color-text); }
  .contents-panel p { margin:10px 6px; color:var(--color-text-muted); font-size:11px; }
  .document-title { min-width:0; display:flex; align-items:center; gap:8px; color:var(--color-text); font:600 15px/1.2 var(--font-display); }
  .document-title span { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .view-switcher { display:flex; gap:4px; padding:3px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:var(--color-surface-sunken); }
  .view-switcher button { padding:5px 10px; border:0; border-radius:calc(var(--radius-m) - 2px); background:transparent; color:var(--color-text-muted); cursor:pointer; }
  .view-switcher button.active { background:var(--color-surface-raised); color:var(--color-accent-text); }
  .document-layout { flex:1; min-width:0; min-height:0; display:grid; grid-template-columns:minmax(0,1fr) 300px; }
  .document-layout.split { grid-template-columns:minmax(0,1fr) minmax(0,1fr) 300px; }
  .editor-pane,.preview-pane,.metadata-panel { min-width:0; min-height:0; overflow:auto; }
  .split-pane-label { position:sticky; top:0; z-index:2; min-height:28px; display:flex; align-items:center; padding:0 12px; border-bottom:1px solid var(--color-border); background:var(--color-surface); color:var(--color-text-muted); font-size:10px; font-weight:600; letter-spacing:.06em; text-transform:uppercase; }
  .editor-pane { display:flex; flex-direction:column; border-right:1px solid var(--color-border); background:var(--color-surface-sunken); }
  .editor-toolbar { min-height:38px; display:flex; align-items:center; justify-content:space-between; gap:10px; padding:0 10px 0 14px; border-bottom:1px solid var(--color-border); color:var(--color-text-muted); font-size:11px; }
  .markdown-formatting { display:flex; align-items:center; gap:3px; }
  .markdown-formatting button { display:inline-flex; align-items:center; justify-content:center; width:28px; height:26px; padding:0; border:0; border-radius:4px; background:transparent; color:var(--color-text-muted); cursor:pointer; }
  .markdown-formatting button:hover { background:var(--color-surface-raised); color:var(--color-accent-text); }
  .editor-toolbar__divider { width:1px; height:18px; margin:0 3px; background:var(--color-border); }
  .editor-toolbar__actions { display:flex; align-items:center; gap:8px; }
  .unsaved-indicator { color:var(--color-accent-text); }
  .unsaved-indicator::before { content:"• "; }
  .editor-save { display:inline-flex; align-items:center; gap:5px; padding:5px 9px; border:1px solid var(--color-border); border-radius:5px; background:var(--color-surface-raised); color:var(--color-text); cursor:pointer; }
  .editor-save:disabled { opacity:.45; cursor:default; }
  .markdown-editor { flex:1; width:100%; min-height:0; box-sizing:border-box; resize:none; border:0; outline:0; padding:24px; background:transparent; color:var(--color-text); font:13px/1.65 ui-monospace,SFMono-Regular,Consolas,monospace; tab-size:2; }
  .editor-error { padding:8px 12px; border-top:1px solid var(--color-error); color:var(--color-error); font-size:11px; }
  .preview-pane { background:var(--color-bg); }
  .image-viewer { flex:1; min-width:0; min-height:0; display:flex; flex-direction:column; align-items:center; justify-content:center; gap:18px; overflow:auto; padding:28px; background:var(--color-bg); }
  .image-viewer img { max-width:min(900px,92%); max-height:calc(100% - 72px); object-fit:contain; border-radius:var(--radius-m); box-shadow:0 10px 36px rgb(0 0 0 / .28); }
  .image-viewer__caption { display:flex; flex-direction:column; gap:3px; align-items:center; color:var(--color-text-muted); font-size:11px; }
  .image-viewer__caption strong { color:var(--color-text); font-size:13px; }
  .wiki-page { width:min(850px,calc(100% - 56px)); margin:0 auto; padding:52px 0 80px; }
  .wiki-page h1 { margin:0 0 28px; padding-bottom:12px; border-bottom:1px solid var(--color-border); color:var(--color-text); font:600 32px/1.15 var(--font-display); }
  .wiki-body :global(h1),.wiki-body :global(h2),.wiki-body :global(h3),.wiki-body :global(h4) { margin:28px 0 10px; padding-bottom:7px; border-bottom:1px solid var(--color-border); color:var(--color-text); font-family:var(--font-display); }
  .wiki-body :global(h1) { font-size:24px; line-height:1.3; font-weight:600; }
  .wiki-body :global(h2) { font-size:19px; line-height:1.35; font-weight:600; }
  .wiki-body :global(h3) { font-size:17px; line-height:1.4; font-weight:600; margin-top:18px; }
  .wiki-body :global(h4) { font-size:16px; line-height:1.4; font-weight:600; margin-top:16px; }
  .wiki-body :global(p),.wiki-body :global(li) { color:var(--color-text); line-height:1.7; }
  .wiki-body :global(.wikilink) { color:var(--color-accent-text); text-decoration:underline; text-decoration-style:dotted; }
  .wiki-body :global(code) { padding:1px 4px; border-radius:4px; background:var(--color-surface-raised); font-family:ui-monospace,SFMono-Regular,Consolas,monospace; }
  .wiki-body :global(.markdown-table-wrap) { width:100%; margin:14px 0 22px; overflow-x:auto; }
  .wiki-body :global(table) { width:100%; border-collapse:collapse; color:var(--color-text); font-size:14px; line-height:1.45; }
  .wiki-body :global(th),.wiki-body :global(td) { padding:9px 12px; border:1px solid var(--color-border); text-align:left; vertical-align:top; }
  .wiki-body :global(th) { background:var(--color-surface-raised); color:var(--color-text); font-weight:700; }
  .wiki-body :global(tbody tr:nth-child(even)) { background:color-mix(in srgb,var(--color-surface-raised) 45%,transparent); }
  .metadata-panel { padding:14px 16px 18px; border-left:1px solid var(--color-border); background:var(--color-surface); overflow-y:auto; }
  .metadata-panel__header { margin-bottom:14px; padding-bottom:10px; border-bottom:1px solid var(--color-border); }
  .metadata-panel__header h3 { margin:0; color:var(--color-text); font:600 16px/1.3 var(--font-display); }
  .metadata-panel__header small { display:block; margin-top:3px; color:var(--color-text-muted); font-size:10px; }
  .metadata-panel__image-error {
    margin: 0 0 12px;
    padding: 8px 10px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    color: var(--color-text-muted);
    background: var(--color-surface-sunken);
    font-size: 11px;
    line-height: 1.45;
  }

  .character-panel__header { margin-bottom:10px; padding-bottom:9px; text-align:left; }
  .character-panel__header .eyebrow { margin-bottom:4px; color:var(--color-accent-text); font-size:9px; letter-spacing:.12em; }
  .character-panel__header h3 { font-size:20px; letter-spacing:.035em; text-transform:uppercase; }
  .character-panel__header small { margin-top:3px; font-size:11px; }
  .character-panel__portrait { display:block; height:auto; max-height:340px; object-fit:contain; object-position:center; }
  .character-panel__badges { display:flex; flex-wrap:wrap; gap:6px; margin:2px 0 12px; padding-bottom:12px; border-bottom:1px solid var(--color-border); }
  .character-panel__badges span { padding:4px 8px; border:1px solid color-mix(in srgb,var(--color-accent-text) 65%,var(--color-border)); border-radius:5px; background:color-mix(in srgb,var(--color-accent-text) 9%,var(--color-surface-sunken)); color:var(--color-accent-text); font-size:10px; line-height:1.2; }
  .character-panel__section { margin-top:0; padding:11px 0 12px; border-bottom:1px solid var(--color-border); }
  .character-panel__section h4 { margin:0 0 9px; color:var(--color-accent-text); font:600 12px/1.3 var(--font-display); letter-spacing:.04em; text-transform:uppercase; }
  .character-panel__facts { display:grid; grid-template-columns:88px minmax(0,1fr); gap:6px 8px; margin:0; font-size:11px; }
  .character-panel__facts dt { color:var(--color-accent-text); font-weight:500; }
  .character-panel__facts dd { margin:0; color:var(--color-text); line-height:1.35; overflow-wrap:anywhere; }
  .character-panel__secondary { color:var(--color-text-muted); }
  .character-panel__links { display:grid; gap:5px; }
  .character-panel__links span { width:auto; padding:6px 8px; border-left:2px solid var(--color-accent-text); border-radius:3px; background:var(--color-surface-sunken); color:var(--color-text); font-size:11px; }
  .character-panel__empty { margin:0; color:var(--color-text-muted); font-size:10px; font-style:italic; }
  .character-panel__tags { display:flex; flex-wrap:wrap; gap:5px; }
  .character-panel__tags span { padding:3px 7px; border:1px solid var(--color-border); border-radius:5px; background:var(--color-surface-sunken); color:var(--color-text-muted); font-size:10px; line-height:1.2; }
  .metadata-panel__image { width:100%; max-height:300px; object-fit:cover; margin:0 0 14px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:var(--color-surface-sunken); }
  .metadata-panel__fields,
  .metadata-panel__technical dl { display:grid; grid-template-columns:96px minmax(0,1fr); gap:9px 10px; margin:0; font-size:11px; }
  .metadata-panel dt { color:var(--color-text-muted); font-weight:600; }
  .metadata-panel dd { margin:0; overflow-wrap:anywhere; color:var(--color-text); line-height:1.45; }
  .metadata-panel__technical { margin-top:16px; padding-top:12px; border-top:1px solid var(--color-border); color:var(--color-text-muted); font-size:11px; }
  .metadata-panel__technical summary { margin-bottom:10px; cursor:pointer; color:var(--color-text-muted); }
  .link-picker { margin:12px 14px 0; padding:12px; border:1px solid var(--color-border); border-radius:var(--radius-m); background:var(--color-surface); color:var(--color-text); }
  .link-picker header { display:flex; align-items:flex-start; justify-content:space-between; gap:12px; }
  .link-picker h3 { margin:0; font:600 16px/1.3 var(--font-display); }
  .link-picker header > button { border:0; background:transparent; color:var(--color-text-muted); cursor:pointer; font-size:18px; }
  .link-picker > p { margin:8px 0 10px; color:var(--color-text-muted); font-size:11px; }
  .link-picker__candidates { display:grid; gap:6px; }
  .link-picker__candidate { display:grid; gap:2px; width:100%; padding:9px 10px; border:1px solid var(--color-border); border-radius:6px; background:var(--color-surface-sunken); color:var(--color-text); text-align:left; cursor:pointer; }
  .link-picker__candidate:hover,.link-picker__candidate:focus-visible { border-color:var(--color-accent-text); background:var(--color-surface-raised); }
  .link-picker__candidate span,.link-picker__candidate code { color:var(--color-text-muted); font-size:10px; overflow-wrap:anywhere; }
  .workspace-error,.empty-document { margin:auto; display:flex; align-items:center; gap:8px; color:var(--color-text-muted); }
  .workspace-error { color:var(--color-error); }
  .spin { display:inline-flex; animation:spin .9s linear infinite; }
  @keyframes spin { to { transform:rotate(360deg); } }
</style>
