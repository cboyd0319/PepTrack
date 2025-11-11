<template>
  <div class="skeleton-loader" :class="variant">
    <div class="skeleton-item" :style="itemStyle" v-for="i in count" :key="i"></div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface Props {
  variant?: 'list' | 'card' | 'text' | 'circle'
  count?: number
  height?: string
  width?: string
}

const props = withDefaults(defineProps<Props>(), {
  variant: 'list',
  count: 3,
  height: '60px',
  width: '100%'
})

const itemStyle = computed(() => ({
  height: props.height,
  width: props.width
}))
</script>

<style scoped>
.skeleton-loader {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px;
}

.skeleton-item {
  background: linear-gradient(
    90deg,
    #f0f0f0 25%,
    #e0e0e0 50%,
    #f0f0f0 75%
  );
  background-size: 200% 100%;
  animation: loading 1.5s infinite;
  border-radius: 8px;
}

.skeleton-loader.card .skeleton-item {
  border-radius: 12px;
}

.skeleton-loader.text .skeleton-item {
  height: 20px;
  border-radius: 4px;
}

.skeleton-loader.circle .skeleton-item {
  border-radius: 50%;
  aspect-ratio: 1;
}

@keyframes loading {
  0% {
    background-position: 200% 0;
  }
  100% {
    background-position: -200% 0;
  }
}

/* Dark mode support */
@media (prefers-color-scheme: dark) {
  .skeleton-item {
    background: linear-gradient(
      90deg,
      #2a2a2a 25%,
      #3a3a3a 50%,
      #2a2a2a 75%
    );
  }
}
</style>
