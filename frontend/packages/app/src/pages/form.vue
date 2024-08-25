<script setup lang="ts">


const value = ref(['A', 'A', 'C'])

</script>

<template>
    <UiCheckbox v-model="value" truthy-value="A">Selecionar A</UiCheckbox>
    <UiCheckbox v-model="value" truthy-value="B">Selecionar B</UiCheckbox>
    <UiCheckbox v-model="value" truthy-value="C">Selecionar C</UiCheckbox>

  <div class="flex items-center theme--dark pa-40">
    <UiTextField placeholder="placeholder"  error="Campo invalido"/>
    <UiTextField placeholder="placeholder"  hint="Compo com hint"/>
    <UiTextField  placeholder="placeholder" type="password"/>
    <UiBtn>Botão</UiBtn>
  </div>

    <pre>{{ value }}</pre>
</template>