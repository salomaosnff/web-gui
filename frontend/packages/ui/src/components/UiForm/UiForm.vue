<script setup lang="ts" generic="S extends z.ZodType">
import { z } from 'zod';
import { provideForm } from '../../composable/useForm';

const props = defineProps<{
    schema: S,
    validateOnMount?: boolean,
    onSubmit?(value: z.output<S>): Promise<any>
}>()

const modelValues = defineModel<z.infer<S> | z.input<S>>()

const form = provideForm({
    schema: () => props.schema,
    values: modelValues,
    onSubmit: async values => props.onSubmit?.(values),
    validateOnMount: () => props.validateOnMount
})
</script>

<template>
    <form @submit.prevent="form.submit" @reset.prevent="form.reset">
        <slot :form="form"></slot>
    </form>
</template>