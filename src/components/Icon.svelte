<script lang="ts">
  let { name, size = 18 }: { name: string; size?: number } = $props();
  const paths: Record<string, string> = {
    grid: 'M3 3h7v7H3z M14 3h7v7h-7z M3 14h7v7H3z M14 14h7v7h-7z',
    style: 'm12 3 9 5-9 5-9-5 9-5z M3 12l9 5 9-5 M3 16l9 5 9-5',
    queue: 'M8 6h13 M8 12h13 M8 18h13 M3 6h.01 M3 12h.01 M3 18h.01',
    export: 'M12 3v12 m-5-5 5 5 5-5 M4 16v5h16v-5',
    plus: 'M12 5v14 M5 12h14',
    close: 'm6 6 12 12 M18 6 6 18',
    check: 'm5 12 4 4L19 6',
    chevron: 'm9 5 7 7-7 7',
    search: 'M21 21l-5-5 M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0',
    play: 'm8 4 12 8-12 8V4z',
    pause: 'M8 5v14 M16 5v14',
    sparkle: 'm12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5L12 3z',
    arrow: 'M5 12h14 m-5-5 5 5-5 5',
    retry: 'M3 10a9 9 0 1 1 2 8 M3 3v7h7',
    trash: 'M3 6h18 M9 6V3h6v3 M5 6l1 15h12l1-15 M10 10v7 M14 10v7',
    copy: 'M8 8h13v13H8z M16 8V3H3v13h5',
    image: 'M3 3h18v18H3z m0 13 5-5 6 6 4-4 3 3 M15 7h.01',
    folder: 'M3 5h7l2 3h9v12H3V5z',
    settings: 'M4 7h16 M4 17h16 M8 4v6 M16 14v6',
    upscale: 'M14 3h7v7 m0-7-8 8 M10 21H3v-7 m0 7 8-8',
    info: 'M12 11v6 M12 7h.01 M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0',
    panel: 'M3 3h18v18H3z M15 3v18',
    list: 'M3 5h18 M3 12h18 M3 19h18',
    circle: 'M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0',
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width="1.6"
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"><path d={paths[name] ?? paths.circle} /></svg
>
