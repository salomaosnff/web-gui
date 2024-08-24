<script setup lang="ts">
import * as AllIcons from '@mdi/js';
import { computed } from 'vue';

const props = defineProps<{
  name?: keyof typeof AllIcons
  path?: string
}>()

const path = computed(() => {
  if (props.name) {
    return AllIcons[props.name]
  }
  return props.path
})
</script>
<template>
  <svg class="ui-icon" viewBox="0 0 24 24" style="fill: currentColor">
    <path :d="path" />
  </svg>
</template>

<style>
.ui-icon {
  display: inline-block;
  vertical-align: middle;
  fill: currentColor;
  width: 1em;
  height: 1em;
}
</style>