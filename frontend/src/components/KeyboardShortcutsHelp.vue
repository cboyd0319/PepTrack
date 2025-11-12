<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="isOpen" class="shortcuts-overlay" @click="close">
        <div class="shortcuts-modal" @click.stop>
          <div class="modal-header">
            <h2>⌨️ Keyboard Shortcuts</h2>
            <button @click="close" class="close-btn">✕</button>
          </div>

          <div class="shortcuts-content">
            <div v-for="section in shortcutSections" :key="section.title" class="shortcuts-section">
              <h3>{{ section.title }}</h3>
              <div class="shortcuts-list">
                <div
                  v-for="shortcut in section.shortcuts"
                  :key="shortcut.keys"
                  class="shortcut-item"
                >
                  <div class="shortcut-keys">
                    <kbd v-for="key in shortcut.keys.split('+')" :key="key" class="key">
                      {{ key.trim() }}
                    </kbd>
                  </div>
                  <div class="shortcut-description">{{ shortcut.description }}</div>
                </div>
              </div>
            </div>
          </div>

          <div class="modal-footer">
            <p class="footer-hint">
              Press <kbd class="key">?</kbd> anytime to view this help panel
            </p>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';

interface Shortcut {
  keys: string;
  description: string;
}

interface ShortcutSection {
  title: string;
  shortcuts: Shortcut[];
}

const isOpen = ref(false);

const shortcutSections: ShortcutSection[] = [
  {
    title: 'Navigation',
    shortcuts: [
      { keys: 'Cmd + K', description: 'Open global search' },
      { keys: '1', description: 'Go to Dashboard' },
      { keys: '2', description: 'Go to Doses' },
      { keys: '3', description: 'Go to Protocols' },
      { keys: '4', description: 'Go to AI Assistant' },
      { keys: '5', description: 'Go to Research' },
      { keys: '6', description: 'Go to Operations' },
      { keys: '7', description: 'Go to Settings' },
      { keys: '8', description: 'Go to Alerts' },
    ],
  },
  {
    title: 'Quick Actions',
    shortcuts: [
      { keys: 'N', description: 'Create new protocol' },
      { keys: 'D', description: 'Log new dose' },
      { keys: 'I', description: 'Add inventory' },
      { keys: 'S', description: 'Add supplier' },
      { keys: 'B', description: 'Quick backup' },
    ],
  },
  {
    title: 'General',
    shortcuts: [
      { keys: '?', description: 'Show keyboard shortcuts' },
      { keys: 'Esc', description: 'Close modals/dialogs' },
      { keys: 'Cmd + S', description: 'Save (where applicable)' },
      { keys: 'Cmd + P', description: 'Print/Export' },
      { keys: 'Cmd + /', description: 'Toggle help mode' },
    ],
  },
  {
    title: 'Search & Filter',
    shortcuts: [
      { keys: '/', description: 'Focus search field' },
      { keys: 'Cmd + F', description: 'Find in page' },
      { keys: 'Cmd + Shift + F', description: 'Advanced search' },
      { keys: 'Alt + F', description: 'Open filters' },
    ],
  },
];

function open() {
  isOpen.value = true;
}

function close() {
  isOpen.value = false;
}

function handleKeyDown(e: KeyboardEvent) {
  // Open help with '?' key
  if (e.key === '?' && !e.metaKey && !e.ctrlKey && !e.altKey) {
    const target = e.target as HTMLElement;
    // Don't trigger if typing in input/textarea
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
      return;
    }
    e.preventDefault();
    open();
  }

  // Close with Escape
  if (e.key === 'Escape' && isOpen.value) {
    e.preventDefault();
    close();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

defineExpose({ open, close });
</script>

<style scoped>
.shortcuts-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  padding: 20px;
}

.shortcuts-modal {
  background: white;
  border-radius: 16px;
  max-width: 900px;
  width: 100%;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 32px;
  border-bottom: 2px solid #e0e0e0;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.modal-header h2 {
  font-size: 28px;
  font-weight: 700;
  margin: 0;
}

.close-btn {
  width: 36px;
  height: 36px;
  background: rgba(255, 255, 255, 0.2);
  border: none;
  border-radius: 50%;
  font-size: 24px;
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: scale(1.1);
}

.shortcuts-content {
  flex: 1;
  overflow-y: auto;
  padding: 32px;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 32px;
}

.shortcuts-section h3 {
  font-size: 18px;
  font-weight: 600;
  color: #1a1a1a;
  margin: 0 0 16px 0;
  padding-bottom: 8px;
  border-bottom: 2px solid #e0e0e0;
}

.shortcuts-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 12px;
  background: #f9f9f9;
  border-radius: 8px;
  transition: all 0.2s;
}

.shortcut-item:hover {
  background: #f0f0f0;
  transform: translateX(4px);
}

.shortcut-keys {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.key {
  display: inline-block;
  padding: 4px 10px;
  background: white;
  border: 2px solid #d0d0d0;
  border-bottom-width: 4px;
  border-radius: 6px;
  font-family: 'SF Mono', 'Monaco', 'Cascadia Code', 'Courier New', monospace;
  font-size: 13px;
  font-weight: 600;
  color: #333;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  min-width: 32px;
  text-align: center;
}

.shortcut-description {
  font-size: 14px;
  color: #666;
  text-align: right;
  flex: 1;
}

.modal-footer {
  padding: 16px 32px;
  background: #f9f9f9;
  border-top: 2px solid #e0e0e0;
  text-align: center;
}

.footer-hint {
  font-size: 13px;
  color: #666;
  margin: 0;
}

/* Animations */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .shortcuts-modal,
.modal-leave-active .shortcuts-modal {
  transition: transform 0.3s ease;
}

.modal-enter-from .shortcuts-modal,
.modal-leave-to .shortcuts-modal {
  transform: scale(0.9);
}

/* Dark mode */
:global(.dark-mode) .shortcuts-modal {
  background: #2a2a2a;
}

:global(.dark-mode) .modal-header {
  border-bottom-color: #404040;
}

:global(.dark-mode) .shortcuts-section h3 {
  color: #fff;
  border-bottom-color: #404040;
}

:global(.dark-mode) .shortcut-item {
  background: #1a1a1a;
}

:global(.dark-mode) .shortcut-item:hover {
  background: #333;
}

:global(.dark-mode) .key {
  background: #333;
  border-color: #555;
  color: #fff;
}

:global(.dark-mode) .shortcut-description {
  color: #aaa;
}

:global(.dark-mode) .modal-footer {
  background: #1a1a1a;
  border-top-color: #404040;
}

/* Responsive */
@media (max-width: 768px) {
  .shortcuts-overlay {
    padding: 0;
  }

  .shortcuts-modal {
    max-height: 100vh;
    border-radius: 0;
  }

  .shortcuts-content {
    grid-template-columns: 1fr;
    padding: 20px;
    gap: 24px;
  }

  .shortcut-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .shortcut-description {
    text-align: left;
  }
}

/* Print styles */
@media print {
  .shortcuts-overlay {
    position: static;
    background: none;
  }

  .shortcuts-modal {
    box-shadow: none;
    max-height: none;
  }

  .close-btn {
    display: none;
  }
}
</style>
