<script lang="ts">
  import { untrack, onDestroy } from 'svelte';
  import type { Asset, Project } from '../lib/types';
  import { preferred, assetStatus } from '../lib/assets';
  import { act, imageUrl, importImages, reveal, studio, notify } from '../lib/studio.svelte';
  import Icon from './Icon.svelte';
  let {
    asset,
    project,
    onclose,
    onupscale,
    ondelete,
    onexport,
  }: {
    asset: Asset;
    project: Project;
    onclose: () => void;
    onupscale: () => void;
    ondelete: () => void;
    onexport: () => void;
  } = $props();
  let name = $state(untrack(() => asset.name));
  let subject = $state(untrack(() => asset.subject));
  let compare = $state(false);
  $effect(() => {
    studio.unsaved = name !== asset.name || subject !== asset.subject;
  });
  onDestroy(() => {
    studio.unsaved = false;
  });
  const result = $derived(preferred(asset));
  const status = $derived(assetStatus(asset, project.jobs));
  const active = $derived(status === 'queued' || status === 'generating');
  const latestJob = $derived([...project.jobs].reverse().find((j) => j.assetId === asset.id));
  async function save() {
    const p = await act({ type: 'editAsset', assetId: asset.id, name, subject });
    if (p) notify('Asset details saved.');
  }
</script>

<aside class="inspector" aria-label="Asset inspector">
  <div class="inspector-heading">
    <span>INSPECTOR</span><button class="icon-button" onclick={onclose} aria-label="Close inspector"
      ><Icon name="close" size={16} /></button
    >
  </div>
  <div class="inspector-body">
    <div class="inspector-preview checker">
      {#if result}<img src={imageUrl(result.image, false)} alt={asset.name} />{:else}<Icon
          name="image"
          size={40}
        />{/if}
    </div>
    <div class="result-caption">
      <span>{result ? `${result.image.width} × ${result.image.height}` : 'No image yet'}</span><span
        >{result?.kind === 'upscale'
          ? '2× local enlargement'
          : result?.kind === 'variant'
            ? 'Variation'
            : 'Original'}</span
      >
    </div>
    {#if result}<button
        class:approved={asset.approved}
        class="approve-button"
        onclick={() => act({ type: 'approve', ids: [asset.id], approved: !asset.approved })}
        ><Icon name="check" size={16} />{asset.approved
          ? 'Approved for export'
          : 'Approve this result'}</button
      >{/if}
    <form
      onsubmit={(e) => {
        e.preventDefault();
        void save();
      }}
      onchange={() => {
        if (name !== asset.name || subject !== asset.subject) void save();
      }}
    >
      <label class="field">Name<input bind:value={name} maxlength="160" required /></label>
      <label class="field"
        >What should I make?<textarea bind:value={subject} rows="4" maxlength="8000" required
        ></textarea></label
      >
      {#if name !== asset.name || subject !== asset.subject}<button
          class="subtle full"
          type="submit"
          disabled={studio.pending > 0}>Save changes</button
        >{/if}
    </form>
    {#if latestJob?.error && latestJob.status === 'failed'}<div class="inline-error">
        <strong>This asset needs attention</strong>
        <p>{latestJob.error}</p>
        <button onclick={() => act({ type: 'retry', ids: [latestJob.id] })}>Retry job</button>
      </div>{/if}
    <div class="inspector-actions">
      <button
        class="primary"
        disabled={active}
        onclick={() => act({ type: 'enqueue', ids: [asset.id], kind: 'generate' })}
        ><Icon name={result ? 'retry' : 'sparkle'} size={15} />{active
          ? 'In the queue'
          : result
            ? 'Regenerate'
            : 'Generate'}</button
      >{#if result}<button
          disabled={active}
          onclick={() => act({ type: 'enqueue', ids: [asset.id], kind: 'variant' })}
          ><Icon name="copy" size={15} />Variation</button
        >{/if}
    </div>
    {#if result}
      <div class="section-heading compact">
        <h2>Results <span>{asset.results.length}</span></h2>
        {#if asset.results.length > 1}<button
            class="text-button"
            onclick={() => (compare = !compare)}>{compare ? 'Close comparison' : 'Compare'}</button
          >{/if}
      </div>
      <div class="variants" class:compare>
        {#each asset.results as variant, i (variant.id)}<button
            class:chosen={result.id === variant.id}
            onclick={() => act({ type: 'prefer', assetId: asset.id, resultId: variant.id })}
            aria-label={`Use result ${i + 1}${variant.kind === 'upscale' ? ', enlarged' : ''}`}
            aria-pressed={result.id === variant.id}
            ><img src={imageUrl(variant.image)} alt={`Result ${i + 1}`} loading="lazy" /><span
              >{i + 1}{variant.kind === 'upscale' ? ' · 2×' : ''}</span
            ></button
          >{/each}
      </div>
      <div class="file-actions">
        <button onclick={onupscale} disabled={active}
          ><Icon name="upscale" />Upscale <span>2× local</span></button
        >
        <button
          onclick={() => act({ type: 'useReference', assetId: asset.id, resultId: result.id })}
          disabled={project.style.references.some((r) => r.id === result.image.id)}
          ><Icon name="style" />Use as style reference</button
        >
        <button
          disabled={active}
          onclick={() => act({ type: 'enqueue', ids: [asset.id], kind: 'variant' })}
          ><Icon name="sparkle" />Bring closer to project style</button
        >
        <button onclick={() => reveal(result.image.id)}><Icon name="folder" />Reveal image</button>
        <button onclick={onexport}><Icon name="export" />Export asset</button>
      </div>
      <details class="generation-details">
        <summary>Generation details</summary>
        <dl>
          <dt>Style</dt>
          <dd>Version {result.request.style.version}</dd>
          <dt>Provider</dt>
          <dd>{result.provider}</dd>
          <dt>Created</dt>
          <dd>{new Date(result.createdAt * 1000).toLocaleString()}</dd>
          <dt>References</dt>
          <dd>{result.request.style.references.length}</dd>
        </dl>
        <label class="field"
          >Effective prompt<textarea readonly value={result.request.prompt} rows="8"
          ></textarea></label
        >
      </details>
    {/if}
    <div class="file-actions">
      <button onclick={() => importImages(asset.id)}
        ><Icon name="image" />Import image as result</button
      ><button onclick={() => act({ type: 'duplicate', ids: [asset.id] })}
        ><Icon name="copy" />Duplicate asset</button
      ><button class="danger-text" onclick={ondelete}><Icon name="trash" />Delete asset</button>
    </div>
  </div>
</aside>
