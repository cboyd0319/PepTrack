<template>
  <div class="empty-state" :class="`variant-${variant}`">
    <div class="empty-illustration">
      <div v-if="type === 'custom' && customIcon" class="custom-icon">
        {{ customIcon }}
      </div>
      <svg v-else class="illustration-svg" :class="`illustration-${type}`" viewBox="0 0 200 200">
        <!-- No Data -->
        <g v-if="type === 'no-data'">
          <circle cx="100" cy="100" r="60" fill="#f0f0f0" />
          <path d="M70 80 L130 80 L130 120 L70 120 Z" fill="#e0e0e0" />
          <circle cx="100" cy="100" r="20" fill="#d0d0d0" />
          <line x1="80" y1="90" x2="120" y2="110" stroke="#999" stroke-width="4" stroke-linecap="round" />
          <line x1="120" y1="90" x2="80" y2="110" stroke="#999" stroke-width="4" stroke-linecap="round" />
        </g>

        <!-- Empty Doses -->
        <g v-if="type === 'doses'">
          <rect x="70" y="60" width="60" height="80" rx="30" fill="#e3f2fd" />
          <circle cx="100" cy="70" r="8" fill="#2196f3" />
          <line x1="100" y1="78" x2="100" y2="130" stroke="#2196f3" stroke-width="3" />
          <polygon points="100,130 95,120 105,120" fill="#2196f3" />
        </g>

        <!-- Empty Protocols -->
        <g v-if="type === 'protocols'">
          <rect x="60" y="50" width="80" height="100" rx="8" fill="#e8f5e9" stroke="#4caf50" stroke-width="3" />
          <line x1="70" y1="70" x2="130" y2="70" stroke="#4caf50" stroke-width="2" />
          <line x1="70" y1="90" x2="120" y2="90" stroke="#4caf50" stroke-width="2" opacity="0.5" />
          <line x1="70" y1="110" x2="110" y2="110" stroke="#4caf50" stroke-width="2" opacity="0.3" />
        </g>

        <!-- Empty Inventory -->
        <g v-if="type === 'inventory'">
          <rect x="60" y="80" width="80" height="60" rx="4" fill="#fff3e0" stroke="#ff9800" stroke-width="3" />
          <rect x="70" y="60" width="60" height="20" rx="2" fill="#ff9800" />
          <circle cx="80" cy="105" r="8" fill="#ff9800" opacity="0.5" />
          <circle cx="100" cy="105" r="8" fill="#ff9800" opacity="0.5" />
          <circle cx="120" cy="105" r="8" fill="#ff9800" opacity="0.5" />
        </g>

        <!-- Empty Search -->
        <g v-if="type === 'search'">
          <circle cx="90" cy="90" r="35" fill="none" stroke="#9e9e9e" stroke-width="4" />
          <line x1="115" y1="115" x2="140" y2="140" stroke="#9e9e9e" stroke-width="4" stroke-linecap="round" />
          <line x1="70" y1="80" x2="110" y2="80" stroke="#e0e0e0" stroke-width="3" />
          <line x1="75" y1="95" x2="105" y2="95" stroke="#e0e0e0" stroke-width="3" />
        </g>

        <!-- Error State -->
        <g v-if="type === 'error'">
          <circle cx="100" cy="100" r="50" fill="#ffebee" stroke="#f44336" stroke-width="3" />
          <line x1="80" y1="80" x2="120" y2="120" stroke="#f44336" stroke-width="4" stroke-linecap="round" />
          <line x1="120" y1="80" x2="80" y2="120" stroke="#f44336" stroke-width="4" stroke-linecap="round" />
        </g>

        <!-- Success State -->
        <g v-if="type === 'success'">
          <circle cx="100" cy="100" r="50" fill="#e8f5e9" stroke="#4caf50" stroke-width="3" />
          <polyline points="75,100 90,115 125,80" fill="none" stroke="#4caf50" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
        </g>
      </svg>
    </div>

    <div class="empty-content">
      <h3 v-if="title" class="empty-title">{{ title }}</h3>
      <p v-if="description" class="empty-description">{{ description }}</p>
      <div v-if="$slots.default" class="empty-slot">
        <slot></slot>
      </div>
    </div>

    <div v-if="actionLabel && actionHandler" class="empty-action">
      <button @click="actionHandler" class="action-button">
        <span v-if="actionIcon" class="action-icon">{{ actionIcon }}</span>
        {{ actionLabel }}
      </button>
    </div>

    <div v-if="helpText" class="empty-help">
      <p class="help-text">{{ helpText }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
export type EmptyStateType =
  | 'no-data'
  | 'doses'
  | 'protocols'
  | 'inventory'
  | 'search'
  | 'error'
  | 'success'
  | 'custom';

export type EmptyStateVariant = 'default' | 'compact' | 'large';

interface Props {
  type?: EmptyStateType;
  variant?: EmptyStateVariant;
  title?: string;
  description?: string;
  actionLabel?: string;
  actionIcon?: string;
  actionHandler?: () => void;
  helpText?: string;
  customIcon?: string;
}

withDefaults(defineProps<Props>(), {
  type: 'no-data',
  variant: 'default',
});
</script>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  text-align: center;
  animation: fadeIn 0.5s ease;
}

