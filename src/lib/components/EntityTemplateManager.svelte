<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { Plus, RefreshCw, Save, Trash2, X } from "lucide-svelte";

  interface TemplateField {
    key: string;
    label: string;
    type: string;
    required: boolean;
    options: string[];
  }

  interface EntityTemplate {
    id: string;
    name: string;
    category: string;
    scope: string;
    document_type: string;
    subtype: string | null;
    version: number;
    description: string | null;
    overview_fields: string[];
    image_field: string;
    fields: TemplateField[];
    body: string;
    source: string;
    relative_path: string | null;
  }

  interface Registry {
    templates: EntityTemplate[];
    issues: { relative_path: string; message: string }[];
    user_templates_dir: string;
  }

  let {
    onClose,
    onRegistryChanged,
  }: {
    onClose: () => void;
    onRegistryChanged?: (registry: Registry) => void;
  } = $props();

  let registry = $state<Registry>({ templates: [], issues: [], user_templates_dir: "" });
  let selectedId = $state<string | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let error = $state<string | null>(null);

  let editingSource = $state<"new" | "user" | "built-in">("new");
  let previousRelativePath = $state<string | null>(null);
  let id = $state("");
  let name = $state("");
  let category = $state("character");
  let scope = $state("story-development");
  let documentType = $state("character");
  let subtype = $state("");
  let description = $state("");
  let relativePath = $state("");
  let overviewFields = $state("");
  let imageField = $state("image");
  let fieldsText = $state("image | Image | image\n");
  let body = $state("# Summary\n\n## Notes\n");

  const canEdit = $derived(editingSource !== "built-in");

  onMount(loadRegistry);

  async function loadRegistry() {
    loading = true;
    error = null;
    try {
      registry = await invoke<Registry>("get_entity_template_registry");
      onRegistryChanged?.(registry);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function fieldLines(fields: TemplateField[]): string {
    return fields
      .map((field) => {
        const required = field.required ? "required" : "";
        const options = field.options.join(",");
        return [field.key, field.label, field.type, required, options].join(" | ").replace(/\s+\|\s*$/, "");
      })
      .join("\n");
  }

  function selectTemplate(template: EntityTemplate) {
    selectedId = template.id;
    editingSource = template.source === "user" ? "user" : "built-in";
    previousRelativePath = template.relative_path;
    id = template.id;
    name = template.name;
    category = template.category;
    scope = template.scope;
    documentType = template.document_type;
    subtype = template.subtype ?? "";
    description = template.description ?? "";
    relativePath = template.relative_path ?? "";
    overviewFields = template.overview_fields.join(", ");
    imageField = template.image_field || "image";
    fieldsText = fieldLines(template.fields);
    body = template.body;
    error = null;
  }

  function newTemplate() {
    selectedId = null;
    editingSource = "new";
    previousRelativePath = null;
    id = "";
    name = "";
    category = "character";
    scope = "story-development";
    documentType = "character";
    subtype = "";
    description = "";
    relativePath = "Custom/New Template.md";
    overviewFields = "";
    imageField = "image";
    fieldsText = "image | Image | image\n";
    body = "# Summary\n\n## Notes\n";
    error = null;
  }

  function parseFields(): TemplateField[] {
    return fieldsText
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .map((line, index) => {
        const [key = "", label = "", type = "", required = "", options = ""] = line
          .split("|")
          .map((part) => part.trim());
        if (!key || !label || !type) {
          throw new Error(`Field line ${index + 1} must contain: key | label | type`);
        }
        return {
          key,
          label,
          type,
          required: required.toLowerCase() === "required" || required.toLowerCase() === "true",
          options: options.split(",").map((value) => value.trim()).filter(Boolean),
        };
      });
  }

  async function saveTemplate() {
    if (!canEdit || saving) return;
    saving = true;
    error = null;
    try {
      const fields = parseFields();
      const input = {
        manifest: {
          id: id.trim(),
          name: name.trim(),
          category: category.trim(),
          scope: scope.trim(),
          document_type: documentType.trim(),
          subtype: subtype.trim() || null,
          version: 1,
          description: description.trim() || null,
          overview_fields: overviewFields.split(",").map((value) => value.trim()).filter(Boolean),
          image_field: imageField.trim(),
          fields,
        },
        body,
        relative_path: relativePath.trim(),
        previous_relative_path: previousRelativePath,
      };
      registry = await invoke<Registry>("save_user_entity_template", { input });
      onRegistryChanged?.(registry);
      const saved = registry.templates.find((template) => template.id === input.manifest.id);
      if (saved) selectTemplate(saved);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function deleteTemplate() {
    if (editingSource !== "user" || !previousRelativePath || saving) return;
    if (!window.confirm(`Delete user template "${name}"?`)) return;
    saving = true;
    error = null;
    try {
      registry = await invoke<Registry>("delete_user_entity_template", {
        relativePath: previousRelativePath,
      });
      onRegistryChanged?.(registry);
      newTemplate();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="template-manager__scrim" role="presentation">
  <div class="template-manager" role="dialog" aria-modal="true" aria-label="Entity templates">
    <header class="template-manager__header">
      <div>
        <p class="eyebrow">Story Development</p>
        <h2>Entity Templates</h2>
      </div>
      <div class="template-manager__header-actions">
        <button type="button" aria-label="Refresh templates" onclick={loadRegistry} disabled={loading}>
          <RefreshCw size={15} aria-hidden="true" />
        </button>
        <button type="button" aria-label="Close entity templates" onclick={onClose}>
          <X size={16} aria-hidden="true" />
        </button>
      </div>
    </header>

    <div class="template-manager__layout">
      <aside class="template-manager__list">
        <button class="new-template" type="button" onclick={newTemplate}>
          <Plus size={14} aria-hidden="true" /> New user template
        </button>
        {#each registry.templates as template (template.id)}
          <button
            type="button"
            class:active={selectedId === template.id}
            class="template-row"
            onclick={() => selectTemplate(template)}
          >
            <span>{template.name}</span>
            <small>{template.source === "user" ? "User" : "Built-in"} · {template.category}</small>
          </button>
        {/each}
        {#if registry.issues.length}
          <div class="template-issues">
            <strong>Template issues</strong>
            {#each registry.issues as issue}
              <p><span>{issue.relative_path}</span>{issue.message}</p>
            {/each}
          </div>
        {/if}
      </aside>

      <main class="template-manager__editor">
        {#if editingSource === "built-in"}
          <div class="read-only-note">Built-in templates are read-only. Create a user template to customize your own scaffold.</div>
        {/if}

        <div class="form-grid">
          <label>ID<input bind:value={id} disabled={!canEdit} placeholder="custom-character" /></label>
          <label>Name<input bind:value={name} disabled={!canEdit} placeholder="Custom Character" /></label>
          <label>Category<input bind:value={category} disabled={!canEdit} /></label>
          <label>Template scope<select bind:value={scope} disabled={!canEdit} aria-label="Template scope"><option value="story-development">World Builder (legacy-compatible)</option><option value="worldbuilding">Worldbuilding</option><option value="campaign">Campaign</option><option value="adventure">Adventure</option><option value="encounter">Encounter</option><option value="session">Session</option><option value="gm">Game Master</option><option value="project">Legacy project</option><option value="outline">Legacy outline</option><option value="manuscript">Legacy manuscript</option><option value="review">Legacy review</option><option value="publishing">Legacy publishing</option></select></label>
          <label>Document type<input bind:value={documentType} disabled={!canEdit} /></label>
          <label>Subtype<input bind:value={subtype} disabled={!canEdit} placeholder="optional" /></label>
          <label>Image field<input bind:value={imageField} disabled={!canEdit} /></label>
          <label class="wide">Description<input bind:value={description} disabled={!canEdit} /></label>
          <label class="wide">Template file<input bind:value={relativePath} disabled={!canEdit} placeholder="Characters/My Template.md" /></label>
          <label class="wide">Overview fields<input bind:value={overviewFields} disabled={!canEdit} placeholder="subtitle, role, age, tags" /></label>
          <label class="wide">
            Fields
            <small>One per line: key | label | type | required | option1,option2</small>
            <textarea bind:value={fieldsText} disabled={!canEdit} rows="8"></textarea>
          </label>
          <label class="wide">
            Markdown body
            <textarea bind:value={body} disabled={!canEdit} rows="14"></textarea>
          </label>
        </div>

        {#if error}<div class="template-error" role="alert">{error}</div>{/if}

        <footer>
          <span class="template-path">{registry.user_templates_dir}</span>
          <div>
            {#if editingSource === "user"}
              <button class="danger" type="button" onclick={deleteTemplate} disabled={saving}>
                <Trash2 size={14} aria-hidden="true" /> Delete
              </button>
            {/if}
            {#if canEdit}
              <button class="primary" type="button" onclick={saveTemplate} disabled={saving}>
                <Save size={14} aria-hidden="true" /> {saving ? "Saving…" : "Save template"}
              </button>
            {/if}
          </div>
        </footer>
      </main>
    </div>
  </div>
</div>

<style>
  .template-manager__scrim { position:fixed; inset:0; z-index:1000; display:grid; place-items:center; padding:28px; background:rgb(0 0 0 / .62); }
  .template-manager { width:min(1120px,96vw); height:min(820px,92vh); display:flex; flex-direction:column; overflow:hidden; border:1px solid var(--color-border); border-radius:10px; background:var(--color-bg); box-shadow:0 24px 70px rgb(0 0 0 / .45); }
  .template-manager__header { min-height:62px; display:flex; align-items:center; justify-content:space-between; padding:10px 14px; border-bottom:1px solid var(--color-border); background:var(--color-surface); }
  .template-manager__header h2 { margin:0; color:var(--color-text); font:600 20px/1.2 var(--font-display); }
  .template-manager__header-actions { display:flex; gap:6px; }
  .template-manager__header-actions button { width:32px; height:32px; display:grid; place-items:center; border:1px solid var(--color-border); border-radius:5px; background:var(--color-surface-raised); color:var(--color-text-muted); cursor:pointer; }
  .template-manager__layout { flex:1; min-height:0; display:grid; grid-template-columns:270px minmax(0,1fr); }
  .template-manager__list { min-height:0; overflow:auto; padding:10px; border-right:1px solid var(--color-border); background:var(--color-surface); }
  .new-template,.template-row { width:100%; border:0; border-radius:5px; text-align:left; cursor:pointer; }
  .new-template { display:flex; align-items:center; gap:6px; margin-bottom:8px; padding:8px 9px; background:var(--color-surface-raised); color:var(--color-accent-text); }
  .template-row { display:grid; gap:2px; padding:8px 9px; background:transparent; color:var(--color-text); }
  .template-row:hover,.template-row.active { background:var(--color-surface-raised); }
  .template-row small { color:var(--color-text-muted); }
  .template-issues { margin-top:12px; padding-top:10px; border-top:1px solid var(--color-border); color:var(--color-error); font-size:10px; }
  .template-issues p { display:grid; gap:2px; }
  .template-manager__editor { min-height:0; display:flex; flex-direction:column; overflow:auto; padding:18px; }
  .read-only-note { margin-bottom:14px; padding:9px 10px; border:1px solid var(--color-border); border-radius:6px; color:var(--color-text-muted); background:var(--color-surface); font-size:11px; }
  .form-grid { display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:12px; }
  .form-grid label { display:grid; gap:5px; color:var(--color-text-muted); font-size:11px; font-weight:600; }
  .form-grid label.wide { grid-column:1 / -1; }
  .form-grid label small { font-weight:400; }
  .form-grid input,.form-grid textarea { width:100%; box-sizing:border-box; padding:8px 9px; border:1px solid var(--color-border); border-radius:5px; background:var(--color-surface-sunken); color:var(--color-text); font:12px/1.45 var(--font-ui); }
  .form-grid textarea { resize:vertical; font-family:ui-monospace,SFMono-Regular,Consolas,monospace; }
  .template-error { margin-top:12px; padding:9px; border:1px solid var(--color-error); border-radius:6px; color:var(--color-error); font-size:11px; }
  footer { display:flex; align-items:center; justify-content:space-between; gap:12px; margin-top:16px; padding-top:12px; border-top:1px solid var(--color-border); }
  footer > div { display:flex; gap:8px; }
  footer button { display:inline-flex; align-items:center; gap:6px; padding:7px 10px; border:1px solid var(--color-border); border-radius:5px; cursor:pointer; }
  footer .primary { background:var(--color-accent-text); color:var(--color-bg); }
  footer .danger { background:transparent; color:var(--color-error); }
  .template-path { min-width:0; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color:var(--color-text-muted); font-size:10px; }
  .eyebrow { margin:0 0 3px; color:var(--color-accent-text); font-size:10px; font-weight:600; letter-spacing:.08em; text-transform:uppercase; }
</style>
