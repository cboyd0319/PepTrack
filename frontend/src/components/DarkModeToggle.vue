<template>
  <div class="dark-mode-toggle">
    <h3>🌙 Dark Mode</h3>
    <p class="section-subtitle">Choose your preferred color scheme</p>

    <div class="theme-options panel">
      <label class="theme-option" :class="{ active: themeMode === 'light' }">
        <input
          type="radio"
          name="theme"
          value="light"
          v-model="currentTheme"
          @change="handleThemeChange"
        />
        <div class="option-content">
          <span class="option-icon">☀️</span>
          <div class="option-text">
            <strong>Light</strong>
            <span class="hint">Always use light mode</span>
          </div>
        </div>
      </label>

      <label class="theme-option" :class="{ active: themeMode === 'dark' }">
        <input
          type="radio"
          name="theme"
          value="dark"
          v-model="currentTheme"
          @change="handleThemeChange"
        />
        <div class="option-content">
          <span class="option-icon">🌙</span>
          <div class="option-text">
            <strong>Dark</strong>
            <span class="hint">Always use dark mode</span>
          </div>
        </div>
      </label>

      <label class="theme-option" :class="{ active: themeMode === 'auto' }">
        <input
          type="radio"
          name="theme"
          value="auto"
          v-model="currentTheme"
          @change="handleThemeChange"
        />
        <div class="option-content">
          <span class="option-icon">🔄</span>
          <div class="option-text">
            <strong>Auto</strong>
            <span class="hint">Match system preference</span>
          </div>
        </div>
      </label>
    </div>

    <div class="current-theme-info">
      <div class="info-box">
        <span class="info-label">Current theme:</span>
        <span class="info-value">{{ isDark ? '🌙 Dark' : '☀️ Light' }}</span>
      </div>
      <div v-if="themeMode === 'auto'" class="auto-note">
        Theme is automatically matching your system preference
      </div>
    </div>

    <!-- Theme Preview -->
    <div class="theme-preview panel">
      <h4>Preview</h4>
      <div class="preview-card" :class="{ 'preview-dark': isDark }">
        <div class="preview-header">
          <div class="preview-title">Sample Card</div>
          <button class="preview-button">Action</button>
        </div>
        <div class="preview-body">
          <p>This is how your interface will look with the selected theme.</p>
          <div class="preview-stats">
            <div class="preview-stat">
              <span class="stat-label">Protocols</span>
              <span class="stat-value">12</span>
            </div>
            <div class="preview-stat">
              <span class="stat-label">Doses</span>
              <span class="stat-value">48</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { themeMode, isDark, setThemeMode, type ThemeMode } from '../utils/darkMode';

const currentTheme = ref<ThemeMode>(themeMode.value);

watch(themeMode, (newMode) => {
  currentTheme.value = newMode;
});

function handleThemeChange() {
  setThemeMode(currentTheme.value);
}
</script>

<style scoped>
.dark-mode-toggle {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
}

.dark-mode-toggle h3 {
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

/* Theme Options */
.theme-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
}

.theme-option {
  display: flex;
  align-items: center;
  padding: 16px;
  background: #f9f9f9;
  border: 2px solid #e0e0e0;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.theme-option:hover {
  background: #f0f0f0;
  border-color: #1976d2;
}

.theme-option.active {
  background: #e3f2fd;
  border-color: #1976d2;
  box-shadow: 0 2px 8px rgba(25, 118, 210, 0.2);
}

.theme-option input[type='radio'] {
  margin-right: 16px;
  width: 20px;
  height: 20px;
  accent-color: #1976d2;
  cursor: pointer;
}

.option-content {
  display: flex;
  align-items: center;
  gap: 16px;
  flex: 1;
}

.option-icon {
  font-size: 32px;
}

.option-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.option-text strong {
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.hint {
  font-size: 13px;
  color: #666;
}

/* Current Theme Info */
.current-theme-info {
  margin-bottom: 24px;
}

.info-box {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: #f0f7ff;
  border: 2px solid #bbdefb;
  border-radius: 8px;
  margin-bottom: 12px;
}

.info-label {
  font-weight: 600;
  color: #1976d2;
}

.info-value {
  font-weight: 700;
  color: #1565c0;
  font-size: 16px;
}

.auto-note {
  font-size: 13px;
  color: #666;
  font-style: italic;
  padding-left: 16px;
}

/* Theme Preview */
.theme-preview h4 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: #1a1a1a;
}

.preview-card {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 10px;
  overflow: hidden;
  transition: all 0.3s;
}

.preview-card.preview-dark {
  background: #2a2a2a;
  border-color: #404040;
  color: #fff;
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: #f9f9f9;
  border-bottom: 1px solid #e0e0e0;
}

.preview-dark .preview-header {
  background: #1a1a1a;
  border-bottom-color: #404040;
}

.preview-title {
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.preview-dark .preview-title {
  color: #fff;
}

.preview-button {
  padding: 6px 16px;
  background: #1976d2;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.preview-body {
  padding: 20px;
}

.preview-body p {
  margin: 0 0 16px 0;
  color: #333;
}

.preview-dark .preview-body p {
  color: #ccc;
}

.preview-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.preview-stat {
  padding: 12px;
  background: #f0f0f0;
  border-radius: 8px;
  text-align: center;
}

.preview-dark .preview-stat {
  background: #1a1a1a;
}

.stat-label {
  display: block;
  font-size: 12px;
  color: #666;
  margin-bottom: 4px;
}

.preview-dark .stat-label {
  color: #999;
}

.stat-value {
  display: block;
  font-size: 24px;
  font-weight: 700;
  color: #1976d2;
}

.preview-dark .stat-value {
  color: #64b5f6;
}

/* Dark mode styles for the component itself */
:global(.dark-mode) .dark-mode-toggle h3,
:global(.dark-mode) .theme-preview h4 {
  color: #fff;
}

:global(.dark-mode) .section-subtitle,
:global(.dark-mode) .hint,
:global(.dark-mode) .auto-note {
  color: #aaa;
}

:global(.dark-mode) .panel {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .theme-option {
  background: #1a1a1a;
  border-color: #404040;
}

:global(.dark-mode) .theme-option:hover {
  background: #333;
  border-color: #64b5f6;
}

:global(.dark-mode) .theme-option.active {
  background: #1e3a5f;
  border-color: #64b5f6;
  box-shadow: 0 2px 8px rgba(100, 181, 246, 0.3);
}

:global(.dark-mode) .option-text strong {
  color: #fff;
}

:global(.dark-mode) .info-box {
  background: #1e3a5f;
  border-color: #1565c0;
}

:global(.dark-mode) .info-label {
  color: #64b5f6;
}

:global(.dark-mode) .info-value {
  color: #90caf9;
}

/* Responsive */
@media (max-width: 768px) {
  .dark-mode-toggle {
    padding: 16px;
  }

  .theme-options {
    padding: 12px;
  }

  .option-icon {
    font-size: 24px;
  }

  .preview-stats {
    grid-template-columns: 1fr;
  }
}
</style>
