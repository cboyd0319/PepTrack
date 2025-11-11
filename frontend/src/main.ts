import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

const app = createApp(App);

// Global error handler for better error management
app.config.errorHandler = (err, instance, info) => {
  console.error('Global error caught:', err);
  console.error('Component:', instance);
  console.error('Error info:', info);
};

// Warn handler for development
app.config.warnHandler = (msg, instance, trace) => {
  console.warn('Vue warning:', msg);
  console.warn('Component trace:', trace);
};

app.mount('#app');