.variant-compact {
  padding: 40px 20px;
}

.variant-large {
  padding: 100px 20px;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Illustration */
.empty-illustration {
  margin-bottom: 24px;
}

.illustration-svg {
  width: 150px;
  height: 150px;
  filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.08));
  animation: float 3s ease-in-out infinite;
}

.variant-compact .illustration-svg {
  width: 100px;
  height: 100px;
}

.variant-large .illustration-svg {
  width: 200px;
  height: 200px;
}

@keyframes float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.custom-icon {
  font-size: 80px;
  animation: float 3s ease-in-out infinite;
}

.variant-compact .custom-icon {
  font-size: 60px;
}

.variant-large .custom-icon {
  font-size: 120px;
}

/* Content */
.empty-content {
  max-width: 500px;
  margin-bottom: 24px;
}

.empty-title {
  font-size: 24px;
  font-weight: 700;
  color: #1a1a1a;
  margin: 0 0 12px 0;
}

.variant-compact .empty-title {
  font-size: 20px;
}

.variant-large .empty-title {
  font-size: 32px;
}

.empty-description {
  font-size: 16px;
  color: #666;
  margin: 0;
  line-height: 1.6;
}

.variant-compact .empty-description {
  font-size: 14px;
}

.variant-large .empty-description {
  font-size: 18px;
}

.empty-slot {
  margin-top: 16px;
}

/* Action */
.empty-action {
  margin-bottom: 16px;
}

.action-button {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 28px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.3s;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.action-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(102, 126, 234, 0.4);
}

.action-button:active {
  transform: translateY(0);
}

.action-icon {
  font-size: 20px;
}

/* Help Text */
.empty-help {
  max-width: 400px;
}

.help-text {
  font-size: 13px;
  color: #999;
  margin: 0;
  font-style: italic;
}

/* Dark mode */
:global(.dark-mode) .empty-title {
  color: #fff;
}

:global(.dark-mode) .empty-description {
  color: #aaa;
}

:global(.dark-mode) .help-text {
  color: #777;
}

:global(.dark-mode) .illustration-svg .illustration-no-data circle,
:global(.dark-mode) .illustration-svg .illustration-no-data path,
:global(.dark-mode) .illustration-svg .illustration-no-data circle:nth-child(3) {
  fill: #404040;
}

:global(.dark-mode) .illustration-svg .illustration-doses rect {
  fill: #1e3a5f;
}

:global(.dark-mode) .illustration-svg .illustration-protocols rect {
  fill: #1a3a1a;
}

:global(.dark-mode) .illustration-svg .illustration-inventory rect:first-child {
  fill: #3a2a1a;
}

/* Responsive */
@media (max-width: 768px) {
  .empty-state {
    padding: 40px 16px;
  }

  .variant-large {
    padding: 60px 16px;
  }

  .empty-title {
    font-size: 20px;
  }

  .empty-description {
    font-size: 14px;
  }

  .illustration-svg {
    width: 120px;
    height: 120px;
  }
}

/* Specific type styles */
.illustration-doses {
  filter: drop-shadow(0 4px 12px rgba(33, 150, 243, 0.2));
}

.illustration-protocols {
  filter: drop-shadow(0 4px 12px rgba(76, 175, 80, 0.2));
}

.illustration-inventory {
  filter: drop-shadow(0 4px 12px rgba(255, 152, 0, 0.2));
}

.illustration-error {
  filter: drop-shadow(0 4px 12px rgba(244, 67, 54, 0.2));
}

.illustration-success {
  filter: drop-shadow(0 4px 12px rgba(76, 175, 80, 0.2));
  animation: successPulse 0.6s ease;
}

@keyframes successPulse {
  0% {
    transform: scale(0.8);
    opacity: 0;
  }
  50% {
    transform: scale(1.1);
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>
