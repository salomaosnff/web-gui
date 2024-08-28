<script setup lang="ts">
import { ref, watchEffect } from 'vue';
import { z } from 'zod';
import { set } from 'lodash-es';

const IDLE = 0x20;
const SUBMITTING = 0x10
const VALIDATING = 0x08;
const VALIDATED = 0x04;
const SUBMITED = 0x02;
const INVALID = 0x01;

const formState = ref<number>(IDLE)

const props = defineProps<{
    schema: z.ZodType,
    onSubmit?(value: any): Promise<any>

}>()

const modelValue = defineModel<any>()
const isSubmitting = defineModel<boolean>('isSubmitting', { default: false })
const errors = defineModel<Record<string, string>>('errors')

const lazyValue = ref(modelValue.value)

watchEffect(() => {
    lazyValue.value = modelValue.value
})

async function submit() {
    if ( formState.value & SUBMITTING) {
        throw new Error('O formulário está no estado SUBMITTING')
    }

    isSubmitting.value = true
    const values = await validate()
    modelValue.value = values

    try {
        await props.onSubmit?.(values)
    } finally {
        isSubmitting.value = false
    }


}

function reset() { }

async function validate() {
    if (formState.value & VALIDATING) {
        throw new Error('O formulário está no estado VALIDATING')
    }

    if(!props.schema){
        return lazyValue.value
    }

    const newErrors: Record<string, string> = {}
    try {
        formState.value |= VALIDATING

        errors.value = undefined
        const result = await props.schema.safeParseAsync(lazyValue.value)
        console.log(lazyValue.value, result);
        if (result.success) {
            
            return result.data
        } else {
            for (const issue of result.error.issues) {
                const path = issue.path.join('.')
                set(newErrors, path, issue.message)
            }
        }
    } finally {
        formState.value &= ~VALIDATING
        errors.value = newErrors
    }


}

</script>

<template>
    <form @submit.prevent="submit" @reset="reset">
        <slot v-if="lazyValue" :data="lazyValue"></slot>
        <pre>{{ errors }}</pre>
        <pre>{{ lazyValue }}</pre>
        <button @click="validate">Validar</button>
    </form>
</template>