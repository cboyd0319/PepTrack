<template>
  <div class="widget-settings">
    <h3>📊 Dashboard Widgets</h3>
    <p class="section-subtitle">Customize your dashboard by enabling, disabling, and rearranging widgets</p>

    <div class="settings-actions panel">
      <button @click="handleResetWidgets" class="reset-btn">
        🔄 Reset to Defaults
      </button>
      <p class="hint">Drag widgets to reorder, or use ↑↓ buttons</p>
    </div>

    <div class="widgets-list panel">
      <div v-for="(widget, index) in widgets" :key="widget.id" class="widget-item">
        <div class="widget-header">
          <label class="widget-toggle">
            <input
              type="checkbox"
              :checked="widget.enabled"
              @change="handleToggleWidget(widget.id)"
            />
            <span class="widget-title">{{ widget.title }}</span>
          </label>

          <div class="widget-controls">
            <!-- Size selector -->
            <select
              :value="widget.size"
              @change="handleChangeSize(widget.id, ($event.target as HTMLSelectElement).value as 'small' | 'medium' | 'large')"
              class="size-select"
              :disabled="!widget.enabled"
            >
              <option value="small">Small</option>
              <option value="medium">Medium</option>
              <option value="large">Large</option>
            </select>

            <!-- Move up/down buttons -->
            <button
              @click="handleMoveUp(widget.id)"
              :disabled="index === 0"
              class="move-btn"
              title="Move up"
            >
              ↑
            </button>
            <button
              @click="handleMoveDown(widget.id)"
              :disabled="index === widgets.length - 1"
              class="move-btn"
              title="Move down"
            >
              ↓
            </button>
          </div>
        </div>

        <div v-if="widget.enabled" class="widget-preview">
          <div class="preview-box" :class="`preview-${widget.size}`">
            <span class="preview-label">{{ getWidgetIcon(widget.type) }} {{ widget.title }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="preview-dashboard panel">
      <h4>📱 Preview</h4>
      <p class="hint">This is how your dashboard will look with current settings</p>
      <div class="dashboard-grid">
        <div
          v-for="widget in enabledWidgets"
          :key="widget.id"
          class="preview-widget"
          :class="`widget-${widget.size}`"
        >
          <div class="preview-widget-content">
            <span class="widget-icon">{{ getWidgetIcon(widget.type) }}</span>
            <span class="widget-name">{{ widget.title }}</span>
          </div>
        </div>
      </div>
      <div v-if="enabledWidgets.length === 0" class="no-widgets">
        <p>No widgets enabled. Enable at least one widget above.</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  widgetConfigs,
  toggleWidget,
  moveWidgetUp,
  moveWidgetDown,
  changeWidgetSize,
  resetWidgets,
  getEnabledWidgets,
  type WidgetType,
} from '../utils/widgets';
import { showSuccessToast } from '../utils/errorHandling';

const widgets = widgetConfigs;
const enabledWidgets = computed(() => getEnabledWidgets());

function handleToggleWidget(widgetId: string) {
  toggleWidget(widgetId);
  showSuccessToast('Widget Updated', 'Widget settings have been updated');
}

function handleMoveUp(widgetId: string) {
  moveWidgetUp(widgetId);
  showSuccessToast('Widget Moved', 'Widget order has been updated');
}

function handleMoveDown(widgetId: string) {
  moveWidgetDown(widgetId);
  showSuccessToast('Widget Moved', 'Widget order has been updated');
}

function handleChangeSize(widgetId: string, size: 'small' | 'medium' | 'large') {
  changeWidgetSize(widgetId, size);
  showSuccessToast('Widget Resized', 'Widget size has been updated');
}

function handleResetWidgets() {
  if (confirm('Are you sure you want to reset all widgets to default settings?')) {
    resetWidgets();
    showSuccessToast('Widgets Reset', 'All widgets have been reset to defaults');
  }
}

function getWidgetIcon(type: WidgetType): string {
  const icons: Record<WidgetType, string> = {
    'quick-stats': '📊',
    'recent-doses': '💉',
    'protocols': '📋',
    'inventory': '📦',
    'alerts': '🔔',
    'price-trends': '📈',
  };
  return icons[type] || '📊';
}
</script>

<style scoped>
.widget-settings {
  padding: 24px;
  max-width: 1000px;
  margin: 0 auto;
}

