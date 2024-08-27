<script lang="ts" setup>
import { ref } from "vue";
import AppPanel from "../components/AppPanel.vue";

interface MenuItem {
  id: string;
  icon?: keyof typeof import("@mdi/js/mdi");
  title: string;
  color?: string;
  children?: MenuItem[];
}

const items: MenuItem[] = [
  {
    id: "create.text",
    icon: "mdiText",
    title: "Texto",
    children: [
      {
        id: "create.text.paragraph",
        title: "Parágrafo",
      },
      {
        id: "create.text.heading",
        title: "Título",
      },
    ],
  },
  {
    id: "create.image",
    icon: "mdiImage",
    title: "Imagem",
    children: [
      {
        id: "create.image.upload",
        title: "Nova imagem",
      },
      {
        id: "create.image.existing",
        title: "Adicionar imagem existente",
      },
      {
        id: "create.image.url",
        title: "Adicionar imagem por URL",
      },
    ],
  },
  {
    id: "create.layout",
    icon: "mdiViewDashboard",
    title: "Layout",
    children: [
      {
        id: "create.layout.flex",
        title: "Flex Layout",
      },
      {
        id: "create.layout.grid",
        title: "Grid Layout",
      },
      {
        id: "create.layout.table",
        title: "Tabela",
      },
    ],
  },
  {
    id: "create.advanced",
    icon: "mdiCodeTags",
    title: "Código",
    children: [
      {
        id: "create.advanced.outerHtml",
        title: "Editar HTML do elemento",
        icon: "mdiLanguageHtml5",
      },
      {
        id: "create.advanced.style",
        title: "Editar CSS do elemento",
        icon: "mdiLanguageCss3",
      },
      {
        id: "create.advanced.inspector",
        title: "Inspecionar Árvore de Elementos",
        icon: "mdiFileTree",
      },
    ],
  },
];

const menuPath = ref<MenuItem[]>([items[0]]);
const loaded = ref(false);
const isMobile = ref(false);
</script>
<template>
  <div class="w-full h-full flex flex-col pa-2 gap-2">
    <AppPanel>
      <div>
        <UiOverlayMenu visible origin="bottom-start">
          <template #activator="{ attrs }">
            <UiBtn v-bind="attrs" icon flat class="h-8">
              <UiIcon name="mdiTools" />
            </UiBtn>
          </template>

          <UiPopup class="!pa-0 flex">
            <div class="pa-2 bg--surface2 rounded-l-md">
              <h3
                class="font-bold text-5 pa-2 inline-flex items-center gap-2 mr-4"
              >
                <UiIcon name="mdiTools" class="text-5" />
                <span>Ferramentas</span>
              </h3>

              <ul>
                <li
                  v-for="item in items"
                  :key="item.id"
                  class="flex w-max rounded-full gap-2 py-2 px-4 text-3 items-center mb-2 cursor-pointer hover:bg--surface-muted"
                  :class="{
                    'bg--surface-primary!': menuPath.includes(item),
                  }"
                  @pointerenter="menuPath.splice(0, menuPath.length, item)"
                >
                  <UiIcon v-if="item.icon" :name="item.icon" class="text-4" />
                  <span>{{ item.title }}</span>
                </li>
              </ul>
            </div>
            <ul class="pa-4 min-w-60">
              <h3
                class="font-bold text-5 mb-2 inline-flex items-center gap-1 mr-4"
              >
                <UiIcon :name="menuPath[0]?.icon" />
                <span>{{ menuPath[0]?.title }}</span>
              </h3>

              <template v-for="item in menuPath[0]?.children" :key="item">
                <template v-if="item.children?.length">
                  <li>
                    <p class="uppercase opacity-50 text-3 mb-2 font-bold">
                      {{ item.title }}
                    </p>
                    <ul>
                      <li
                        v-for="child in item.children"
                        :key="child.id"
                        class="flex w-max rounded-full gap-2 py-2 px-4 text-3 items-center mb-2 cursor-pointer hover:bg--surface-muted"
                        :class="{
                          'bg--surface-primary!': menuPath.includes(child),
                        }"
                        @click="
                          menuPath.splice(1, menuPath.length - 1, item, child)
                        "
                      >
                        <UiIcon
                          v-if="item.icon"
                          :name="item.icon"
                          class="text-4"
                        />

                        <span>{{ child.title }}</span>
                      </li>
                    </ul>
                  </li>
                </template>
                <template v-else>
                  <li
                    class="flex w-max rounded-full gap-2 py-2 px-4 text-3 items-center mb-2 cursor-pointer hover:bg--surface-muted"
                    :class="{
                      'bg--surface-primary!': menuPath.includes(item),
                    }"
                    @click="menuPath.splice(1, menuPath.length - 1, item)"
                  >
                    <UiIcon v-if="item.icon" :name="item.icon" class="text-4" />
                    <span>{{ item.title }}</span>
                  </li>
                </template>
              </template>
            </ul>
          </UiPopup>
        </UiOverlayMenu>

        <UiBtn icon flat class="h-8" @click="isMobile = !isMobile">
          <UiIcon :name="isMobile ? 'mdiMonitor' : 'mdiCellphone'" />
        </UiBtn>
      </div>
    </AppPanel>
    <div
      class="w-full flex-1 gap-2 relative bg--surface justify-center items-center"
    >
      <iframe
        class="h-full rounded-md transition-all duration-1000 mx-auto"
        :class="[
          loaded ? ['visible', 'opacity-100'] : ['invisible', 'opacity-0'],
          isMobile ? 'w-480px' : 'w-full',
        ]"
        frameborder="0"
        src="https://wikipedia.org"
        anonymous
        @load="loaded = true"
      ></iframe>

      <UiIcon
        v-if="!loaded"
        name="mdiRefresh"
        class="fg--muted absolute top-50% right-50% translate-[-50%,-50%] animate-spin text-25"
      />
    </div>
  </div>
</template>
<style scoped>
.tool-grid {
  display: grid;
  width: 100%;
  grid-template-columns: repeat(auto-fill, 32px);
  gap: 0.5rem;
}
</style>
