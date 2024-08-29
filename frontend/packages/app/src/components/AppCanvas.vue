<script setup lang="ts">

const modelValue = defineModel<string>('html')
const hoverEl = defineModel<HTMLElement>('hoverEl')
const activeEl = defineModel<HTMLElement>('activeEl')
const loaded = ref(false)
const iframe = ref<HTMLIFrameElement>()
const isMobile = ref(false)

const hoverBox = shallowRef<DOMRect>()
const activeBox = shallowRef<DOMRect>()



watch(modelValue, () => {
  loaded.value = false
})

whenever(loaded, () => {
  iframe.value?.contentDocument?.addEventListener('pointermove', (event) => hoverEl.value = event.target as HTMLElement)
  iframe.value?.contentDocument?.addEventListener('click', (event) => activeEl.value = event.target as HTMLElement)
})

watch([hoverEl, activeEl], () => updateHover())


function updateHover() {
  if (!iframe.value) {
    return
  }

  const boxIframe = iframe.value.getBoundingClientRect()
  if (hoverEl.value) {
    const boxhoverEl = hoverEl.value.getBoundingClientRect()
    hoverBox.value = new DOMRect(boxIframe.x + boxhoverEl.x, boxIframe.y + boxhoverEl.y, boxhoverEl.width, boxhoverEl.height)
  }

  if (activeEl.value) {
    const boxActiveEl = activeEl.value.getBoundingClientRect()
    activeBox.value = new DOMRect(boxIframe.x + boxActiveEl.x, boxIframe.y + boxActiveEl.y, boxActiveEl.width, boxActiveEl.height)
  }
}

function getElementSelector(el:HTMLElement): string {
  let selector = `${el.tagName.toLowerCase()}`

  if (el.id) {
    selector += `#${el.id}`

    return selector
  }

  if (el.classList.length) {
    for (const className of el.classList) {
      selector += `.${className}`
    }

    return selector
  }
  
  return selector
}

const hoverSelector = computed(() => hoverEl.value ? getElementSelector(hoverEl.value) : '')
const activeSelector = computed(() => activeEl.value ? getElementSelector(activeEl.value) : '')

useEventListener('scroll', updateHover, { passive: true })
useEventListener(() => iframe.value?.contentWindow, 'scroll', updateHover, { passive: true })

</script>
<template>
  <div class="rounded-md transition-all duration-1000 shadow-lg">
    <iframe ref="iframe" class="h-full w-full" :class="[
      loaded ? ['visible', 'opacity-100'] : ['invisible', 'opacity-0'],
      isMobile ? 'w-480px' : 'w-full']" .srcdoc="modelValue" frameborder="0" @load="loaded = true"></iframe>

    <div class="app-canvas__hover fixed pointer-events-none"
      :style="{ width: `${hoverBox?.width}px`, height: `${hoverBox?.height}px`, top: `${hoverBox?.top}px`, left: `${hoverBox?.left}px` }"
      :data-tag="hoverSelector">
    </div>

    <div class="app-canvas__hover app-canvas__hover--active fixed pointer-events-none"
      :style="{ width: `${activeBox?.width}px`, height: `${activeBox?.height}px`, top: `${activeBox?.top}px`, left: `${activeBox?.left}px` }"
      :data-tag="activeSelector">
    </div>
  </div>
</template>

<style lang="scss">
.app-canvas {


  &__hover {
    // box-shadow: 0 0 0 1px var(--color-primary);
    outline: 1px solid var(--color-primary);
    // mix-blend-mode: difference;
    background-color: var(--color-surface-primary);

    &::before {
      content: attr(data-tag);
      position: absolute;
      left: -1px;
      bottom: 100%;
      background-color: var(--color-primary);
      padding: 2px 4px;
      font-weight: bold;
      font-size: 0.75em;
    }

    &--active {
      outline: 3px solid var(--color-secondary);

      &::before {
        left: -3px;
        padding-left: 5px;
      }
    }
  }
}
</style>
