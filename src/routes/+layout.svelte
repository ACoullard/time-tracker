<script>
  import "../app.css";
  import ModalOverlay from "$lib/components/modal-overlay.svelte";
  let { children } = $props();
</script>

{@render children()}
<ModalOverlay />
