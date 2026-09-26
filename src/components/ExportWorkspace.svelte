<script lang="ts">
  import { untrack } from 'svelte';
  import type { Project, ExportOptions, ExportReport } from '../lib/types';
  import { exportAssets, studio } from '../lib/studio.svelte';
  import { preferred } from '../lib/assets';
  import Icon from './Icon.svelte';
  let { project, selected }: { project: Project; selected: string[] } = $props();
  let scope = $state<'selected' | 'approved' | 'all'>(
    untrack(() => (selected.length ? 'selected' : 'approved')),
  );
  let version = $state<ExportOptions['version']>('preferred');
  let metadata = $state(true);
  let prefix = $state('');
  let report = $state<ExportReport | null>(null);
  const assets = $derived(
    project.assets.filter(
      (a) => scope === 'all' || (scope === 'approved' ? a.approved : selected.includes(a.id)),
    ),
  );
  const count = $derived(
    assets.filter((a) =>
      version === 'upscaled' ? a.results.some((r) => r.kind === 'upscale') : !!preferred(a),
    ).length,
  );
  async function run() {
    report =
      (await exportAssets({ ids: assets.map((a) => a.id), version, metadata, prefix })) ?? null;
  }
</script>

<div class="scroll-workspace export-workspace">
  <div class="workspace-heading">
    <div>
      <span class="eyebrow">READY FOR THE WORLD</span>
      <h1>Take your collection with you.</h1>
      <p>Full-resolution PNG files, ready for whatever comes next.</p>
    </div>
  </div>
  <div class="export-layout">
    <form
      onsubmit={(e) => {
        e.preventDefault();
        void run();
      }}
    >
      <fieldset>
        <legend>Which assets?</legend
        >{#each [['selected', 'Selected assets', selected.length], ['approved', 'Approved assets', project.assets.filter((a) => a.approved).length], ['all', 'Entire project', project.assets.length]] as [value, label, number]}<label
            class="choice-row"
            ><input type="radio" name="scope" {value} bind:group={scope} /><span>{label}</span><span
              class="muted">{number}</span
            ></label
          >{/each}
      </fieldset>
      <label class="field"
        >Image version<select bind:value={version}
          ><option value="preferred">Preferred result</option><option value="original"
            >First original</option
          ><option value="upscaled">Latest upscaled result</option></select
        ></label
      >
      <label class="field"
        >Filename prefix <span class="muted">Optional</span><input
          bind:value={prefix}
          maxlength="80"
          placeholder="woodland"
        /></label
      >
      <label class="checkbox-field"
        ><input type="checkbox" bind:checked={metadata} />Include a JSON file with generation
        details</label
      >
      <div class="export-submit">
        <button class="primary" disabled={!count || studio.pending > 0}
          ><Icon name="export" />Choose folder & export {count}
          {count === 1 ? 'image' : 'images'}</button
        >
        <p class="small muted">
          A new folder is created for every export. Existing files stay intact.
        </p>
      </div>
    </form>
    <aside class="export-note">
      <Icon name="folder" size={36} />
      <h2>Your work. Your files.</h2>
      <p>
        Original images remain in your project. Exported files work in any image editor or game
        engine.
      </p>
      <div class="filename-example">
        {prefix ? `${prefix.toLowerCase().replace(/\W+/g, '-')}-` : ''}001-asset-name-a1b2c3d4.png
      </div>
      <p class="small">
        Names include a sequence number and an asset ID to keep them predictable and unique.
      </p>
      {#if assets.length > count}<p class="small">
          {assets.length - count} assets have no matching image and will be skipped.
        </p>{/if}
    </aside>
  </div>
  {#if report}<div class="callout success">
      <Icon name="check" />
      <div>
        <strong>{report.count} {report.count === 1 ? 'image' : 'images'} exported</strong>
        <p class="selectable">{report.directory}</p>
        {#if report.skipped}<p>{report.skipped} assets had no matching image.</p>{/if}
      </div>
    </div>{/if}
</div>
