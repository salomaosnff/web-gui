import vue from '@vitejs/plugin-vue'
import path from 'path'
import Uno from 'unocss/vite'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import VueRouter from 'unplugin-vue-router/vite'
import url from 'url'
import { defineConfig } from 'vite'
import Externalize from 'vite-plugin-externalize-dependencies'

const PROJECT_ROOT = path.dirname(url.fileURLToPath(import.meta.url))


// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    VueRouter(),
    vue(),
    Uno({
      configFile: '../ui/uno.config.ts',
    }),
    Externalize({
      externals: [
        (source) => /^lenz(:?\/.+)?$/.test(source),
      ]
    }),
    AutoImport({
      imports: ['vue', 'vue-router', '@vueuse/core'],
      dirs: ['src/store', 'src/domain', 'src/containers', 'src/directives', '../ui/src/composable'],
    }),
    Components({
      dirs: ['src/components', path.resolve(PROJECT_ROOT, '../ui/src/components'), path.resolve('../ui/src/directives')],
      deep: true,
    })
  ],
})
