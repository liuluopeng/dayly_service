import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import i18n from "./i18n";
import "./index.css";

const app = createApp(App);

app.config.errorHandler = (err, _instance, info) => {
  console.error('[Vue Error]', info, err);
};

app.use(createPinia());
app.use(router);
app.use(i18n as any);
app.mount("#app");

