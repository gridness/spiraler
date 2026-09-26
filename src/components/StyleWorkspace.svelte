<script lang="ts">
  import { untrack, onDestroy } from 'svelte';
  import type { Project, Style } from '../lib/types';
  import { act, imageUrl, importImages, studio } from '../lib/studio.svelte';
  import Icon from './Icon.svelte';
  let { project }: { project: Project } = $props();
  let draft = $state<Style>(untrack(() => structuredClone($state.snapshot(project.style))));
  let savedVersion = $state(untrack(() => project.style.version));
  let dirty = $state(false);
  let saving = false;
  let saveAgain = false;
  $effect(() => {
    studio.unsaved = dirty;
  });
  onDestroy(() => {
    studio.unsaved = false;
  });
  $effect(() => {
    if (project.style.version !== savedVersion) {
      if (!dirty) draft = structuredClone($state.snapshot(project.style));
      else {
        draft.references = project.style.references;
        draft.version = project.style.version;
      }
      savedVersion = project.style.version;
    }
  });
  const signature = (style: Style) => JSON.stringify({ ...style, version: 0, references: [] });
  async function save() {
    if (saving) {
      saveAgain = true;
      return;
    }
    saving = true;
    const submitted = $state.snapshot(draft);
    const p = await act({ type: 'saveStyle', style: submitted, expectedVersion: savedVersion });
    if (p) {
      savedVersion = p.style.version;
      if (signature(draft) === signature(submitted)) {
        dirty = false;
        draft = structuredClone($state.snapshot(p.style));
      } else {
        draft.version = p.style.version;
        draft.references = p.style.references;
      }
    }
    saving = false;
    if (p && dirty && saveAgain) {
      saveAgain = false;
      void save();
    }
  }
</script>

<div class="scroll-workspace style-workspace">
  <div class="workspace-heading">
    <div>
      <span class="eyebrow">THE FAMILY RESEMBLANCE</span>
      <h1>Your style, carried through.</h1>
      <p>Set the visual direction once. Every asset starts here.</p>
    </div>
    <button class="primary" disabled={!dirty || studio.pending > 0} onclick={save}
      >{dirty ? 'Save style' : `Style saved · v${project.style.version}`}</button
    >
  </div>
  <section class="style-references">
    <div class="section-heading">
      <h2>Reference board</h2>
      <span>{project.style.references.length} / 5 images</span>
    </div>
    <p class="muted small">
      Add a few images that share the look you want. Drop images anywhere in this workspace.
    </p>
    <div class="reference-board">
      {#each project.style.references as reference (reference.id)}
        <div class="reference">
          <img src={imageUrl(reference)} alt={reference.name} loading="lazy" />
          <div>
            <span>{reference.name}</span><button
              class="icon-button"
              title="Remove from references"
              aria-label={`Remove ${reference.name} from references`}
              onclick={() => act({ type: 'removeReference', referenceId: reference.id })}
              ><Icon name="close" size={14} /></button
            >
          </div>
        </div>
      {/each}
      {#if project.style.references.length < 5}<button
          class="reference-add"
          onclick={() => importImages()}
          ><Icon name="plus" size={24} /><span>Add references</span><small>PNG, JPEG or WebP</small
          ></button
        >{/if}
    </div>
  </section>
  <form
    onsubmit={(e) => {
      e.preventDefault();
      void save();
    }}
    oninput={() => (dirty = true)}
    onchange={() => {
      if (dirty) void save();
    }}
  >
    <div class="section-heading">
      <h2>Art direction</h2>
      <span>Version {project.style.version}{dirty ? ' · Unsaved changes' : ''}</span>
    </div>
    <label class="field"
      >Describe the look<textarea
        bind:value={draft.direction}
        rows="4"
        maxlength="8000"
        placeholder="Handcrafted woodland objects, soft clay shapes, a little imperfect. Warm and inviting, like props from a stop-motion film."
      ></textarea></label
    >
    <div class="field-grid">
      <label class="field"
        >Palette<input
          bind:value={draft.palette}
          maxlength="8000"
          placeholder="Moss green, burnt orange, warm ivory"
        /></label
      >
      <label class="field"
        >Materials & texture<input
          bind:value={draft.material}
          maxlength="8000"
          placeholder="Matte clay, gently worn edges"
        /></label
      >
      <label class="field"
        >Lighting<input
          bind:value={draft.lighting}
          maxlength="8000"
          placeholder="Soft window light from the upper left"
        /></label
      >
      <label class="field"
        >Perspective & lines<input
          bind:value={draft.perspective}
          maxlength="8000"
          placeholder="Three-quarter view, rounded outlines"
        /></label
      >
    </div>
    <details class="style-details" open>
      <summary>Framing & finishing</summary>
      <div class="field-grid">
        <label class="field"
          >Composition<textarea bind:value={draft.composition} maxlength="8000" rows="2"
          ></textarea></label
        >
        <label class="field"
          >Keep out<textarea bind:value={draft.negative} maxlength="8000" rows="2"
          ></textarea></label
        >
        <label class="field"
          >Background<input bind:value={draft.background} maxlength="8000" /><small
            >Transparency is a request; check the returned image before export.</small
          ></label
        >
        <label class="field"
          >Image shape<select bind:value={draft.aspect}
            ><option value="square">Square</option><option value="portrait">Portrait</option><option
              value="landscape">Landscape</option
            ></select
          ><small>Codex chooses the supported output resolution.</small></label
        >
      </div>
    </details>
    <p class="footnote">
      <Icon name="info" size={15} /> Style changes apply to future jobs. Existing results and queued requests
      keep their original style version.
    </p>
  </form>
</div>
