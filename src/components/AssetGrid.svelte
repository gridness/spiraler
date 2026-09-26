<script lang="ts">
  import type { Asset, Job } from '../lib/types';
  import { preferred, assetStatus, selectRange } from '../lib/assets';
  import { imageUrl } from '../lib/studio.svelte';
  import Icon from './Icon.svelte';
  let {
    assets,
    jobs,
    selected = $bindable(),
    density,
    onopen,
    onreorder,
  }: {
    assets: Asset[];
    jobs: Job[];
    selected: string[];
    density: number;
    onopen: () => void;
    onreorder: (source: string, target: string) => void;
  } = $props();
  let anchor = $state('');
  let dragged = $state('');
  let dropTarget = $state('');
  function select(id: string, event: MouseEvent | KeyboardEvent) {
    if (event.shiftKey && anchor)
      selected = selectRange(
        assets.map((a) => a.id),
        anchor,
        id,
      );
    else if (event.metaKey || event.ctrlKey) {
      selected = selected.includes(id) ? selected.filter((i) => i !== id) : [...selected, id];
      anchor = id;
    } else {
      selected = [id];
      anchor = id;
    }
  }
  function key(event: KeyboardEvent, index: number) {
    const target = event.currentTarget as HTMLElement;
    const cols = Math.max(
      1,
      Math.floor((target.parentElement?.clientWidth ?? density) / (density + 16)),
    );
    const delta: Record<string, number> = {
      ArrowRight: 1,
      ArrowLeft: -1,
      ArrowDown: cols,
      ArrowUp: -cols,
    };
    if (event.key in delta) {
      event.preventDefault();
      const next = Math.max(0, Math.min(assets.length - 1, index + delta[event.key]));
      select(assets[next].id, event);
      (target.parentElement?.children[next] as HTMLElement)?.focus();
    }
  }
</script>

<div class="asset-grid" style={`--tile-min: ${density}px`}>
  {#each assets as asset, index (asset.id)}
    {@const result = preferred(asset)}
    {@const status = assetStatus(asset, jobs)}
    <button
      class="asset-tile"
      class:selected={selected.includes(asset.id)}
      class:drop-target={dropTarget === asset.id}
      aria-pressed={selected.includes(asset.id)}
      aria-label={`${asset.name}, ${status}${asset.approved ? ', approved' : ''}`}
      tabindex={selected.length === 0
        ? index === 0
          ? 0
          : -1
        : selected.at(-1) === asset.id
          ? 0
          : -1}
      onclick={(e) => select(asset.id, e)}
      ondblclick={onopen}
      onkeydown={(e) => key(e, index)}
      draggable="true"
      ondragstart={(e) => {
        dragged = asset.id;
        e.dataTransfer?.setData('text/spiraler-asset', asset.id);
      }}
      ondragend={() => {
        dragged = '';
        dropTarget = '';
      }}
      ondragover={(e) => {
        if (dragged && dragged !== asset.id) {
          e.preventDefault();
          dropTarget = asset.id;
        }
      }}
      ondragleave={() => (dropTarget = '')}
      ondrop={(e) => {
        e.preventDefault();
        if (dragged && dragged !== asset.id) onreorder(dragged, asset.id);
        dragged = '';
        dropTarget = '';
      }}
    >
      <div class="asset-art" class:checker={!!result}>
        {#if result}<img
            src={imageUrl(result.image)}
            alt={asset.name}
            loading="lazy"
            decoding="async"
          />
        {:else}<div class="draft-art">
            <span class="draft-number">{String(index + 1).padStart(2, '0')}</span><Icon
              name={status === 'failed' ? 'info' : 'image'}
              size={28}
            /><span
              >{status === 'generating'
                ? 'Creating your asset'
                : 'Waiting for its first image'}</span
            >
          </div>{/if}
        <span class="selection-check" class:checked={selected.includes(asset.id)}
          ><Icon name="check" size={12} /></span
        >
        {#if asset.approved}<span class="approval-mark" title="Approved"
            ><Icon name="check" size={13} /></span
          >{/if}
        {#if status === 'generating' || status === 'queued'}<span class="tile-state"
            ><span class:spinner={status === 'generating'} class="status-dot"></span>{status ===
            'generating'
              ? 'Generating'
              : 'Queued'}</span
          >{/if}
        {#if result?.kind === 'upscale'}<span class="tile-kind">2×</span>{/if}
      </div>
      <div class="asset-caption">
        <span class="asset-name">{asset.name}</span><span
          class="asset-meta"
          class:failed={status === 'failed'}
          >{status === 'failed'
            ? 'Needs attention'
            : result
              ? `${asset.results.length} ${asset.results.length === 1 ? 'result' : 'results'}`
              : 'Draft'}</span
        >
      </div>
    </button>
  {/each}
</div>
