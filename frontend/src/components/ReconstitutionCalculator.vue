<template>
  <div class="reconstitution-calculator">
    <div class="calculator-header">
      <h3>🧮 Reconstitution Calculator</h3>
      <p class="subtitle">Calculate peptide dosing after reconstitution</p>
    </div>

    <div class="calculator-body">
      <!-- Input Section -->
      <div class="input-section">
        <div class="form-group">
          <label for="vial-size">Vial Size (mg)</label>
          <input
            id="vial-size"
            v-model.number="vialSizeMg"
            type="number"
            step="0.1"
            min="0"
            placeholder="e.g., 5"
            @input="calculate"
          />
        </div>

        <div class="form-group">
          <label for="bac-water">Bacteriostatic Water (ml)</label>
          <input
            id="bac-water"
            v-model.number="bacWaterMl"
            type="number"
            step="0.1"
            min="0"
            placeholder="e.g., 2"
            @input="calculate"
          />
        </div>

        <div class="form-group">
          <label for="desired-dose">Desired Dose (mg)</label>
          <input
            id="desired-dose"
            v-model.number="desiredDoseMg"
            type="number"
            step="0.01"
            min="0"
            placeholder="e.g., 0.25"
            @input="calculate"
          />
        </div>
      </div>

      <!-- Results Section -->
      <div v-if="isValid" class="results-section">
        <div class="result-card primary">
          <div class="result-icon">💉</div>
          <div class="result-content">
            <div class="result-label">Draw Volume</div>
            <div class="result-value">{{ drawVolumeMl.toFixed(3) }} ml</div>
            <div class="result-subtext">{{ drawVolumeUnits }} units on 1ml syringe</div>
          </div>
        </div>

        <div class="result-card">
          <div class="result-icon">⚗️</div>
          <div class="result-content">
            <div class="result-label">Concentration</div>
            <div class="result-value">{{ concentrationMgMl.toFixed(2) }} mg/ml</div>
          </div>
        </div>

        <div class="result-card">
          <div class="result-icon">💊</div>
          <div class="result-content">
            <div class="result-label">Total Doses</div>
            <div class="result-value">{{ totalDoses }}</div>
            <div class="result-subtext">at {{ desiredDoseMg }}mg each</div>
          </div>
        </div>
      </div>

      <!-- Quick Reference -->
      <div class="quick-reference">
        <h4>📏 Syringe Reference</h4>
        <div class="reference-grid">
          <div class="reference-item">
            <strong>1ml syringe:</strong> 100 units = 1ml
          </div>
          <div class="reference-item">
            <strong>0.5ml syringe:</strong> 50 units = 0.5ml
          </div>
          <div class="reference-item">
            <strong>0.3ml syringe:</strong> 30 units = 0.3ml
          </div>
        </div>
      </div>

      <!-- Common Presets -->
      <div class="presets-section">
        <h4>⚡ Common Presets</h4>
        <div class="preset-buttons">
          <button @click="applyPreset(5, 2, 0.25)" class="preset-btn">
            5mg / 2ml / 0.25mg
          </button>
          <button @click="applyPreset(10, 2, 0.5)" class="preset-btn">
            10mg / 2ml / 0.5mg
          </button>
          <button @click="applyPreset(5, 1, 0.25)" class="preset-btn">
            5mg / 1ml / 0.25mg
          </button>
          <button @click="applyPreset(10, 3, 0.5)" class="preset-btn">
            10mg / 3ml / 0.5mg
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

const vialSizeMg = ref(5);
const bacWaterMl = ref(2);
const desiredDoseMg = ref(0.25);

const isValid = computed(() => {
  return vialSizeMg.value > 0 && bacWaterMl.value > 0 && desiredDoseMg.value > 0;
});

const concentrationMgMl = computed(() => {
  if (!isValid.value) return 0;
  return vialSizeMg.value / bacWaterMl.value;
});

const drawVolumeMl = computed(() => {
  if (!isValid.value) return 0;
  return desiredDoseMg.value / concentrationMgMl.value;
});

const drawVolumeUnits = computed(() => {
  // 1ml syringe has 100 units
  return Math.round(drawVolumeMl.value * 100);
});

const totalDoses = computed(() => {
  if (!isValid.value) return 0;
  return Math.floor(vialSizeMg.value / desiredDoseMg.value);
});

function calculate() {
  // Trigger reactivity by accessing computed properties
  concentrationMgMl.value;
}

function applyPreset(vial: number, water: number, dose: number) {
  vialSizeMg.value = vial;
  bacWaterMl.value = water;
  desiredDoseMg.value = dose;
  calculate();
}

// Initialize with default calculation
calculate();
</script>

<style scoped>
.reconstitution-calculator {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.calculator-header {
  margin-bottom: 24px;
}

.calculator-header h3 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.subtitle {
  font-size: 14px;
  color: #666;
  margin: 0;
}

.calculator-body {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.input-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.form-group input {
  padding: 10px 12px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 16px;
  transition: all 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.results-section {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.result-card {
  background: #f8f9fa;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  gap: 16px;
  align-items: flex-start;
  transition: all 0.2s;
}

.result-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.result-card.primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
}

.result-icon {
  font-size: 32px;
  line-height: 1;
}

.result-content {
  flex: 1;
}

.result-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  opacity: 0.8;
  margin-bottom: 4px;
}

.result-card.primary .result-label {
  opacity: 0.9;
}

.result-value {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 4px;
}

.result-subtext {
  font-size: 13px;
  opacity: 0.7;
}

.result-card.primary .result-subtext {
  opacity: 0.8;
}

.quick-reference {
  background: #fff3cd;
  border: 2px solid #ffc107;
  border-radius: 12px;
  padding: 16px;
}

.quick-reference h4 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: #856404;
}

.reference-grid {
  display: grid;
  gap: 8px;
}

.reference-item {
  font-size: 14px;
  color: #856404;
}

.reference-item strong {
  font-weight: 600;
}

.presets-section h4 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 12px 0;
  color: #333;
}

.preset-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.preset-btn {
  padding: 8px 16px;
  background: #e7e7ff;
  border: 2px solid #667eea;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 600;
  color: #667eea;
  cursor: pointer;
  transition: all 0.2s;
}

.preset-btn:hover {
  background: #667eea;
  color: white;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
}

@media (prefers-color-scheme: dark) {
  .reconstitution-calculator {
    background: #2a2a2a;
    color: #e0e0e0;
  }

  .calculator-header h3 {
    color: #fff;
  }

  .subtitle {
    color: #aaa;
  }

  .form-group label {
    color: #e0e0e0;
  }

  .form-group input {
    background: #3a3a3a;
    border-color: #4a4a4a;
    color: #fff;
  }

  .result-card {
    background: #3a3a3a;
    border-color: #4a4a4a;
  }

  .quick-reference {
    background: #3a2f0f;
    border-color: #ffc107;
  }

  .quick-reference h4,
  .reference-item {
    color: #ffc107;
  }

  .presets-section h4 {
    color: #e0e0e0;
  }
}
</style>
