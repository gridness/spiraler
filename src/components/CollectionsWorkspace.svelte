<script lang="ts">
  import type { Library, ProjectSummary } from '../lib/types';
  import { restoreProject, studio } from '../lib/studio.svelte';
  import Icon from './Icon.svelte';
  let {
    library,
    onopen,
    oncreate,
    ondelete,
  }: {
    library: Library;
    onopen: (id: string) => void;
    oncreate: () => void;
    ondelete: (project: ProjectSummary) => void;
  } = $props();
</script>

<main class="scroll-workspace collections-workspace">
  <div class="workspace-heading">
    <div>
      <span class="eyebrow">YOUR STUDIO</span>
      <h1>Collections</h1>
      <p>Each collection has its own assets and shared style.</p>
    </div>
    <button class="primary" onclick={oncreate}><Icon name="plus" size={16} />New collection</button>
  </div>
  {#if library.projects.length}
    <div class="collections-grid">
      {#each library.projects as collection (collection.id)}
        <article class="collection-card">
          <button
            class="collection-open"
            onclick={() => onopen(collection.id)}
            aria-label={`Open ${collection.name}`}
          >
            <span class="collection-monogram" aria-hidden="true"
              >{collection.name.slice(0, 1).toUpperCase()}</span
            >
            <strong>{collection.name}</strong><span>{collection.assetType}</span>
            <small>{collection.count} {collection.count === 1 ? 'asset' : 'assets'}</small>
          </button>
          <button
            class="icon-button collection-delete"
            aria-label={`Delete ${collection.name}`}
            title="Delete collection"
            onclick={() => ondelete(collection)}><Icon name="trash" size={16} /></button
          >
        </article>
      {/each}
    </div>
  {:else}
    <div class="empty-workspace">
      <Icon name="grid" size={36} />
      <h2>No collections yet</h2>
      <p>Create a collection to start defining its style and assets.</p>
      <button onclick={oncreate}>Create collection</button>
    </div>
  {/if}
  {#if library.deletedProjects.length}
    <details class="deleted-collections">
      <summary>Deleted collections ({library.deletedProjects.length})</summary>
      <p class="muted small">
        Assets and image files are kept on this computer so you can restore a collection.
      </p>
      {#each library.deletedProjects as collection (collection.id)}
        <div class="deleted-collection">
          <span><strong>{collection.name}</strong><small>{collection.count} assets</small></span
          ><button
            disabled={studio.pending > 0}
            onclick={() => restoreProject(collection.id)}
            aria-label={`Restore ${collection.name}`}>Restore</button
          >
        </div>
      {/each}
    </details>
  {/if}
</main>
