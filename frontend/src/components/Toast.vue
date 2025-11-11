<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

export interface ToastMessage {
  id: string;
  type: "success" | "error" | "warning" | "info";
  title: string;
  message: string;
  duration?: number;
}

const toasts = ref<ToastMessage[]>([]);
let toastCounter = 0;

function addToast(toast: Omit<ToastMessage, "id">) {
  const id = `toast-${++toastCounter}`;
  const duration = toast.duration || 5000;

  const newToast: ToastMessage = {
    ...toast,
    id,
  };

  toasts.value.push(newToast);

  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }

  return id;
}

function removeToast(id: string) {
  const index = toasts.value.findIndex((t) => t.id === id);
  if (index !== -1) {
    toasts.value.splice(index, 1);
  }
}

function getIcon(type: string): string {
  switch (type) {
    case "success":
      return "✅";
    case "error":
      return "❌";
    case "warning":
      return "⚠️";
    case "info":
      return "ℹ️";
    default:
      return "📌";
  }
}

function getColorClass(type: string): string {
  switch (type) {
    case "success":
      return "toast-success";
    case "error":
      return "toast-error";
    case "warning":
      return "toast-warning";
    case "info":
      return "toast-info";
    default:
      return "toast-info";
  }
}

// Global API
declare global {
  interface Window {
    showToast: (toast: Omit<ToastMessage, "id">) => string;
  }
}

onMounted(() => {
  window.showToast = addToast;
});

onUnmounted(() => {
  delete window.showToast;
});

defineExpose({
  addToast,
  removeToast,
});
</script>

<template>
  <teleport to="body">
    <div class="toast-container">
      <transition-group name="toast" tag="div" class="toast-list">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="['toast', getColorClass(toast.type)]"
          @click="removeToast(toast.id)"
        >
          <div class="toast-icon">{{ getIcon(toast.type) }}</div>
          <div class="toast-content">
            <div class="toast-title">{{ toast.title }}</div>
            <div class="toast-message">{{ toast.message }}</div>
          </div>
          <button @click.stop="removeToast(toast.id)" class="toast-close">×</button>
        </div>
      </transition-group>
    </div>
  </teleport>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 9999;
  pointer-events: none;
}

.toast-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.toast {
  min-width: 320px;
  max-width: 400px;
  padding: 16px;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  display: flex;
  align-items: flex-start;
  gap: 12px;
  cursor: pointer;
  pointer-events: auto;
  transition: all 0.3s ease;
}

.toast:hover {
  transform: translateX(-4px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.toast-success {
  background: #d4edda;
  border-left: 4px solid #28a745;
}

.toast-error {
  background: #f8d7da;
  border-left: 4px solid #dc3545;
}

.toast-warning {
  background: #fff3cd;
  border-left: 4px solid #ffc107;
}

.toast-info {
  background: #d1ecf1;
  border-left: 4px solid #17a2b8;
}

.toast-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.toast-content {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-weight: 600;
  font-size: 14px;
  color: #333;
  margin-bottom: 4px;
}

.toast-message {
  font-size: 13px;
  color: #555;
  word-wrap: break-word;
}

.toast-close {
  background: transparent;
  border: none;
  font-size: 24px;
  line-height: 1;
  color: #666;
  cursor: pointer;
  padding: 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
  flex-shrink: 0;
}

.toast-close:hover {
  background: rgba(0, 0, 0, 0.1);
  color: #333;
}

/* Animations */
.toast-enter-active {
  animation: toast-in 0.3s ease;
}

.toast-leave-active {
  animation: toast-out 0.3s ease;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toast-out {
  from {
    opacity: 1;
    transform: translateX(0);
  }
  to {
    opacity: 0;
    transform: translateX(100%);
  }
}

@media (max-width: 768px) {
  .toast-container {
    top: 10px;
    right: 10px;
    left: 10px;
  }

  .toast {
    min-width: auto;
    max-width: none;
  }
}
</style>
