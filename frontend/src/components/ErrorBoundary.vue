<template>
  <div v-if="error" class="error-boundary">
    <div class="error-content">
      <h2>⚠️ Something Went Wrong</h2>
      <p class="error-message">{{ error.message }}</p>
      <details class="error-details" v-if="showDetails">
        <summary>Technical Details</summary>
        <pre>{{ error.stack }}</pre>
      </details>
      <div class="error-actions">
        <button @click="retry" class="btn-primary">Try Again</button>
        <button @click="reload" class="btn-secondary">Reload Page</button>
      </div>
    </div>
  </div>
  <slot v-else></slot>
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue'

interface Props {
  showDetails?: boolean
  onError?: (error: Error) => void
}

const props = withDefaults(defineProps<Props>(), {
  showDetails: true
})

const error = ref<Error | null>(null)

onErrorCaptured((err) => {
  error.value = err as Error
  props.onError?.(err as Error)
  console.error('ErrorBoundary caught:', err)
  return false // Prevent error from propagating
})

function retry() {
  error.value = null
}

function reload() {
  window.location.reload()
}
</script>

<style scoped>
.error-boundary {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  padding: 24px;
}

.error-content {
  max-width: 600px;
  text-align: center;
}

.error-content h2 {
  font-size: 24px;
  margin-bottom: 16px;
  color: #d32f2f;
}

.error-message {
  font-size: 16px;
  color: #666;
  margin-bottom: 20px;
}

.error-details {
  text-align: left;
  margin: 20px 0;
  padding: 16px;
  background: #f5f5f5;
  border-radius: 8px;
  border: 1px solid #e0e0e0;
}

.error-details summary {
  cursor: pointer;
  font-weight: 600;
  margin-bottom: 12px;
}

.error-details pre {
  font-size: 12px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

.error-actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.btn-primary,
.btn-secondary {
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  font-size: 14px;
  transition: all 0.2s;
}

.btn-primary {
  background: #1976d2;
  color: white;
}

.btn-primary:hover {
  background: #1565c0;
}

.btn-secondary {
  background: #f5f5f5;
  color: #333;
}

.btn-secondary:hover {
  background: #e0e0e0;
}

@media (prefers-color-scheme: dark) {
  .error-message {
    color: #aaa;
  }

  .error-details {
    background: #2a2a2a;
    border-color: #3a3a3a;
  }

  .btn-secondary {
    background: #3a3a3a;
    color: #fff;
  }

  .btn-secondary:hover {
    background: #4a4a4a;
  }
}
</style>
