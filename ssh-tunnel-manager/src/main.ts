import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'

// TDesign
import TDesign from 'tdesign-vue-next'
import 'tdesign-vue-next/dist/tdesign.min.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(TDesign)

app.mount('#app')
