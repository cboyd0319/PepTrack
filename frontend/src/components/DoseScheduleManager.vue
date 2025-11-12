<template>
  <div class="schedule-manager">
    <div class="schedule-header">
      <h3>⏰ Dose Schedules</h3>
      <p class="subtitle">Set up recurring reminders for your peptide doses</p>
    </div>

    <!-- Create Schedule Form -->
    <div class="schedule-form-card">
      <h4>{{ editingSchedule ? '✏️ Edit Schedule' : '➕ New Schedule' }}</h4>
      <form @submit.prevent="handleSubmit" class="schedule-form">
        <div class="form-row">
          <label>
            Protocol *
            <select v-model="form.protocolId" required :disabled="loading">
              <option value="">Select a protocol...</option>
              <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
                {{ protocol.name }} ({{ protocol.peptide_name }})
              </option>
            </select>
          </label>

          <label>
            Amount (mg) *
            <input
              v-model.number="form.amountMg"
              type="number"
              step="0.01"
              min="0"
              required
              :disabled="loading"
              placeholder="e.g., 250"
            />
          </label>
        </div>

        <div class="form-row">
          <label>
            Time of Day *
            <input
              v-model="form.timeOfDay"
              type="time"
              required
              :disabled="loading"
            />
          </label>

          <label>
            Injection Site
            <input
              v-model="form.site"
              type="text"
              :disabled="loading"
              placeholder="e.g., abdomen, thigh"
            />
          </label>
        </div>

        <div class="days-selector">
          <label>Days of Week *</label>
          <div class="days-grid">
            <label
              v-for="(day, index) in daysOfWeek"
              :key="index"
              class="day-checkbox"
              :class="{ selected: form.daysOfWeek.includes(index) }"
            >
              <input
                type="checkbox"
                :value="index"
                v-model="form.daysOfWeek"
                :disabled="loading"
              />
              <span>{{ day }}</span>
            </label>
          </div>
        </div>

        <label>
          Notes
          <textarea
            v-model="form.notes"
            rows="2"
            :disabled="loading"
            placeholder="Optional notes about this schedule..."
          />
        </label>

        <div class="form-actions">
          <button
            type="submit"
            class="btn-primary"
            :disabled="loading || !isFormValid"
          >
            {{ loading ? '⏳ Saving...' : (editingSchedule ? '💾 Update Schedule' : '➕ Create Schedule') }}
          </button>
          <button
            v-if="editingSchedule"
            type="button"
            class="btn-secondary"
            @click="cancelEdit"
            :disabled="loading"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>

    <!-- Schedules List -->
    <div class="schedules-list">
      <div class="list-header">
        <h4>📋 Your Schedules ({{ schedules.length }})</h4>
        <button @click="loadSchedules" class="btn-refresh" :disabled="loading">
          🔄 Refresh
        </button>
      </div>

      <div v-if="schedules.length === 0" class="empty-state">
        <p>📅 No schedules yet</p>
        <p class="hint">Create your first dose schedule above to get reminder notifications</p>
      </div>

      <div v-else class="schedule-cards">
        <div
          v-for="schedule in sortedSchedules"
          :key="schedule.id"
          class="schedule-card"
          :class="{ disabled: !schedule.enabled }"
        >
          <div class="schedule-main">
            <div class="schedule-info">
              <h5>
                {{ schedule.protocolName }}
                <span class="badge">{{ schedule.peptideName }}</span>
              </h5>
              <div class="schedule-details">
                <span class="detail">🕐 {{ formatTime(schedule.timeOfDay) }}</span>
                <span class="detail">💉 {{ schedule.amountMg }}mg</span>
                <span v-if="schedule.site" class="detail">📍 {{ schedule.site }}</span>
              </div>
              <div class="schedule-days">
                <span
                  v-for="(day, index) in daysOfWeek"
                  :key="index"
                  class="day-pill"
                  :class="{ active: schedule.daysOfWeek.includes(index) }"
                >
                  {{ day.substring(0, 3) }}
                </span>
              </div>
              <p v-if="schedule.notes" class="schedule-notes">{{ schedule.notes }}</p>
            </div>

            <div class="schedule-actions">
              <label class="toggle-switch" :title="schedule.enabled ? 'Enabled' : 'Disabled'">
                <input
                  type="checkbox"
                  :checked="schedule.enabled"
                  @change="toggleSchedule(schedule)"
                  :disabled="loading"
                />
                <span class="slider"></span>
              </label>

              <button
                @click="startEdit(schedule)"
                class="btn-icon"
                title="Edit schedule"
                :disabled="loading"
              >
                ✏️
              </button>

              <button
                @click="confirmDelete(schedule)"
                class="btn-icon danger"
                title="Delete schedule"
                :disabled="loading"
              >
                🗑️
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Modal -->
    <div v-if="deletingSchedule" class="modal-overlay" @click="cancelDelete">
      <div class="modal-content" @click.stop>
        <h4>⚠️ Delete Schedule?</h4>
        <p>
          Are you sure you want to delete the schedule for
          <strong>{{ deletingSchedule.protocolName }}</strong>?
        </p>
        <p class="warning-text">This action cannot be undone.</p>
        <div class="modal-actions">
          <button @click="handleDelete" class="btn-danger" :disabled="loading">
            {{ loading ? '⏳ Deleting...' : '🗑️ Delete' }}
          </button>
          <button @click="cancelDelete" class="btn-secondary" :disabled="loading">
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  createDoseSchedule,
  listDoseSchedules,
  updateDoseSchedule,
  deleteDoseSchedule,
  listProtocols,
  type DoseSchedule,
  type CreateSchedulePayload,
  type UpdateSchedulePayload,
  type PeptideProtocol,
} from '../api/peptrack';
import { showSuccessToast, showErrorToast } from '../utils/errorHandling';

