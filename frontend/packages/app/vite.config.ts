import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'
import Externalize from 'vite-plugin-externalize-dependencies'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue(), Externalize({
    externals: [
      (source) => /^lenz(:?\/.+)?$/.test(source),
    ]
  })],
})
