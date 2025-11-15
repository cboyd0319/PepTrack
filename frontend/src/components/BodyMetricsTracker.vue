<template>
  <div class="body-metrics-tracker">
    <h2>📊 Body Metrics Tracking</h2>
    <p class="subtitle">Track your body composition and health metrics</p>

    <!-- Log New Metric Form -->
    <div class="panel log-metric-section">
      <h3>➕ Log Body Metric</h3>
      <form @submit.prevent="handleLogMetric" class="metric-form">
        <div class="form-row">
          <label for="metric-date">
            Date
            <input
              id="metric-date"
              v-model="form.date"
              type="date"
              required
              aria-label="Measurement date"
            />
          </label>

          <label for="weight-kg">
            Weight (kg)
            <input
              id="weight-kg"
              v-model.number="form.weight_kg"
              type="number"
              step="0.1"
              min="0"
              placeholder="e.g., 75.5"
              aria-label="Weight in kilograms"
            />
          </label>
        </div>

        <div class="form-row">
          <label for="body-fat">
            Body Fat (%)
            <input
              id="body-fat"
              v-model.number="form.body_fat_percentage"
              type="number"
              step="0.1"
              min="0"
              max="100"
              placeholder="e.g., 15.5"
              aria-label="Body fat percentage"
            />
          </label>

          <label for="muscle-mass">
            Muscle Mass (kg)
            <input
              id="muscle-mass"
              v-model.number="form.muscle_mass_kg"
              type="number"
              step="0.1"
              min="0"
              placeholder="e.g., 45.2"
              aria-label="Muscle mass in kilograms"
            />
          </label>
        </div>

        <div class="form-row">
          <label for="waist">
            Waist (cm)
            <input
              id="waist"
              v-model.number="form.waist_cm"
              type="number"
              step="0.1"
              min="0"
              placeholder="e.g., 85.0"
              aria-label="Waist circumference in centimeters"
            />
          </label>

          <div></div> <!-- Spacer for grid -->
        </div>

        <label for="metric-notes">
          Notes (optional)
        </label>
        <textarea
          id="metric-notes"
          v-model="form.notes"
          rows="2"
          placeholder="Notes about measurements, conditions, etc."
          aria-label="Additional notes"
        />

        <button
          type="submit"
          :disabled="isLogging || !form.date"
          class="primary-btn"
          aria-label="Log body metric"
          :aria-busy="isLogging"
        >
          {{ isLogging ? '⏳ Logging...' : '💾 Save Metric' }}
        </button>
      </form>
    </div>

    <!-- Metrics History -->
    <div class="history-section">
      <div class="history-header">
        <h3>📈 Your Metrics History</h3>
        <button
          @click="loadMetrics"
          class="refresh-btn"
          aria-label="Refresh metrics history"
        >
          ↻ Refresh
        </button>
      </div>

      <div v-if="metrics.length === 0" class="no-metrics">
        No metrics logged yet. Log your first measurement above!
      </div>

      <div v-else class="metrics-list">
        <div v-for="metric in metrics" :key="metric.id" class="metric-card">
          <div class="metric-header">
            <div class="metric-date">
              {{ formatDate(metric.date) }}
            </div>
            <button
              @click="deleteMetric(metric.id)"
              class="delete-btn"
              :aria-label="`Delete metric from ${formatDate(metric.date)}`"
            >
              🗑️
            </button>
          </div>

          <div class="metric-details">
            <div v-if="metric.weight_kg" class="metric-item">
              <span class="metric-label">Weight:</span>
              <span class="metric-value">{{ metric.weight_kg }} kg</span>
            </div>
            <div v-if="metric.body_fat_percentage" class="metric-item">
              <span class="metric-label">Body Fat:</span>
              <span class="metric-value">{{ metric.body_fat_percentage }}%</span>
            </div>
            <div v-if="metric.muscle_mass_kg" class="metric-item">
              <span class="metric-label">Muscle Mass:</span>
              <span class="metric-value">{{ metric.muscle_mass_kg }} kg</span>
            </div>
            <div v-if="metric.waist_cm" class="metric-item">
              <span class="metric-label">Waist:</span>
              <span class="metric-value">{{ metric.waist_cm }} cm</span>
            </div>
          </div>

          <p v-if="metric.notes" class="metric-notes">
            📝 {{ metric.notes }}
          </p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';
