<script lang="ts">
  import type { Project } from '../lib/types';
  import { act } from '../lib/studio.svelte';
  import Icon from './Icon.svelte';
  let { project, oninspect }: { project: Project; oninspect: (id: string) => void } = $props();
  const pending = $derived(project.jobs.filter((j) => ['queued', 'generating'].includes(j.status)));
  const failures = $derived(project.jobs.filter((j) => j.status === 'failed'));
  const completed = $derived(project.jobs.filter((j) => j.status === 'ready').length);
</script>

<div class="scroll-workspace queue-workspace">
  <div class="workspace-heading">
    <div>
      <span class="eyebrow">A LITTLE WORK IN PROGRESS</span>
      <h1>The generation queue</h1>
      <p>
        {pending.length
          ? `${pending.length} ${pending.length === 1 ? 'job' : 'jobs'} remaining. Keep working while they take shape.`
          : 'Every job keeps its own history. Your finished images stay safe.'}
      </p>
    </div>
    {#if pending.length}<button onclick={() => act({ type: 'pause', paused: !project.paused })}
        ><Icon name={project.paused ? 'play' : 'pause'} />{project.paused
          ? 'Resume queue'
          : 'Pause queue'}</button
      >{/if}
  </div>
  {#if project.paused}<div class="callout">
      <Icon name="pause" />
      <div>
        <strong>The queue is paused</strong>
        <p>Resume when you are ready. A job already running can finish.</p>
      </div>
      <button onclick={() => act({ type: 'pause', paused: false })}>Resume</button>
    </div>{/if}
  {#if !project.jobs.length}<div class="empty-workspace">
      <Icon name="queue" size={38} />
      <h2>Room for your next idea.</h2>
      <p>
        Add some assets, then choose Generate.<br />Each image will appear here as it takes shape.
      </p>
    </div>
  {:else}
    <div class="queue-summary">
      <span
        >{completed} ready · {pending.length} in queue{failures.length
          ? ` · ${failures.length} need attention`
          : ''}</span
      >
      <div>
        {#if failures.length}<button
            class="text-button"
            onclick={() => act({ type: 'retry', ids: failures.map((j) => j.id) })}
            >Retry failed</button
          >{/if}{#if pending.length}<button
            class="text-button"
            onclick={() => act({ type: 'cancel', ids: pending.map((j) => j.id) })}
            >Cancel remaining</button
          >{/if}
      </div>
    </div>
    <div class="job-list">
      {#each [...project.jobs].reverse() as job (job.id)}
        <article class="job-row">
          <div
            class="job-icon"
            class:failed={job.status === 'failed'}
            class:ready={job.status === 'ready'}
          >
            {#if job.status === 'generating'}<span class="spinner"></span>{:else}<Icon
                name={job.status === 'ready'
                  ? 'check'
                  : job.status === 'failed'
                    ? 'info'
                    : job.status === 'cancelled'
                      ? 'close'
                      : 'circle'}
              />{/if}
          </div>
          <div class="job-description">
            <button class="text-button job-title" onclick={() => oninspect(job.assetId)}
              >{job.assetName}</button
            ><span
              >{job.kind === 'upscale'
                ? '2× local enlargement'
                : job.kind === 'variant'
                  ? 'Variation'
                  : 'Generation'} · Style v{job.request.style.version}{job.attempts > 1
                ? ` · Attempt ${job.attempts} of 3`
                : ''}</span
            >{#if job.error}<p class:failed={job.status === 'failed'}>
                {job.error}
              </p>{/if}{#if job.retryAt > Date.now() / 1000 && job.status === 'queued'}<p>
                Waiting briefly before retrying.
              </p>{/if}
          </div>
          <span class="job-status">{job.status}</span
          >{#if ['queued', 'generating'].includes(job.status)}<button
              class="icon-button"
              title="Cancel job"
              aria-label={`Cancel ${job.assetName}`}
              onclick={() => act({ type: 'cancel', ids: [job.id] })}><Icon name="close" /></button
            >{:else if ['failed', 'cancelled'].includes(job.status)}<button
              class="icon-button"
              title="Retry job"
              aria-label={`Retry ${job.assetName}`}
              onclick={() => act({ type: 'retry', ids: [job.id] })}><Icon name="retry" /></button
            >{/if}
        </article>
      {/each}
    </div>
  {/if}
</div>
