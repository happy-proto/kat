<script lang="ts">
  export let title: string = "Dracula";
</script>

<section class="panel">
  {#if title}
    <h1>{title}</h1>
  {/if}
</section>

<style>
  .panel {
    color: var(--accent);
  }
</style>
