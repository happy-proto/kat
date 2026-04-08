<template>
  <section class="panel">
    <h1>{{ title.toUpperCase() }}</h1>
  </section>
</template>

<script lang="ts">
interface Props {
  title: string;
}

const props: Props = {
  title: "Dracula",
};
</script>

<style>
.panel {
  color: var(--accent);
}
</style>
