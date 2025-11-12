<template>
  <Teleport to="body">
    <Transition name="onboarding">
      <div v-if="isActive && currentStep < steps.length" class="onboarding-overlay">
        <div class="onboarding-container">
          <div class="onboarding-content">
            <!-- Progress -->
            <div class="onboarding-progress">
              <div
                v-for="(step, index) in steps"
                :key="index"
                class="progress-dot"
                :class="{ active: index === currentStep, completed: index < currentStep }"
              ></div>
            </div>

            <!-- Step Content -->
            <div class="step-content">
              <div class="step-icon">{{ steps[currentStep].icon }}</div>
              <h2 class="step-title">{{ steps[currentStep].title }}</h2>
              <p class="step-description">{{ steps[currentStep].description }}</p>

              <!-- Interactive Demo -->
              <div v-if="steps[currentStep].demo" class="step-demo">
                <component :is="steps[currentStep].demo" />
              </div>
            </div>

            <!-- Navigation -->
            <div class="onboarding-navigation">
              <button
                v-if="currentStep > 0"
                @click="prevStep"
                class="nav-btn back-btn"
              >
                ← Back
              </button>
              <button @click="skipOnboarding" class="nav-btn skip-btn">
                Skip Tour
              </button>
              <button
                @click="nextStep"
                class="nav-btn next-btn"
              >
                {{ currentStep === steps.length - 1 ? 'Get Started' : 'Next →' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

interface OnboardingStep {
  icon: string;
  title: string;
  description: string;
  demo?: any; // Optional component for interactive demo
}

const STORAGE_KEY = 'peptrack-onboarding-completed';

const isActive = ref(false);
const currentStep = ref(0);

const steps: OnboardingStep[] = [
  {
    icon: '👋',
    title: 'Welcome to PepTrack',
    description: 'Your all-in-one solution for tracking peptide protocols, doses, inventory, and research. Let\'s take a quick tour!',
  },
  {
    icon: '📋',
    title: 'Create Your First Protocol',
    description: 'Protocols help you organize your peptide regimens. Track dosing schedules, concentrations, and notes all in one place.',
  },
  {
    icon: '💉',
    title: 'Log Your Doses',
    description: 'Easily track every dose with details like amount, injection site, and timing. Build a complete history for better insights.',
  },
  {
    icon: '📦',
    title: 'Manage Inventory',
    description: 'Keep track of your peptide stock, costs, suppliers, and expiration dates. Never run out unexpectedly!',
  },
  {
    icon: '📊',
    title: 'Visualize Your Progress',
    description: 'View beautiful charts, heatmaps, and analytics to understand your adherence, spending, and progress over time.',
  },
  {
    icon: '🤖',
    title: 'AI-Powered Recommendations',
    description: 'Get science-backed protocol recommendations from our AI assistant, powered by the latest research papers.',
  },
  {
    icon: '⌨️',
    title: 'Keyboard Shortcuts',
    description: 'Power users love shortcuts! Press Cmd+K for search, ? for help, and number keys (1-8) to navigate between tabs.',
  },
  {
    icon: '🎉',
    title: 'You\'re All Set!',
    description: 'Start tracking your first protocol, log doses, and explore all the features. Your data is stored privately on your device.',
  },
];

const emit = defineEmits<{
  (e: 'complete'): void;
  (e: 'skip'): void;
}>();

onMounted(() => {
  // Check if onboarding has been completed
  const completed = localStorage.getItem(STORAGE_KEY);
  if (!completed) {
    startOnboarding();
  }
});

function startOnboarding() {
  isActive.value = true;
  currentStep.value = 0;
}

function nextStep() {
  if (currentStep.value < steps.length - 1) {
    currentStep.value++;
  } else {
    completeOnboarding();
  }
}

function prevStep() {
  if (currentStep.value > 0) {
    currentStep.value--;
  }
}

function skipOnboarding() {
  isActive.value = false;
  localStorage.setItem(STORAGE_KEY, 'skipped');
  emit('skip');
}

function completeOnboarding() {
  isActive.value = false;
  localStorage.setItem(STORAGE_KEY, 'completed');
  emit('complete');
}

// Expose methods for parent component
defineExpose({
  start: startOnboarding,
  reset: () => {
    localStorage.removeItem(STORAGE_KEY);
    startOnboarding();
  },
});
</script>

<style scoped>
.onboarding-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.85);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10001;
  padding: 20px;
}

.onboarding-container {
  max-width: 600px;
  width: 100%;
}

.onboarding-content {
  background: white;
  border-radius: 20px;
  padding: 40px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  animation: slideUp 0.5s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(40px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Progress */
.onboarding-progress {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-bottom: 32px;
}

.progress-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #e0e0e0;
  transition: all 0.3s;
}

.progress-dot.active {
  width: 32px;
  border-radius: 5px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.progress-dot.completed {
  background: #4caf50;
}

/* Step Content */
.step-content {
  text-align: center;
  margin-bottom: 32px;
  min-height: 280px;
}

.step-icon {
  font-size: 80px;
  margin-bottom: 24px;
  animation: bounce 0.6s ease;
}

@keyframes bounce {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-20px);
  }
}

.step-title {
  font-size: 28px;
  font-weight: 700;
  color: #1a1a1a;
  margin: 0 0 16px 0;
}

.step-description {
  font-size: 16px;
  line-height: 1.6;
  color: #666;
  margin: 0;
  max-width: 500px;
  margin-left: auto;
  margin-right: auto;
}

.step-demo {
  margin-top: 24px;
  padding: 20px;
  background: #f9f9f9;
  border-radius: 12px;
}

/* Navigation */
.onboarding-navigation {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}

.nav-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn {
  background: white;
  color: #666;
  border: 2px solid #e0e0e0;
}

.back-btn:hover {
  background: #f5f5f5;
  border-color: #ccc;
}

.skip-btn {
  background: transparent;
  color: #999;
  border: none;
  flex: 1;
}

.skip-btn:hover {
  color: #666;
  text-decoration: underline;
}

.next-btn {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
  min-width: 120px;
}

.next-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgba(102, 126, 234, 0.4);
}

.next-btn:active {
  transform: translateY(0);
}

/* Transitions */
.onboarding-enter-active,
.onboarding-leave-active {
  transition: opacity 0.3s ease;
}

.onboarding-enter-from,
.onboarding-leave-to {
  opacity: 0;
}

/* Dark mode */
:global(.dark-mode) .onboarding-content {
  background: #2a2a2a;
}

:global(.dark-mode) .step-title {
  color: #fff;
}

:global(.dark-mode) .step-description {
  color: #aaa;
}

:global(.dark-mode) .step-demo {
  background: #1a1a1a;
}

:global(.dark-mode) .back-btn {
  background: #1a1a1a;
  border-color: #404040;
  color: #aaa;
}

:global(.dark-mode) .back-btn:hover {
  background: #333;
}

:global(.dark-mode) .skip-btn {
  color: #777;
}

:global(.dark-mode) .skip-btn:hover {
  color: #aaa;
}

/* Responsive */
@media (max-width: 768px) {
  .onboarding-overlay {
    padding: 0;
  }

  .onboarding-content {
    border-radius: 0;
    padding: 24px;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
  }

  .step-content {
    min-height: auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
  }

  .step-icon {
    font-size: 64px;
  }

  .step-title {
    font-size: 24px;
  }

  .step-description {
    font-size: 15px;
  }

  .onboarding-navigation {
    flex-direction: column;
  }

  .back-btn,
  .next-btn {
    width: 100%;
  }

  .skip-btn {
    order: -1;
  }
}
</style>
