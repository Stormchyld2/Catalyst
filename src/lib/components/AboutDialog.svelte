<script lang="ts">
  import lithomarkLogo from "../../assets/lithomark-logo.png";
  import { getVersion } from "@tauri-apps/api/app";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { ChevronRight, ExternalLink, Send } from "lucide-svelte";
  import DialogHeader from "./DialogHeader.svelte";
  import { onMount } from "svelte";
  import { modalFocus } from "../utils/modalFocus";

  let { onClose, onSendFeedback }: { onClose: () => void; onSendFeedback: () => void } = $props();

  let version = $state("...");

  onMount(async () => {
    try {
      version = await getVersion();
    } catch {
      version = "unknown";
    }
  });

  let linkError = $state<string | null>(null);

  async function openLink(url: string) {
    linkError = null;
    try {
      await openUrl(url);
    } catch (error) {
      linkError = `Could not open the link: ${error}`;
    }
  }
</script>

<div
  class="dialog-scrim"
  use:modalFocus={{ onEscape: onClose }}
  role="dialog"
  aria-modal="true"
  aria-labelledby="about-title"
  tabindex="-1"
>
  <div class="app-dialog-surface ka-dialog-narrow dialog-shell" data-testid="about-dialog">
    <DialogHeader
      title="About LithoMark"
      titleId="about-title"
      {onClose}
      closeLabel="Close"
      closeTestId="about-close"
    />

    <div class="ka-dialog-body about">
      <div class="about-identity">
        <img class="about-logo" src={lithomarkLogo} alt="LithoMark Writer OS" />
        <p class="about-tagline">Spark your draft &mdash; bridge the gap between outline and prose.</p>
      </div>

      <dl class="ka-facts">
        <div>
          <dt>Version</dt>
          <dd>{version}</dd>
        </div>
      </dl>

      <ul class="about-links">
        <li>
          <button type="button" onclick={onSendFeedback} class="about-link">
            <Send class="w-5 h-5" aria-hidden="true" />
            <span>Send feedback</span>
            <ChevronRight class="w-5 h-5" aria-hidden="true" />
          </button>
        </li>
        <li>
          <button
            type="button"
            onclick={() => openLink("https://github.com/Stormchyld2/LithoMark")}
            class="about-link"
          >
            <ExternalLink class="w-5 h-5" aria-hidden="true" />
            <span>GitHub repository</span>
          </button>
        </li>
        <li>
          <button
            type="button"
            onclick={() => openLink("https://github.com/Stormchyld2/LithoMark/issues/new")}
            class="about-link"
          >
            <ExternalLink class="w-5 h-5" aria-hidden="true" />
            <span>Report an issue</span>
          </button>
        </li>
        <li>
          <button
            type="button"
            onclick={() => openLink("https://github.com/Stormchyld2/LithoMark/releases")}
            class="about-link"
          >
            <ExternalLink class="w-5 h-5" aria-hidden="true" />
            <span>Release notes</span>
          </button>
        </li>
      </ul>
      {#if linkError}
        <p class="ka-error" role="alert">{linkError}</p>
      {/if}

      <p class="ka-help">&copy; 2026 LithoMark</p>
    </div>
  </div>
</div>

<style>
  .about {
    display: grid;
    gap: var(--space-s);
  }
  .about-identity {
    display: grid;
    justify-items: start;
    gap: var(--space-xs);
  }
  .about-logo {
    display: block;
    width: min(280px, 80%);
    height: auto;
  }
  .about-tagline {
    margin: 0;
    max-width: var(--measure);
    font: italic var(--text-body) / var(--leading-relaxed) var(--font-body);
    color: var(--color-text);
  }
  .about-links {
    display: grid;
    margin: 0;
    padding: 0;
    list-style: none;
    border-top: var(--border-hair);
  }
  .about-links li {
    border-bottom: var(--border-hair);
  }
  .about-link {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    width: 100%;
    min-height: var(--control-target);
    padding: var(--space-2xs) var(--space-3xs);
    border: 0;
    border-radius: var(--radius-xs);
    background: transparent;
    color: var(--color-text);
    font: var(--text-ui) / 1.5 var(--font-ui);
    text-align: left;
    cursor: pointer;
  }
  .about-link span {
    flex: 1;
  }
  .about-link :global(svg) {
    color: var(--color-text-muted);
  }
  @media (hover: hover) {
    .about-link:hover {
      background: var(--color-surface-sunken);
    }
  }
</style>