const daysOfWeek = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];

const schedules = ref<DoseSchedule[]>([]);
const protocols = ref<PeptideProtocol[]>([]);
const loading = ref(false);
const editingSchedule = ref<DoseSchedule | null>(null);
const deletingSchedule = ref<DoseSchedule | null>(null);

const form = ref({
  protocolId: '',
  amountMg: 0,
  site: '',
  timeOfDay: '09:00',
  daysOfWeek: [] as number[],
  notes: '',
});

const isFormValid = computed(() => {
  return (
    form.value.protocolId &&
    form.value.amountMg > 0 &&
    form.value.timeOfDay &&
    form.value.daysOfWeek.length > 0
  );
});

const sortedSchedules = computed(() => {
  return [...schedules.value].sort((a, b) => {
    // Sort by time of day
    return a.timeOfDay.localeCompare(b.timeOfDay);
  });
});

onMounted(async () => {
  await loadProtocols();
  await loadSchedules();
});

async function loadProtocols() {
  try {
    protocols.value = await listProtocols();
  } catch (error) {
    showErrorToast(error, { operation: 'load protocols' });
  }
}

async function loadSchedules() {
  loading.value = true;
  try {
    schedules.value = await listDoseSchedules();
  } catch (error) {
    showErrorToast(error, { operation: 'load schedules' });
  } finally {
    loading.value = false;
  }
}

async function handleSubmit() {
  if (!isFormValid.value) return;

  loading.value = true;
  try {
    if (editingSchedule.value) {
      // Update existing schedule
      const payload: UpdateSchedulePayload = {
        id: editingSchedule.value.id,
        amountMg: form.value.amountMg,
        site: form.value.site || undefined,
        timeOfDay: form.value.timeOfDay,
        daysOfWeek: form.value.daysOfWeek,
        notes: form.value.notes || undefined,
      };
      await updateDoseSchedule(payload);
      showSuccessToast('Schedule updated successfully');
    } else {
      // Create new schedule
      const payload: CreateSchedulePayload = {
        protocolId: form.value.protocolId,
        amountMg: form.value.amountMg,
        site: form.value.site || undefined,
        timeOfDay: form.value.timeOfDay,
        daysOfWeek: form.value.daysOfWeek,
        notes: form.value.notes || undefined,
      };
      await createDoseSchedule(payload);
      showSuccessToast('Schedule created successfully');
    }

    resetForm();
    await loadSchedules();
  } catch (error) {
    showErrorToast(error, { operation: editingSchedule.value ? 'update schedule' : 'create schedule' });
  } finally {
    loading.value = false;
  }
}

async function toggleSchedule(schedule: DoseSchedule) {
  loading.value = true;
  try {
    await updateDoseSchedule({
      id: schedule.id,
      enabled: !schedule.enabled,
    });
    showSuccessToast(`Schedule ${!schedule.enabled ? 'enabled' : 'disabled'}`);
    await loadSchedules();
  } catch (error) {
    showErrorToast(error, { operation: 'toggle schedule' });
  } finally {
    loading.value = false;
  }
}

