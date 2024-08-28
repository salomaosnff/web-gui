<script setup lang="ts">
import { markRaw, ref } from 'vue';
import { string, z } from 'zod';

const form = ref({ name: ''})

const schema  = markRaw(z.object({
  name: z.string().min(1, 'Campo Obrigátorio')
}))
</script>



<template>
  <div class="bg--surface pa-4 mx-auto w-120 rounded-md mb-8">
    <p class="text-6 mb-4">Fazer Login</p>
    <UiTextField placeholder="Digite..." label="E-mail" type="email" />
    <UiTextField placeholder="Digite..." label="Password" type="password" />
    <div class="text-right">
      <UiBtn class="mt-4" type="submit">Submit</UiBtn>
    </div>
  </div>
  <div class="bg--surface pa-4 mx-auto w-80 rounded-md mb-8">
    <p class="text-6 mb-4">Fazer Login</p>

    <UiTextField placeholder="E-mail" type="email" />
    <UiTextField placeholder="Password" type="password" />
    <UiBtn class="mt-4 w-full" type="submit">Submit</UiBtn>
  </div>

  <UiForm :schema="schema" v-model="form" v-slot="{data}">
    <input v-model="data.name" />
  </UiForm>
</template>