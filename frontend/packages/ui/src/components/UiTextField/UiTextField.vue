<script setup lang="ts">
import { ref } from 'vue';
import { vColor } from '../../directives/vColor';
import UiIcon from '../UiIcon/UiIcon.vue';

const props = withDefaults(defineProps<{
    type?: 'text' | 'password' | 'number' | 'url' | 'email' | 'search'
    min?: number;
    minlength?: number;
    max?: number;
    maxlength?: number;
    step?: number;
    autocomplete?: string
    placeholder?: string;
    disabled?: boolean;


    hideMessages?: boolean
    error?: string
    hint?: string
    color?: string
}>(), { autocomplete: 'off', color: 'primary' });

const showPassword = defineModel<boolean>('showPassword')

</script>

<template>
    <div v-color="color" class="ui-textfield w-full" :class="{ 'ui-textfield--error': error }">
        <div class="flex items-start ui-textfield__field mt-2 w-full bg--surface ">

            <input class="flex-1 ui-textfield__control bg-transparent fg--foreground px-2 py-1"
                :type="showPassword ? 'text' : type" :min :max :minlength :maxlength :disabled :step :placeholder
                :autocomplete />
            <div>
                <div class="pt-1 px-2" v-if="type === 'password'" @click="showPassword = !showPassword">
                    <UiIcon class="fg--muted hover:fg--foreground cursor-pointer"
                        :name="showPassword ? 'mdiEyeClosed' : 'mdiEye'" />
                </div>
            </div>
        </div>
        <p class="text-3 h-3">
            <span v-if="error" class="fg--danger">{{ error }}</span>
            <span v-else-if="hint" class="fg--muted">{{ hint }}</span>
        </p>
    </div>
</template>

<style lang="scss">
.ui-textfield {
    &--error {
        .ui-textfield__field {
            border: 1px solid var(--color-danger);
        }
    }

    &:not(&--error) {
        .ui-textfield__field {
            border: 1px solid var(--current-color);
        }
    }

    &__field {
        &:focus-within {
            outline: 3px solid var(--current-color);
        }
    }

    &__control {
        border: none;
        outline: none;
    }
}
</style>