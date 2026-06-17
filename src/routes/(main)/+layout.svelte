<script>
  import NavPill from "$lib/components/nav-pill.svelte";
  import ModalOverlay from "$lib/components/modal-overlay.svelte";
  let { children } = $props();
</script>

<div class="pt-10">
  {@render children()}
</div>
<NavPill />
<ModalOverlay />
