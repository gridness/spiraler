<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import type { Mode, Filter, JobKind, ProjectSummary } from './lib/types';
  import {
    studio,
    initialize,
    openProject,
    createProject,
    deleteProject,
    act,
    notify,
    importImages,
    importAssetList,
    refreshLibrary,
    attempt,
  } from './lib/studio.svelte';
  import { filterAssets, parseAssetList, preferred } from './lib/assets';
  import Icon from './components/Icon.svelte';
  import Logo from './components/Logo.svelte';
  import Dialog from './components/Dialog.svelte';
  import AssetGrid from './components/AssetGrid.svelte';
  import Inspector from './components/Inspector.svelte';
  import StyleWorkspace from './components/StyleWorkspace.svelte';
  import QueueWorkspace from './components/QueueWorkspace.svelte';
  import ExportWorkspace from './components/ExportWorkspace.svelte';
  import SettingsWorkspace from './components/SettingsWorkspace.svelte';
  import CollectionsWorkspace from './components/CollectionsWorkspace.svelte';

  let mode = $state<Mode>('assets');
  let selected = $state<string[]>([]);
  let search = $state('');
  let filter = $state<Filter>('all');
  let inspector = $state(true);
  let density = $state(190);
  let projectsOpen = $state(false);
  let dragging = $state(false);
  let modal = $state<'new' | 'bulk' | 'delete' | 'deleteCollection' | 'upscale' | 'batch' | null>(
    null,
  );
  let collectionToDelete = $state<ProjectSummary | null>(null);
  let modalIds = $state<string[]>([]);
  let projectName = $state('');
  let projectDescription = $state('');
  let assetType = $state('Game assets');
  let bulkText = $state('');
  let bulkError = $state('');
  let batchInstructions = $state('');
  let localBusy = $state(false);
  let searchInput = $state<HTMLInputElement>();
  const project = $derived(studio.project);
  const visible = $derived(
    project ? filterAssets(project.assets, project.jobs, search, filter) : [],
  );
  const selectedAsset = $derived(
    selected.length === 1 ? project?.assets.find((a) => a.id === selected[0]) : undefined,
  );
  const canInspect = $derived(mode === 'assets' && !!selectedAsset);
  const activeJobs = $derived(
    project?.jobs.filter((j) => j.status === 'queued' || j.status === 'generating') ?? [],
  );
  const readyCount = $derived(project?.assets.filter((a) => preferred(a)).length ?? 0);
  const approvedCount = $derived(project?.assets.filter((a) => a.approved).length ?? 0);
  const selectedReady = $derived(
    project?.assets.filter((a) => selected.includes(a.id) && preferred(a)).map((a) => a.id) ?? [],
  );
  const bulkCount = $derived.by(() => {
    try {
      return parseAssetList(bulkText).length;
    } catch {
      return 0;
    }
  });
  const nav: { id: Mode; label: string; icon: string }[] = [
    { id: 'assets', label: 'Assets', icon: 'grid' },
    { id: 'style', label: 'Style', icon: 'style' },
    { id: 'queue', label: 'Queue', icon: 'queue' },
    { id: 'export', label: 'Export', icon: 'export' },
  ];

  onMount(() => {
    let dispose = () => {};
    let dropDispose = () => {};
    let destroyed = false;
    void initialize().then((fn) => {
      if (destroyed) fn();
      else dispose = fn;
    });
    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type === 'over' || event.payload.type === 'enter') dragging = true;
        else if (event.payload.type === 'leave') dragging = false;
        else if (event.payload.type === 'drop') {
          dragging = false;
          if (project && (mode === 'assets' || mode === 'style')) {
            if (mode === 'style') void importImages(null, event.payload.paths);
            else if (selected.length === 1) void importImages(selected[0], event.payload.paths);
            else notify('Open Style to add references, or select one asset to import results.');
          }
        }
      })
      .then((fn) => {
        if (destroyed) fn();
        else dropDispose = fn;
      })
      .catch(() => {});
    return () => {
      destroyed = true;
      dispose();
      dropDispose();
    };
  });
  $effect(() => {
    if (
      !studio.loading &&
      !project &&
      mode === 'assets' &&
      (studio.library?.projects.length || studio.library?.deletedProjects.length)
    )
      mode = 'collections';
  });
  $effect(() => {
    const current = project;
    if (current) {
      const valid = untrack(() => selected.filter((id) => current.assets.some((a) => a.id === id)));
      if (valid.length !== untrack(() => selected.length)) selected = valid;
    }
  });
  $effect(() => {
    if (studio.notice) {
      const timer = setTimeout(() => (studio.notice = ''), 6500);
      return () => clearTimeout(timer);
    }
  });

  async function switchProject(id: string) {
    projectsOpen = false;
    await openProject(id);
    selected = [];
    search = '';
    filter = 'all';
    mode = 'assets';
  }
  function showCollections() {
    projectsOpen = false;
    mode = 'collections';
    void attempt(refreshLibrary);
  }
  function showAssets() {
    projectsOpen = false;
    mode = 'assets';
    search = '';
    filter = 'all';
  }
  function showDeleteCollection(collection: ProjectSummary) {
    projectsOpen = false;
    collectionToDelete = collection;
    modal = 'deleteCollection';
  }
  async function removeCollection() {
    if (!collectionToDelete || localBusy) return;
    localBusy = true;
    const wasCurrent = collectionToDelete.id === project?.id;
    const deleted = await deleteProject(collectionToDelete.id);
    localBusy = false;
    if (deleted) {
      modal = null;
      collectionToDelete = null;
      if (wasCurrent) {
        selected = [];
        mode = 'collections';
      }
    }
  }
  async function newProject() {
    localBusy = true;
    const p = await createProject(projectName, projectDescription, assetType);
    localBusy = false;
    if (p) {
      modal = null;
      selected = [];
      search = '';
      filter = 'all';
      mode = 'style';
      projectName = '';
      projectDescription = '';
      notify('Project created. Give your collection a style, then add some assets.');
    }
  }
  async function addAssets() {
    try {
      const items = parseAssetList(bulkText);
      if (!items.length) {
        bulkError = 'Add at least one asset.';
        return;
      }
      localBusy = true;
      const p = await act({ type: 'addAssets', items });
      if (p) {
        modal = null;
        bulkText = '';
        mode = 'assets';
        filter = 'all';
        search = '';
        notify(
          `${items.length} ${items.length === 1 ? 'asset' : 'assets'} added to your collection.`,
        );
      }
    } catch (error) {
      bulkError = String(error).replace(/^Error: /, '');
    } finally {
      localBusy = false;
    }
  }
  async function generate(ids?: string[], kind: JobKind = 'generate') {
    if (!project) return;
    if (kind !== 'upscale' && studio.provider && !studio.provider.authenticated) {
      mode = 'settings';
      notify('Connect your ChatGPT account to start generating.');
      return;
    }
    const targets =
      ids ??
      (selected.length ? selected : project.assets.filter((a) => !preferred(a)).map((a) => a.id));
    if (!targets.length) {
      notify('Select assets to generate another result.');
      return;
    }
    const p = await act({ type: 'enqueue', ids: targets, kind });
    if (p)
      notify(
        `${targets.length} ${targets.length === 1 ? 'asset' : 'assets'} added to the queue.${p.paused ? ' Resume from Queue when ready.' : ''}`,
      );
  }
  function showDelete() {
    if (selected.length) {
      modalIds = [...selected];
      modal = 'delete';
    }
  }
  function showUpscale(ids = selectedReady) {
    if (ids.length) {
      modalIds = [...ids];
      modal = 'upscale';
    }
  }
  async function removeAssets() {
    if (await act({ type: 'deleteAssets', ids: modalIds })) {
      selected = [];
      modal = null;
      notify('Assets removed. Image files remain in the project folder.');
    }
  }
  function reorder(source: string, target: string) {
    if (!project) return;
    const ids = project.assets.map((a) => a.id).filter((id) => id !== source);
    ids.splice(ids.indexOf(target), 0, source);
    void act({ type: 'reorder', ids });
  }
  function moveSelected(delta: number) {
    if (!project || selected.length !== 1) return;
    const ids = project.assets.map((a) => a.id);
    const index = ids.indexOf(selected[0]);
    const next = index + delta;
    if (next >= 0 && next < ids.length) {
      [ids[index], ids[next]] = [ids[next], ids[index]];
      void act({ type: 'reorder', ids });
    }
  }
  function shortcut(event: KeyboardEvent) {
    const editing = (event.target as HTMLElement)?.closest(
      'input, textarea, select, [contenteditable="true"]',
    );
    if (modal) return;
    if (event.key === 'Escape') {
      selected = [];
      projectsOpen = false;
      return;
    }
    if (event.metaKey || event.ctrlKey) {
      if (event.key.toLowerCase() === 'n') {
        event.preventDefault();
        modal = project && mode !== 'collections' ? 'bulk' : 'new';
      } else if (event.key.toLowerCase() === 'f' && project && mode !== 'collections') {
        event.preventDefault();
        mode = 'assets';
        queueMicrotask(() => searchInput?.focus());
      } else if (event.key.toLowerCase() === 'i' && canInspect) {
        event.preventDefault();
        inspector = !inspector;
      } else if (!editing && event.key.toLowerCase() === 'a' && mode === 'assets') {
        event.preventDefault();
        selected = visible.map((a) => a.id);
      } else if (event.key === 'Enter' && mode === 'assets') {
        event.preventDefault();
        void generate();
      }
    }
    if (
      !editing &&
      mode === 'assets' &&
      (event.key === 'Delete' || event.key === 'Backspace') &&
      selected.length
    ) {
      event.preventDefault();
      showDelete();
    }
  }