import {
  logBodyMetric,
  listBodyMetrics,
  deleteBodyMetric,
  type BodyMetric,
  type BodyMetricPayload,
} from '../api/peptrack';

// State
const metrics = ref<BodyMetric[]>([]);
const isLogging = ref(false);

// Form state - initialize with today's date
const form = ref<BodyMetricPayload & { date: string }>({
  date: new Date().toISOString().split('T')[0], // YYYY-MM-DD format
  weight_kg: null,
  body_fat_percentage: null,
  muscle_mass_kg: null,
  waist_cm: null,
  notes: '',
});

onMounted(async () => {
  await loadMetrics();
});

async function loadMetrics() {
  try {
    metrics.value = await listBodyMetrics();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load body metrics' });
  }
}

async function handleLogMetric() {
  if (!form.value.date) {
    showErrorToast(new Error('Please select a date'), { operation: 'log metric' });
    return;
  }

  isLogging.value = true;

  try {
    // Convert date to ISO 8601 format
    const payload: BodyMetricPayload = {
      date: new Date(form.value.date).toISOString(),
      weight_kg: form.value.weight_kg || null,
      body_fat_percentage: form.value.body_fat_percentage || null,
      muscle_mass_kg: form.value.muscle_mass_kg || null,
      waist_cm: form.value.waist_cm || null,
      notes: form.value.notes?.trim() || null,
    };

    await logBodyMetric(payload);
    showSuccessToast('Success', 'Body metric logged successfully!');

    // Reset form to today's date
    form.value = {
      date: new Date().toISOString().split('T')[0],
      weight_kg: null,
      body_fat_percentage: null,
      muscle_mass_kg: null,
      waist_cm: null,
      notes: '',
    };

    // Reload metrics
    await loadMetrics();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'log body metric' });
  } finally {
    isLogging.value = false;
  }
}

async function deleteMetric(metricId: string) {
  if (!confirm('Are you sure you want to delete this metric entry?')) {
    return;
  }

  try {
    await deleteBodyMetric(metricId);
    await loadMetrics();
    showSuccessToast('Success', 'Metric deleted successfully');
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'delete metric' });
  }
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  } catch {
    return dateStr;
  }
}
</script>

<style scoped>
.body-metrics-tracker {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

h2 {
  margin-bottom: 8px;
  color: #2c3e50;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin-bottom: 20px;
  margin-top: 0;
}

.panel {
  background: #f9f9f9;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.log-metric-section h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #2c3e50;
}

.metric-form {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.metric-form label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-weight: 500;
  color: #2c3e50;
}

.metric-form input,
.metric-form textarea {
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.metric-form input:focus,
.metric-form textarea:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
}

.primary-btn {
  padding: 12px 24px;
  background-color: #42b983;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: bold;
  transition: background-color 0.2s;
}

.primary-btn:hover:not(:disabled) {
  background-color: #359268;
}

.primary-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.history-section {
  margin-top: 30px;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  flex-wrap: wrap;
  gap: 10px;
}

.history-header h3 {
  margin: 0;
  color: #2c3e50;
}

.refresh-btn {
  padding: 6px 12px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.refresh-btn:hover {
  background-color: #2980b9;
}

.no-metrics {
  padding: 40px;
  text-align: center;
  color: #999;
  font-style: italic;
  background: #f9f9f9;
  border-radius: 8px;
}

.metrics-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.metric-card {
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 15px;
  transition: box-shadow 0.2s;
}

.metric-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.metric-date {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
}

.delete-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 18px;
  padding: 4px;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.delete-btn:hover {
  opacity: 1;
}

.metric-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 10px;
  margin-bottom: 10px;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.metric-label {
  font-size: 12px;
  color: #666;
  font-weight: 500;
}

.metric-value {
  font-size: 16px;
  font-weight: 600;
  color: #667eea;
}

.metric-notes {
  margin: 10px 0 0 0;
  padding: 10px;
  background: #f9f9f9;
  border-radius: 4px;
  font-size: 14px;
  color: #555;
  line-height: 1.5;
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .metric-details {
    grid-template-columns: 1fr;
  }

  .history-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .refresh-btn {
    width: 100%;
  }
}
</style>
