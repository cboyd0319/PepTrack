<template>
  <div class="quick-actions-fab">
    <!-- FAB Button -->
    <button
      class="fab-main"
      :class="{ active: isExpanded }"
      @click="toggleExpanded"
      title="Quick Actions"
    >
      <span class="fab-icon">{{ isExpanded ? '✕' : '⚡' }}</span>
    </button>

    <!-- Action Buttons -->
    <Transition name="fab-actions">
      <div v-if="isExpanded" class="fab-actions">
        <button
          v-for="(action, index) in actions"
          :key="action.id"
          class="fab-action"
          :style="{ transitionDelay: `${index * 30}ms` }"
          @click="handleAction(action.id)"
          :title="action.label"
        >
          <span class="action-icon">{{ action.icon }}</span>
          <span class="action-label">{{ action.label }}</span>
        </button>
      </div>
    </Transition>

    <!-- Backdrop -->
    <Transition name="fade">
      <div v-if="isExpanded" class="fab-backdrop" @click="toggleExpanded"></div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface QuickAction {
  id: string;
  label: string;
  icon: string;
}

const emit = defineEmits<{
  (e: 'navigate', view: string): void;
  (e: 'openSearch'): void;
  (e: 'quickAction', actionId: string): void;
}>();

const isExpanded = ref(false);

const actions: QuickAction[] = [
  { id: 'log-dose', label: 'Log Dose', icon: '💉' },
  { id: 'add-inventory', label: 'Add Inventory', icon: '📦' },
  { id: 'create-protocol', label: 'Create Protocol', icon: '📋' },
  { id: 'add-supplier', label: 'Add Supplier', icon: '🏭' },
  { id: 'global-search', label: 'Search (Cmd+K)', icon: '🔍' },
  { id: 'ai-assistant', label: 'AI Assistant', icon: '🤖' },
];

function toggleExpanded() {
  isExpanded.value = !isExpanded.value;
}

function handleAction(actionId: string) {
  isExpanded.value = false;

  switch (actionId) {
    case 'log-dose':
      emit('navigate', 'doses');
      break;
    case 'add-inventory':
    case 'add-supplier':
      emit('navigate', 'operations');
      break;
    case 'create-protocol':
      emit('navigate', 'protocols');
      break;
    case 'global-search':
      emit('openSearch');
      break;
    case 'ai-assistant':
      emit('navigate', 'ai-assistant');
      break;
    default:
      emit('quickAction', actionId);
  }
}
</script>

<style scoped>
.quick-actions-fab {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 1000;
}

/* Main FAB Button */
.fab-main {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: linear-gradient(135deg, #1976d2 0%, #1565c0 100%);
  border: none;
  color: white;
  font-size: 32px;
  cursor: pointer;
  box-shadow: 0 6px 20px rgba(25, 118, 210, 0.4);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  z-index: 1002;
  display: flex;
  align-items: center;
  justify-content: center;
}

.fab-main:hover {
  transform: scale(1.1);
  box-shadow: 0 8px 24px rgba(25, 118, 210, 0.5);
  background: linear-gradient(135deg, #1565c0 0%, #0d47a1 100%);
}

.fab-main:active {
  transform: scale(0.95);
}

.fab-main.active {
  background: linear-gradient(135deg, #d32f2f 0%, #c62828 100%);
  transform: rotate(90deg);
}

.fab-icon {
  transition: transform 0.3s;
}

/* Action Buttons */
.fab-actions {
  position: absolute;
  bottom: 80px;
  right: 0;
  display: flex;
  flex-direction: column-reverse;
  gap: 12px;
  z-index: 1001;
}

.fab-action {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 28px;
  color: #1a1a1a;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  white-space: nowrap;
  transform-origin: right center;
  animation: slideInRight 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.fab-action:hover {
  background: #1976d2;
  color: white;
  border-color: #1976d2;
  transform: translateX(-4px);
  box-shadow: 0 6px 16px rgba(25, 118, 210, 0.3);
}

.fab-action:active {
  transform: translateX(-4px) scale(0.95);
}

.action-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.action-label {
  font-size: 14px;
}

/* Backdrop */
.fab-backdrop {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.3);
  z-index: 999;
  cursor: pointer;
}

/* Animations */
@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(20px) scale(0.8);
  }
  to {
    opacity: 1;
    transform: translateX(0) scale(1);
  }
}

.fab-actions-enter-active {
  transition: all 0.3s ease;
}

.fab-actions-leave-active {
  transition: all 0.2s ease;
}

.fab-actions-enter-from,
.fab-actions-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

.fade-enter-active {
  transition: opacity 0.3s;
}

.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

/* Dark mode */
:global(.dark-mode) .fab-action {
  background: #2a2a2a;
  border-color: #404040;
  color: #fff;
}

:global(.dark-mode) .fab-action:hover {
  background: #1976d2;
  border-color: #1976d2;
  color: #fff;
}

/* Responsive */
@media (max-width: 768px) {
  .quick-actions-fab {
    bottom: 16px;
    right: 16px;
  }

  .fab-main {
    width: 56px;
    height: 56px;
    font-size: 28px;
  }

  .fab-actions {
    bottom: 72px;
  }

  .fab-action {
    padding: 10px 16px;
    font-size: 14px;
  }

  .action-icon {
    font-size: 20px;
  }

  .action-label {
    font-size: 13px;
  }
}

@media (max-width: 480px) {
  /* On very small screens, only show icons */
  .action-label {
    display: none;
  }

  .fab-action {
    padding: 12px;
    border-radius: 50%;
    width: 48px;
    height: 48px;
    justify-content: center;
  }
}

/* Accessibility */
.fab-main:focus-visible,
.fab-action:focus-visible {
  outline: 3px solid #1976d2;
  outline-offset: 2px;
}

/* Prevent FAB from interfering with scrolling */
@media (max-height: 600px) {
  .quick-actions-fab {
    bottom: 12px;
    right: 12px;
  }

  .fab-main {
    width: 52px;
    height: 52px;
    font-size: 24px;
  }
}
</style>