</script>

<svelte:window onkeydown={shortcut} />
<div class="app-shell">
  <aside class="sidebar">
    <button
      class="brand"
      aria-label="Spiraler · All collections"
      title="All collections"
      onclick={showCollections}
      ><Logo /><span>spiraler<span class="brand-dot">.</span></span></button
    >
    <div class="project-switcher">
      <button
        class="project-current"
        onclick={() => {
          projectsOpen = !projectsOpen;
          if (projectsOpen) void attempt(refreshLibrary);
        }}
        aria-expanded={projectsOpen}
        aria-label="Switch collection"
        ><span class="project-avatar">{project?.name.slice(0, 1).toUpperCase() ?? 'S'}</span><span
          class="project-label"
          ><strong>{project?.name ?? 'Your workspace'}</strong><small
            >{project?.assetType ?? 'A place for your ideas'}</small
          ></span
        ><Icon name="chevron" size={13} /></button
      >
      {#if projectsOpen}<div class="project-menu">
          <span class="eyebrow">COLLECTIONS</span>{#each studio.library?.projects ?? [] as p}<div
              class="project-menu-row"
            >
              <button class:current={p.id === project?.id} onclick={() => switchProject(p.id)}
                ><span>{p.name}</span><small>{p.count}</small></button
              ><button
                class="icon-button"
                aria-label={`Delete ${p.name}`}
                title="Delete collection"
                onclick={() => showDeleteCollection(p)}><Icon name="trash" size={15} /></button
              >
            </div>{/each}<button onclick={showCollections}
            ><Icon name="grid" size={15} />All collections</button
          ><button
            onclick={() => {
              projectsOpen = false;
              modal = 'new';
            }}><Icon name="plus" size={15} />New collection</button
          >
        </div>{/if}
    </div>
    <nav aria-label="Workspaces">
      {#each nav as item}<button
          class:active={mode === item.id}
          disabled={!project}
          onclick={() => (mode = item.id)}
          title={item.label}
          ><Icon name={item.icon} /><span>{item.label}</span
          >{#if item.id === 'assets' && project}<small>{project.assets.length}</small
            >{:else if item.id === 'queue' && activeJobs.length}<small class="queue-count"
              >{activeJobs.length}</small
            >{/if}</button
        >{/each}
    </nav>
    {#if project}<div class="style-mini">
        <span class="eyebrow">SHARED DIRECTION</span><button onclick={() => (mode = 'style')}
          ><span class="mini-orbit"><Icon name="style" size={21} /></span><strong
            >Style profile <small>v{project.style.version}</small></strong
          ></button
        >
        <p>{project.style.direction || 'A shared style makes every asset feel at home.'}</p>
        <span>{project.style.references.length} references</span>
      </div>{/if}
    <div class="sidebar-bottom">
      {#if activeJobs.length}<button class="running-queue" onclick={() => (mode = 'queue')}
          ><span class:spinner={!project?.paused} class="status-dot"></span><span
            >{project?.paused ? 'Queue paused' : 'Creating your collection'}<small
              >{activeJobs.length} remaining</small
            ></span
          ></button
        >{/if}<button
        class:active={mode === 'settings'}
        onclick={() => (mode = 'settings')}
        title="Settings"
        ><Icon name="settings" /><span>Settings</span><span
          class="connection-dot"
          class:connected={studio.provider?.authenticated}
        ></span></button
      >
      <div class="local-label">
        <span class="status-dot"></span><span>Saved on this computer</span>
      </div>
    </div>
  </aside>
  <div class="studio-main">
    <header class="topbar">
      <div class="breadcrumb">
        {#if project && mode !== 'collections'}<button
            class="breadcrumb-collection"
            onclick={showAssets}
            title="View collection assets">{project.name}</button
          >{:else}<span>Spiraler</span>{/if}<Icon name="chevron" size={12} /><strong
          >{mode === 'collections'
            ? 'Collections'
            : mode === 'settings'
              ? 'Settings'
              : nav.find((n) => n.id === mode)?.label}</strong
        >
      </div>
      <div class="topbar-actions">
        {#if project && mode !== 'collections'}<span class="save-state"
            >{studio.pending
              ? 'Working…'
              : studio.unsaved
                ? 'Unsaved edits'
                : 'Saved locally'}</span
          >{/if}{#if canInspect}<button
            class="icon-button"
            aria-label="Toggle inspector"
            aria-pressed={inspector}
            title="Toggle inspector · ⌘I"
            onclick={() => (inspector = !inspector)}><Icon name="panel" /></button
          >{/if}
      </div>
    </header>
    {#if studio.error}<div class="error-banner" role="alert">
        <Icon name="info" />
        <p>{studio.error}</p>
        <button class="icon-button" onclick={() => (studio.error = '')} aria-label="Dismiss error"
          ><Icon name="close" size={16} /></button
        >
      </div>{/if}
    {#each studio.library?.warnings ?? [] as warning}<div class="error-banner" role="alert">
        <p>{warning}</p>
      </div>{/each}
    {#if studio.loading}<div class="loading-screen">
        <Logo size={48} /><span>Opening your studio…</span>
      </div>
    {:else if mode === 'settings'}<SettingsWorkspace />
    {:else if studio.library && (mode === 'collections' || (!project && (studio.library.projects.length || studio.library.deletedProjects.length)))}<CollectionsWorkspace
        library={studio.library}
        onopen={switchProject}
        oncreate={() => (modal = 'new')}
        ondelete={showDeleteCollection}
      />
    {:else if !project}<div class="welcome">
        <div class="welcome-mark"><Logo size={100} /></div>
        <span class="eyebrow">MANY ASSETS. ONE VISUAL WORLD.</span>
        <h1>A whole collection.<br />A common thread.</h1>
        <p>
          Give your ideas a shared style, then bring them to life.<br />A local studio for the
          things you want to make.
        </p>
        <button class="primary large" onclick={() => (modal = 'new')}
          >Create your first project<Icon name="arrow" /></button
        >
        <div class="welcome-steps">
          <span><b>01</b>Set a style</span><span><b>02</b>Make your list</span><span
            ><b>03</b>Shape a collection</span
          >
        </div>
      </div>
    {:else if mode === 'style'}{#key project.id}<StyleWorkspace {project} />{/key}
    {:else if mode === 'queue'}<QueueWorkspace
        {project}
        oninspect={(id) => {
          selected = [id];
          mode = 'assets';
          inspector = true;
        }}
      />
    {:else if mode === 'export'}{#key project.id}<ExportWorkspace {project} {selected} />{/key}
    {:else}<div class="assets-layout">
        <main class="collection">
          <div class="collection-heading">
            <div>
              <span class="eyebrow">YOUR COLLECTION</span>
              <h1>{project.name}</h1>
              <p>
                {project.assets.length
                  ? `${project.assets.length} assets · ${readyCount} ready · ${approvedCount} approved`
                  : 'Every great collection starts with a few ideas.'}
              </p>
            </div>
            <div class="button-row">
              <button
                onclick={() => {
                  bulkError = '';
                  modal = 'bulk';
                }}><Icon name="plus" size={16} />Add assets</button
              ><button class="primary" disabled={!project.assets.length} onclick={() => generate()}
                ><Icon name="sparkle" size={16} /><span
                  >Generate{selected.length ? ` ${selected.length}` : ''}</span
                ></button
              >
            </div>
          </div>
          {#if project.assets.length}<div class="collection-toolbar">
              <div class="filters" aria-label="Filter assets">
                {#each [['all', 'All assets'], ['ready', 'Ready'], ['approved', 'Approved'], ['draft', 'Drafts'], ['failed', 'Failed']] as [value, label]}<button
                    class:active={filter === value}
                    aria-pressed={filter === value}
                    onclick={() => (filter = value as Filter)}>{label}</button
                  >{/each}
              </div>
              <label class="search"
                ><Icon name="search" size={15} /><input
                  bind:this={searchInput}
                  bind:value={search}
                  placeholder="Find an asset"
                  aria-label="Find an asset"
                /></label
              >
            </div>{/if}
          {#if selected.length}<div class="selection-toolbar">
              <span><b>{selected.length}</b> selected</span><button
                class="text-button"
                onclick={() => (selected = visible.map((a) => a.id))}>Select all</button
              ><span class="toolbar-divider"></span><button
                class="icon-button"
                title="Duplicate selected"
                aria-label="Duplicate selected"
                onclick={() => act({ type: 'duplicate', ids: selected })}
                ><Icon name="copy" size={16} /></button
              ><button
                class="text-button"
                onclick={() => {
                  batchInstructions = '';
                  modalIds = [...selected];
                  modal = 'batch';
                }}>Edit together</button
              ><button
                class="text-button"
                disabled={!selectedReady.length}
                onclick={() => act({ type: 'approve', ids: selectedReady, approved: true })}
                >Approve</button
              ><button
                class="text-button"
                disabled={!selectedReady.length}
                onclick={() => showUpscale()}>Upscale</button
              ><button
                class="icon-button"
                title="Delete selected"
                aria-label="Delete selected"
                onclick={showDelete}><Icon name="trash" size={16} /></button
              ><button
                class="icon-button selection-clear"
                aria-label="Clear selection"
                onclick={() => (selected = [])}><Icon name="close" size={15} /></button
              >
            </div>{/if}
          <div class="collection-scroll">
            {#if !project.assets.length}<div class="empty-workspace collection-empty">
                <div class="empty-grid" aria-hidden="true">
                  <span><Icon name="image" size={27} /></span><span
                    ><Icon name="plus" size={27} /></span
                  ><span><Icon name="image" size={27} /></span>
                </div>
                <h2>What belongs in your world?</h2>
                <p>
                  Write one idea per line. A potion, a character, a tiny tree.<br />They will all
                  share the style you define.
                </p>
                <button class="primary" onclick={() => (modal = 'bulk')}
                  ><Icon name="plus" size={16} />Add your first assets</button
                ><button class="text-button" onclick={() => (mode = 'style')}
                  >Or start with your style <Icon name="arrow" size={14} /></button
                >
              </div>
            {:else if !visible.length}<div class="empty-workspace">
                <Icon name="search" size={32} />
                <h2>No assets here yet.</h2>
                <p>Try a different search or filter.</p>
                <button
                  onclick={() => {
                    search = '';
                    filter = 'all';
                  }}>Show all assets</button
                >
              </div>
            {:else}<AssetGrid
                assets={visible}
                jobs={project.jobs}
                bind:selected
                {density}
                onopen={() => (inspector = true)}
                onreorder={reorder}
              />{/if}
          </div>
          <footer class="collection-footer">
            <span
              >{visible.length}
              {visible.length === 1 ? 'asset' : 'assets'}{selected.length
                ? ` · ${selected.length} selected`
                : ''}</span
            >
            <div>
              {#if selected.length === 1}<button
                  class="text-button"
                  onclick={() => moveSelected(-1)}
                  title="Move earlier">Move earlier</button
                ><button class="text-button" onclick={() => moveSelected(1)} title="Move later"
                  >Move later</button
                >{/if}<label class="density"
                ><Icon name="grid" size={13} /><input
                  type="range"
                  min="150"
                  max="280"
                  step="10"
                  bind:value={density}
                  aria-label="Thumbnail size"
                /></label
              >
            </div>
          </footer>
        </main>
        {#if inspector && selectedAsset}{#key selectedAsset.id}<Inspector
              asset={selectedAsset}
              {project}
              onclose={() => (inspector = false)}
              onupscale={() => showUpscale()}
              ondelete={showDelete}
              onexport={() => (mode = 'export')}
            />{/key}{/if}
      </div>{/if}
  </div>
  {#if dragging}<div class="drop-overlay">
      <Icon name="image" size={40} />
      <h2>
        {mode === 'style'
          ? 'Add to your reference board'
          : selected.length === 1
            ? 'Add images to this asset'
            : 'Select an asset or open Style to import'}
      </h2>
      <p>PNG, JPEG or WebP · Up to 50 MB each</p>
    </div>{/if}
  {#if studio.notice}<div class="toast" role="status">
      <Icon name="check" size={16} /><span>{studio.notice}</span><button
        class="icon-button"
        aria-label="Dismiss notification"
        onclick={() => (studio.notice = '')}><Icon name="close" size={14} /></button
      >
    </div>{/if}
</div>

{#if modal === 'new'}<Dialog title="A new collection" onclose={() => (modal = null)}
    ><form
      onsubmit={(e) => {
        e.preventDefault();
        void newProject();
      }}
    >
      <p class="dialog-intro">One project, one visual family. Start with a name.</p>
      <label class="field"
        >Project name<input
          bind:value={projectName}
          placeholder="Woodland inventory"
          maxlength="120"
          required
        /></label
      ><label class="field"
        >What are you making?<select bind:value={assetType}
          ><option>Game assets</option><option>Icons</option><option>Illustrations</option><option
            >Characters</option
          ><option>Stickers</option><option>Product images</option><option>Marketing assets</option
          ><option>UI artwork</option></select
        ></label
      ><label class="field"
        >A few words about it <span class="muted">Optional</span><textarea
          bind:value={projectDescription}
          rows="2"
          maxlength="4000"
          placeholder="A cozy adventure in a world of small things."></textarea></label
      >
      <p class="small muted">You can add style references in the next step.</p>
      <div class="dialog-footer">
        <button type="button" onclick={() => (modal = null)}>Cancel</button><button
          class="primary"
          disabled={!projectName.trim() || localBusy}
          >Create project <Icon name="arrow" size={16} /></button
        >
      </div>
    </form></Dialog
  >
{:else if modal === 'bulk'}<Dialog
    title="Make a little list. Or a big one."
    wide
    onclose={() => (modal = null)}
    ><form
      onsubmit={(e) => {
        e.preventDefault();
        void addAssets();
      }}
    >
      <p class="dialog-intro">
        One asset per line. Add a description after a dash, or paste a CSV with name and subject
        columns.
      </p>
      <label class="field bulk-field"
        ><span>What should we make?</span><textarea
          bind:value={bulkText}
          rows="10"
          spellcheck="false"
          oninput={() => (bulkError = '')}
          placeholder={'Health potion — a round glass bottle of red potion\nMana potion — a tall vial of glowing blue potion\nWooden shield — a small round shield with iron trim\nTreasure chest — a locked oak chest with brass corners'}
        ></textarea></label
      >{#if bulkError}<p class="failed" role="alert">{bulkError}</p>{/if}
      <div class="bulk-tools">
        <button
          type="button"
          class="text-button"
          onclick={async () => {
            const text = await importAssetList();
            if (text) bulkText = text;
          }}>Import TXT, CSV or TSV</button
        ><span>{bulkCount} {bulkCount === 1 ? 'asset' : 'assets'} in this list</span>
      </div>
      <div class="dialog-footer">
        <button type="button" onclick={() => (modal = null)}>Cancel</button><button
          class="primary"
          disabled={!bulkText.trim() || localBusy}>Add {bulkCount || ''} assets</button
        >
      </div>
    </form></Dialog
  >
{:else if modal === 'delete'}<Dialog
    title={`Delete ${modalIds.length} ${modalIds.length === 1 ? 'asset' : 'assets'}?`}
    onclose={() => (modal = null)}
    ><p>
      These assets and their review history will be removed from the collection. Image files remain
      in the project folder for recovery. Queued jobs will be cancelled.
    </p>
    <div class="dialog-footer">
      <button onclick={() => (modal = null)}>Keep assets</button><button
        class="danger"
        onclick={removeAssets}>Delete assets</button
      >
    </div></Dialog
  >
{:else if modal === 'upscale'}<Dialog
    title={`Upscale ${modalIds.length} ${modalIds.length === 1 ? 'asset' : 'assets'}`}
    onclose={() => (modal = null)}
    ><div class="upscale-illustration">
      <span>1×</span><Icon name="arrow" size={26} /><span>2×</span>
    </div>
    <p>
      Enlarge the preferred results to twice their width and height using local Lanczos
      interpolation. Originals are preserved.
    </p>
    <p class="muted">
      This smooths an image at a larger size. It does not add new AI detail or guarantee lossless
      quality.
    </p>
    <div class="dialog-footer">
      <button onclick={() => (modal = null)}>Cancel</button><button
        class="primary"
        onclick={async () => {
          await generate(modalIds, 'upscale');
          modal = null;
        }}>Queue 2× enlargement</button
      >
    </div></Dialog
  >
{:else if modal === 'deleteCollection' && collectionToDelete}<Dialog
    title={`Delete “${collectionToDelete.name}”?`}
    onclose={() => {
      if (!localBusy) modal = null;
    }}
  >
    <p>
      This removes the collection and all {collectionToDelete.count} assets from your studio. Queued and
      running jobs will be cancelled.
    </p>
    <p class="muted">You can restore the collection and its images from Deleted collections.</p>
    <div class="dialog-footer">
      <button disabled={localBusy} onclick={() => (modal = null)}>Keep collection</button><button
        class="danger"
        disabled={localBusy}
        onclick={removeCollection}>{localBusy ? 'Deleting…' : 'Delete collection'}</button
      >
    </div>
  </Dialog>
{:else if modal === 'batch'}<Dialog
    title={`Edit ${modalIds.length} assets together`}
    onclose={() => (modal = null)}
    ><form
      onsubmit={async (e) => {
        e.preventDefault();
        if (
          await act({ type: 'appendInstructions', ids: modalIds, instructions: batchInstructions })
        ) {
          modal = null;
          notify('Instructions added to selected assets.');
        }
      }}
    >
      <p>
        Add a shared instruction to each selected subject. Existing images and queued jobs keep
        their original instructions.
      </p>
      <label class="field"
        >Additional instructions<textarea
          bind:value={batchInstructions}
          rows="4"
          maxlength="4000"
          placeholder="Add a small worn brass detail to each object."
          required></textarea></label
      >
      <div class="dialog-footer">
        <button type="button" onclick={() => (modal = null)}>Cancel</button><button
          class="primary"
          disabled={!batchInstructions.trim()}>Apply to {modalIds.length} assets</button
        >
      </div>
    </form></Dialog
  >{/if}
