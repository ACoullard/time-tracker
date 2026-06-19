<script>
  import NavPill from "$lib/components/nav-pill.svelte";
  import ModalOverlay from "$lib/components/modal-overlay.svelte";
  import { pageHeader } from '$lib/page-header.svelte';
  let { children } = $props();
</script>

<div class="h-screen flex flex-col">
  <header class="h-16 shrink-0 flex items-center px-8">
    <NavPill />
    <div class="flex-1"></div>
    {#if pageHeader.content}
      {@render pageHeader.content()}
    {/if}
  </header>
  <div class="flex-1 min-h-0">
    {@render children()}
  </div>
</div>
<ModalOverlay />
