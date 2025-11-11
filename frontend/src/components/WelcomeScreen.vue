<template>
  <div v-if="showWelcome" class="welcome-overlay">
    <div class="welcome-modal">
      <div class="welcome-header">
        <h1>👋 Welcome to PepTrack!</h1>
        <p>Let's show you around really quick</p>
      </div>

      <div class="welcome-content">
        <div class="welcome-step">
          <div class="step-icon">💊</div>
          <h3>Track Your Peptides</h3>
          <p>Create plans for each peptide you're using. Track what you're taking, where you got it, and any notes you want to remember.</p>
        </div>

        <div class="welcome-step">
          <div class="step-icon">📚</div>
          <h3>Find Research Papers</h3>
          <p>Search for scientific studies about peptides. Papers are automatically saved so you can find them later.</p>
        </div>

        <div class="welcome-step">
          <div class="step-icon">🤖</div>
          <h3>Get AI Summaries</h3>
          <p>Paste complex research text and get a simple summary. All processing happens on your computer - nothing is sent to the cloud.</p>
        </div>

        <div class="welcome-step">
          <div class="step-icon">🔒</div>
          <h3>Your Privacy Matters</h3>
          <p>Everything you save is encrypted and stays on your computer. No accounts, no cloud servers, no tracking.</p>
        </div>

        <div class="welcome-step ai-setup-section">
          <div class="step-icon">⚙️</div>
          <h3>Optional: AI Summaries</h3>
          <p>The AI Summary Helper is <strong>completely optional</strong>. If you want to use it, you'll need to install either:</p>
          <ul class="ai-options">
            <li><strong>Codex CLI</strong> (uses GPT-5) - <a href="https://developers.openai.com/codex/cli" target="_blank">Install Guide</a></li>
            <li><strong>Claude CLI</strong> (uses Haiku 4.5) - <a href="https://code.claude.com/docs/en/cli-reference" target="_blank">Install Guide</a></li>
          </ul>
          <p class="ai-note">Don't worry if you skip this - everything else in PepTrack works perfectly without AI!</p>
        </div>
      </div>

      <div class="welcome-footer">
        <button @click="closeWelcome" class="primary-button">
          Got It, Let's Go! 🚀
        </button>
        <label class="dont-show-again">
          <input type="checkbox" v-model="dontShowAgain" />
          Don't show this again
        </label>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';

const showWelcome = ref(false);
const dontShowAgain = ref(false);

const STORAGE_KEY = 'peptrack-welcome-shown';

// Expose method to parent component
defineExpose({
  open: () => {
    showWelcome.value = true;
    dontShowAgain.value = false;
  }
});

onMounted(() => {
  const hasSeenWelcome = localStorage.getItem(STORAGE_KEY);
  if (!hasSeenWelcome) {
    showWelcome.value = true;
  }
});

function closeWelcome() {
  if (dontShowAgain.value) {
    localStorage.setItem(STORAGE_KEY, 'true');
  }
  showWelcome.value = false;
}
</script>

<style scoped>
.welcome-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
  animation: fadeIn 0.3s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.welcome-modal {
  background: white;
  border-radius: 16px;
  max-width: 700px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.welcome-header {
  background: linear-gradient(135deg, #42b983 0%, #359268 100%);
  color: white;
  padding: 30px;
  text-align: center;
  border-radius: 16px 16px 0 0;
}

.welcome-header h1 {
  margin: 0 0 10px 0;
  font-size: 32px;
}

.welcome-header p {
  margin: 0;
  font-size: 16px;
  opacity: 0.95;
}

.welcome-content {
  padding: 30px;
}

.welcome-step {
  margin-bottom: 30px;
  text-align: center;
}

.welcome-step:last-child {
  margin-bottom: 0;
}

.step-icon {
  font-size: 48px;
  margin-bottom: 10px;
}

.welcome-step h3 {
  margin: 0 0 10px 0;
  color: #2c3e50;
  font-size: 20px;
}

.welcome-step p {
  margin: 0;
  color: #666;
  line-height: 1.6;
  font-size: 15px;
}

.welcome-footer {
  padding: 20px 30px 30px;
  text-align: center;
  border-top: 1px solid #eee;
}

.primary-button {
  background: #42b983;
  color: white;
  border: none;
  border-radius: 8px;
  padding: 14px 32px;
  font-size: 16px;
  font-weight: bold;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 15px;
  width: 100%;
  max-width: 300px;
}

.primary-button:hover {
  background: #359268;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(66, 185, 131, 0.3);
}

.primary-button:active {
  transform: translateY(0);
}

.dont-show-again {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  user-select: none;
}

.dont-show-again input[type="checkbox"] {
  cursor: pointer;
  width: 18px;
  height: 18px;
}

.ai-setup-section {
  background: #f8f9fa;
  border: 1px solid #dee2e6;
  border-radius: 8px;
  padding: 20px;
  margin-top: 10px;
}

.ai-options {
  text-align: left;
  margin: 15px auto;
  max-width: 400px;
  padding-left: 20px;
}

.ai-options li {
  margin: 8px 0;
  line-height: 1.6;
}

.ai-options a {
  color: #42b983;
  text-decoration: none;
  font-weight: 600;
}

.ai-options a:hover {
  text-decoration: underline;
}

.ai-note {
  font-size: 13px;
  color: #666;
  font-style: italic;
  margin-top: 12px;
  background: #fffbea;
  padding: 8px 12px;
  border-radius: 4px;
  border-left: 3px solid #f39c12;
}

/* Mobile responsive */
@media (max-width: 768px) {
  .welcome-modal {
    max-width: 100%;
    border-radius: 12px;
  }

  .welcome-header {
    padding: 24px;
  }

  .welcome-header h1 {
    font-size: 26px;
  }

  .welcome-content {
    padding: 20px;
  }

  .welcome-step {
    margin-bottom: 24px;
  }

  .step-icon {
    font-size: 40px;
  }
}
</style>