.widget-settings h3 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.section-subtitle {
  font-size: 15px;
  color: #666;
  margin: 0 0 24px 0;
}

.panel {
  background: white;
  padding: 24px;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  margin-bottom: 24px;
}

/* Settings Actions */
.settings-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
}

.reset-btn {
  padding: 10px 20px;
  background: #f44336;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.reset-btn:hover {
  background: #d32f2f;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(244, 67, 54, 0.3);
}

.hint {
  font-size: 13px;
  color: #999;
  margin: 0;
}

/* Widgets List */
.widgets-list {
  padding: 16px;
}

.widget-item {
  padding: 16px;
  background: #f9f9f9;
  border: 2px solid #e0e0e0;
  border-radius: 10px;
  margin-bottom: 12px;
  transition: all 0.2s;
}

.widget-item:hover {
  border-color: #1976d2;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.widget-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.widget-toggle {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  flex: 1;
}

.widget-toggle input[type='checkbox'] {
  width: 20px;
  height: 20px;
  accent-color: #1976d2;
  cursor: pointer;
}

.widget-title {
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.widget-controls {
  display: flex;
  align-items: center;
  gap: 8px;
}

.size-select {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 13px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.size-select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.size-select:hover:not(:disabled) {
  border-color: #1976d2;
}

.move-btn {
  width: 32px;
  height: 32px;
  background: white;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.move-btn:hover:not(:disabled) {
  background: #1976d2;
  color: white;
  border-color: #1976d2;
}

.move-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

/* Widget Preview */
.widget-preview {
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px solid #e0e0e0;
}

.preview-box {
  padding: 16px;
  background: white;
  border: 2px dashed #1976d2;
  border-radius: 8px;
  text-align: center;
  color: #1976d2;
  font-weight: 600;
}

.preview-box.preview-small {
  max-width: 200px;
}

.preview-box.preview-medium {
  max-width: 400px;
}

.preview-box.preview-large {
  max-width: 100%;
}

.preview-label {
  font-size: 14px;
}

/* Preview Dashboard */
.preview-dashboard h4 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 16px;
  margin-top: 16px;
}

.preview-widget {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 10px;
  padding: 20px;
  color: white;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s;
}

.preview-widget:hover {
  transform: translateY(-4px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

.preview-widget.widget-small {
  grid-column: span 1;
  min-height: 100px;
}

.preview-widget.widget-medium {
  grid-column: span 1;
  min-height: 150px;
}

.preview-widget.widget-large {
  grid-column: span 2;
  min-height: 200px;
}

.preview-widget-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.widget-icon {
  font-size: 32px;
}

.widget-name {
  font-size: 14px;
  font-weight: 600;
  text-align: center;
}

.no-widgets {
  padding: 40px;
  text-align: center;
  color: #999;
  font-style: italic;
}

/* Dark Mode */
:global(.dark-mode) .widget-settings h3,
:global(.dark-mode) .preview-dashboard h4 {
  color: #fff;
}

:global(.dark-mode) .section-subtitle,
:global(.dark-mode) .hint {
  color: #aaa;
}

:global(.dark-mode) .panel {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .widget-item {
  background: #1a1a1a;
  border-color: #404040;
}

:global(.dark-mode) .widget-item:hover {
  border-color: #64b5f6;
}

:global(.dark-mode) .widget-title {
  color: #fff;
}

:global(.dark-mode) .size-select,
:global(.dark-mode) .move-btn {
  background: #1a1a1a;
  border-color: #404040;
  color: #fff;
}

:global(.dark-mode) .size-select:hover:not(:disabled),
:global(.dark-mode) .move-btn:hover:not(:disabled) {
  border-color: #64b5f6;
  background: #1976d2;
}

:global(.dark-mode) .preview-box {
  background: #1a1a1a;
  border-color: #64b5f6;
  color: #64b5f6;
}

/* Responsive */
@media (max-width: 768px) {
  .widget-settings {
    padding: 16px;
  }

  .settings-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .widget-header {
    flex-direction: column;
    align-items: stretch;
  }

  .widget-controls {
    justify-content: space-between;
  }

  .dashboard-grid {
    grid-template-columns: 1fr;
  }

  .preview-widget.widget-large {
    grid-column: span 1;
  }
}
</style>
