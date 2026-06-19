import type { Snippet } from 'svelte';

let content = $state<Snippet | null>(null);

export const pageHeader = {
  get content() { return content; },
  set(s: Snippet | null) { content = s; }
};
