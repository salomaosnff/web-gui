<script setup lang="ts"></script>
<template>
  <iframe
    class="h-full rounded-md transition-all duration-1000 mx-auto bg--surface shadow-lg"
    :class="[
      loaded ? ['visible', 'opacity-100'] : ['invisible', 'opacity-0'],
      isMobile ? 'w-480px' : 'w-full',
    ]"
    frameborder="0"
    src="https://wikipedia.org"
    anonymous
    @load="loaded = true"
  ></iframe>
</template>