function startEdit(schedule: DoseSchedule) {
  editingSchedule.value = schedule;
  form.value = {
    protocolId: schedule.protocolId,
    amountMg: schedule.amountMg,
    site: schedule.site || '',
    timeOfDay: schedule.timeOfDay,
    daysOfWeek: [...schedule.daysOfWeek],
    notes: schedule.notes || '',
  };
  // Scroll to form
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function cancelEdit() {
  editingSchedule.value = null;
  resetForm();
}

function confirmDelete(schedule: DoseSchedule) {
  deletingSchedule.value = schedule;
}

function cancelDelete() {
  deletingSchedule.value = null;
}

async function handleDelete() {
  if (!deletingSchedule.value) return;

  loading.value = true;
  try {
    await deleteDoseSchedule(deletingSchedule.value.id);
    showSuccessToast('Schedule deleted successfully');
    deletingSchedule.value = null;
    await loadSchedules();
  } catch (error) {
    showErrorToast(error, { operation: 'delete schedule' });
  } finally {
    loading.value = false;
  }
}

function resetForm() {
  form.value = {
    protocolId: '',
    amountMg: 0,
    site: '',
    timeOfDay: '09:00',
    daysOfWeek: [],
    notes: '',
  };
  editingSchedule.value = null;
}

function formatTime(time: string): string {
  const [hours, minutes] = time.split(':').map(Number);
  const period = hours >= 12 ? 'PM' : 'AM';
  const displayHours = hours % 12 || 12;
  return `${displayHours}:${minutes.toString().padStart(2, '0')} ${period}`;
}
</script>

<style scoped>
.schedule-manager {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.schedule-header {
  margin-bottom: 24px;
}

.schedule-header h3 {
  margin: 0 0 8px 0;
  font-size: 24px;
  color: #333;
}

.subtitle {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.schedule-form-card {
  background: white;
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 32px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.schedule-form-card h4 {
  margin: 0 0 20px 0;
  font-size: 18px;
  color: #333;
}

.schedule-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 14px;
  font-weight: 600;
  color: #555;
}

input[type="text"],
input[type="number"],
input[type="time"],
select,
textarea {
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
  transition: border-color 0.2s;
}

input:focus,
select:focus,
textarea:focus {
  outline: none;
  border-color: #667eea;
}

input:disabled,
select:disabled,
textarea:disabled {
  background: #f5f5f5;
  cursor: not-allowed;
}

.days-selector {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.days-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: 8px;
}

.day-checkbox {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
  border: 2px solid #ddd;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  font-weight: 500;
  font-size: 13px;
}

.day-checkbox input {
  display: none;
}

.day-checkbox.selected {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
}

.day-checkbox:hover {
  border-color: #667eea;
  transform: translateY(-2px);
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 8px;
}

.btn-primary,
.btn-secondary,
.btn-danger {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: #f5f5f5;
  color: #555;
}

.btn-secondary:hover:not(:disabled) {
  background: #e0e0e0;
}

.schedules-list {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.list-header h4 {
  margin: 0;
  font-size: 18px;
  color: #333;
}

.btn-refresh {
  padding: 8px 16px;
  background: #f5f5f5;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-refresh:hover:not(:disabled) {
  background: #e0e0e0;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #999;
}

.empty-state p {
  margin: 8px 0;
}

.hint {
  font-size: 13px;
  color: #aaa;
}

.schedule-cards {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.schedule-card {
  border: 2px solid #e0e0e0;
  border-radius: 10px;
  padding: 20px;
  transition: all 0.2s;
}

.schedule-card:hover {
  border-color: #667eea;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.1);
}

.schedule-card.disabled {
  opacity: 0.6;
  background: #f9f9f9;
}

.schedule-main {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
}

.schedule-info {
  flex: 1;
}

.schedule-info h5 {
  margin: 0 0 12px 0;
  font-size: 18px;
  color: #333;
  display: flex;
  align-items: center;
  gap: 10px;
}

.badge {
  padding: 4px 10px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
}

.schedule-details {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-bottom: 12px;
}

.detail {
  font-size: 14px;
  color: #666;
}

.schedule-days {
  display: flex;
  gap: 6px;
  margin-bottom: 12px;
}

.day-pill {
  padding: 4px 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 12px;
  color: #999;
  font-weight: 500;
}

.day-pill.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
}

.schedule-notes {
  margin: 8px 0 0 0;
  padding: 10px;
  background: #f9f9f9;
  border-radius: 6px;
  font-size: 13px;
  color: #666;
  font-style: italic;
}

.schedule-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.toggle-switch {
  position: relative;
  width: 50px;
  height: 26px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: 0.3s;
  border-radius: 26px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

input:checked + .slider:before {
  transform: translateX(24px);
}

.btn-icon {
  padding: 8px 12px;
  background: #f5f5f5;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 16px;
}

.btn-icon:hover:not(:disabled) {
  background: #e0e0e0;
  transform: translateY(-2px);
}

.btn-icon.danger:hover:not(:disabled) {
  background: #fee;
  color: #d32f2f;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  padding: 32px;
  border-radius: 12px;
  max-width: 450px;
  width: 90%;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.modal-content h4 {
  margin: 0 0 16px 0;
  font-size: 20px;
  color: #333;
}

.modal-content p {
  margin: 12px 0;
  color: #666;
}

.warning-text {
  color: #d32f2f;
  font-size: 13px;
  font-weight: 600;
}

.modal-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.btn-danger {
  background: #d32f2f;
  color: white;
}

.btn-danger:hover:not(:disabled) {
  background: #b71c1c;
  transform: translateY(-2px);
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .days-grid {
    grid-template-columns: repeat(4, 1fr);
  }

  .schedule-main {
    flex-direction: column;
  }

  .schedule-actions {
    width: 100%;
    justify-content: space-between;
  }
}

/* Dark mode */
@media (prefers-color-scheme: dark) {
  .schedule-header h3,
  .schedule-form-card h4,
  .list-header h4,
  .schedule-info h5 {
    color: #e0e0e0;
  }

  .subtitle,
  .detail,
  .schedule-notes {
    color: #aaa;
  }

  .schedule-form-card,
  .schedules-list,
  input,
  select,
  textarea {
    background: #2a2a2a;
    color: #e0e0e0;
    border-color: #3a3a3a;
  }

  .schedule-card {
    background: #2a2a2a;
    border-color: #3a3a3a;
  }

  .schedule-card.disabled {
    background: #1a1a1a;
  }

  .day-checkbox {
    border-color: #3a3a3a;
    color: #e0e0e0;
  }

  .day-pill {
    border-color: #3a3a3a;
    background: #1a1a1a;
    color: #aaa;
  }

  .btn-secondary,
  .btn-refresh,
  .btn-icon {
    background: #3a3a3a;
    color: #e0e0e0;
  }

  .btn-secondary:hover,
  .btn-refresh:hover,
  .btn-icon:hover {
    background: #4a4a4a;
  }

  .modal-content {
    background: #2a2a2a;
    color: #e0e0e0;
  }
}

:global(.dark-mode) .schedule-header h3,
:global(.dark-mode) .schedule-form-card h4,
:global(.dark-mode) .list-header h4,
:global(.dark-mode) .schedule-info h5 {
  color: #e0e0e0;
}

:global(.dark-mode) .subtitle,
:global(.dark-mode) .detail,
:global(.dark-mode) .schedule-notes {
  color: #aaa;
}

:global(.dark-mode) .schedule-form-card,
:global(.dark-mode) .schedules-list,
:global(.dark-mode) input,
:global(.dark-mode) select,
:global(.dark-mode) textarea {
  background: #2a2a2a;
  color: #e0e0e0;
  border-color: #3a3a3a;
}

:global(.dark-mode) .schedule-card {
  background: #2a2a2a;
  border-color: #3a3a3a;
}

:global(.dark-mode) .schedule-card.disabled {
  background: #1a1a1a;
}

:global(.dark-mode) .day-checkbox {
  border-color: #3a3a3a;
  color: #e0e0e0;
}

:global(.dark-mode) .day-pill {
  border-color: #3a3a3a;
  background: #1a1a1a;
  color: #aaa;
}

:global(.dark-mode) .btn-secondary,
:global(.dark-mode) .btn-refresh,
:global(.dark-mode) .btn-icon {
  background: #3a3a3a;
  color: #e0e0e0;
}

:global(.dark-mode) .btn-secondary:hover,
:global(.dark-mode) .btn-refresh:hover,
:global(.dark-mode) .btn-icon:hover {
  background: #4a4a4a;
}

:global(.dark-mode) .modal-content {
  background: #2a2a2a;
  color: #e0e0e0;
}
</style>
