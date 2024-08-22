import { createApp } from 'vue';
import App from './App.vue';
import './style.css';

import * as dialog from 'lenz/dialog';

dialog.show({
  title: 'Sucesso! 🎉',
  message: 'Se você está vendo essa mensagem, é porque o frontend está funcionando corretamente e a comunicação com o backend está ok 🎉',
})

createApp(App).mount('#app')
