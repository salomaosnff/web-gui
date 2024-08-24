<script setup lang="ts">
import { computed, ref } from 'vue';
import { vColor } from '../../directives/vColor';

const props = withDefaults(
  defineProps<{
    is?: string
    color?: string
    disabled?: boolean
    loading?: boolean
    icon?: boolean
    flat?: boolean
    onClick?(event: MouseEvent): void | Promise<void>
  }>(),
  {
    is: 'button',
    color: 'primary',
  }
)

const pendingTask = ref(false)

async function handleClick(event: MouseEvent) {
  if (isDisabled.value || typeof props.onClick !== 'function') {
    return
  }

  try {
    pendingTask.value = true
    await props.onClick(event)
  } finally {
    pendingTask.value = false
  }
}

const isLoading = computed(() => props.loading || pendingTask.value)
const isDisabled = computed(() => props.disabled || pendingTask.value)

</script>
<template>
  <component v-color="color" :is="is" class="ui-btn" :class="{
    'ui-btn--disabled': isDisabled,
    'cursor-wait': isLoading,
    'ui-btn--icon': icon,
    'ui-btn--flat': flat,
  }" @click="handleClick">
    <Transition>
      <div v-if="isLoading"
        class="ui-btn__loading w-full h-full absolute top-0 left-0 flex items-center justify-center">
        <UiIcon name="mdiRefresh" class="animate-spin" />
      </div>
    </Transition>
    <div class="ui-btn__content" :class="{
      'ui-btn__content--loading': isLoading
    }">
      <slot />
    </div>
  </component>
</template>

<style lang="scss">
.ui-btn {
  background: var(--current-color);
  color: oklch(from white l c h / 82%);
  @apply inline-flex items-center justify-center px-4 py-2 rounded-md relative;

  &--icon {
    aspect-ratio: 1;
    @apply p-2 rounded-full;
  }

  &--flat {
    background: transparent;
    color: var(--current-color);

    &:hover {
      background: var(--current-surface-color);
    }
  }

  &__loading {

    &.v-enter-active,
    &.v-leave-active {
      transition: all .25s;
    }

    &.v-enter-active {
      transition-delay: 0.25s;
    }

    &.v-enter-from,
    &.v-leave-to {
      transform: scale(0);
    }
  }

  &__content {
    transition: transform .25s;
    transition-delay: 0.25s;

    &--loading {
      transform: scale(0);
      transition-delay: 0s;
    }
  }

  &:focus-visible {
    outline: 2px solid var(--current-color);
    outline-offset: 2px;
  }

  &--disabled {
    pointer-events: none;
    color: oklch(from var(--color-foreground) l c h / 50%);

    &:not(.ui-btn--flat) {
      background: oklch(from var(--color-foreground) l c h / 12.5%) !important;
    }
  }

  &:not(&--icon)>.ui-btn__content {
    display: flex;
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;

    .ui-icon {
      height: 1.25em;

      &:first-child {
        @apply ml--2 mr-1;
      }

      &:last-child {
        @apply mr--2 ml-1;
      }
    }
  }
}
</style>