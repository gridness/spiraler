<script lang="ts">
  import type { Snippet } from 'svelte';
  import Icon from './Icon.svelte';
  let {
    title,
    onclose,
    children,
    wide = false,
  }: { title: string; onclose: () => void; children: Snippet; wide?: boolean } = $props();
  let dialog: HTMLDialogElement;
  $effect(() => {
    dialog?.showModal();
  });
</script>

<dialog bind:this={dialog} class:wide oncancel={onclose}>
  <div class="dialog-heading">
    <h2>{title}</h2>
    <button class="icon-button" onclick={onclose} aria-label="Close dialog"
      ><Icon name="close" /></button
    >
  </div>
  {@render children()}
</dialog>
