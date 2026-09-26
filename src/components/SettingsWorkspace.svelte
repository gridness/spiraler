<script lang="ts">
  import { studio, connect, checkConnection, installationHelp, reveal } from '../lib/studio.svelte';
  import Icon from './Icon.svelte';
</script>

<div class="scroll-workspace settings-workspace">
  <div class="workspace-heading">
    <div>
      <span class="eyebrow">YOUR STUDIO</span>
      <h1>Connection & local files</h1>
      <p>A subscription connection, with your creative work saved on this computer.</p>
    </div>
  </div>
  <section class="settings-section">
    <div class="section-heading">
      <h2>ChatGPT through Codex</h2>
      <span class="connection-state" class:connected={studio.provider?.authenticated}
        ><span class="status-dot"></span>{studio.provider?.authenticated
          ? 'Connected'
          : 'Connection needed'}</span
      >
    </div>
    <p>{studio.provider?.message ?? 'Checking your connection…'}</p>
    <div class="button-row">
      {#if !studio.provider?.installed}<button class="primary" onclick={installationHelp}
          >Install Codex</button
        >{:else}<button class="primary" disabled={studio.signingIn} onclick={connect}
          >{studio.signingIn
            ? 'Complete sign-in in your browser…'
            : studio.provider?.authenticated
              ? 'Reconnect account'
              : 'Connect ChatGPT'}</button
        >{/if}<button onclick={checkConnection}>Check connection</button>
    </div>
    <p class="small muted">{studio.provider?.version ?? 'Official Codex CLI required'}</p>
    <details>
      <summary>How subscription access works</summary>
      <p>
        OpenAI does not document a general image-generation endpoint that third-party apps can bill
        to a ChatGPT subscription. Spiraler uses the official Codex CLI as the supported
        subscription fallback.
      </p>
      <p>
        Codex manages sign-in and credentials. Spiraler does not read browser sessions or
        authentication files. New sign-ins request the OS credential store.
      </p>
      <p>
        Image generation draws from Codex subscription limits. Spiraler runs one job at a time.
        Availability and limits depend on your plan and workspace.
      </p>
    </details>
  </section>
  <section class="settings-section">
    <h2>Local project library</h2>
    <p>
      Projects contain a readable manifest, original images, thumbnails, and job history. Saves are
      atomic, with a previous manifest kept as a backup.
    </p>
    <p class="path selectable">{studio.library?.root}</p>
    {#if studio.project}<button onclick={() => reveal()}
        ><Icon name="folder" />Reveal current project</button
      >{/if}
    <p class="small muted">
      Back up this folder with your other creative work. Closing Spiraler stops generation;
      unfinished jobs can be retried after reopening.
    </p>
  </section>
  <section class="settings-section">
    <h2>Upscaling</h2>
    <p>
      Spiraler can enlarge images 2× locally with Lanczos interpolation. This preserves the original
      and creates a separate result. It does not recover new detail or promise lossless AI
      enhancement.
    </p>
  </section>
  <section class="settings-section">
    <h2>Keyboard shortcuts</h2>
    <dl class="shortcut-list">
      <dt>Add assets</dt>
      <dd>⌘ / Ctrl + N</dd>
      <dt>Select visible assets</dt>
      <dd>⌘ / Ctrl + A</dd>
      <dt>Generate selection</dt>
      <dd>⌘ / Ctrl + Enter</dd>
      <dt>Find assets</dt>
      <dd>⌘ / Ctrl + F</dd>
      <dt>Toggle inspector</dt>
      <dd>⌘ / Ctrl + I</dd>
      <dt>Move through the grid</dt>
      <dd>Arrow keys</dd>
      <dt>Select a range</dt>
      <dd>Shift + click</dd>
    </dl>
  </section>
</div>
