<template>
  <div class="protocol-progress">
    <h3>📊 Protocol Progress</h3>
    <p class="subtitle">Track your active protocol cycles and completion</p>

    <div v-if="loading" class="loading">Loading protocols...</div>

    <div v-else-if="protocolProgress.length === 0" class="empty-state">
      <span class="empty-icon">📋</span>
      <p>No active protocols found</p>
      <p class="hint">Create a protocol to start tracking progress</p>
    </div>

    <div v-else class="progress-list">
      <div
        v-for="progress in protocolProgress"
        :key="progress.protocol.id"
        class="progress-card"
      >
        <div class="card-header">
          <div class="protocol-info">
            <h4>{{ progress.protocol.name }}</h4>
            <p class="peptide-name">{{ progress.protocol.peptide_name }}</p>
          </div>
          <div class="progress-ring-container">
            <svg class="progress-ring" width="80" height="80">
              <circle
                class="progress-ring-background"
                cx="40"
                cy="40"
                r="32"
                fill="none"
                stroke="#e0e0e0"
                stroke-width="6"
              />
              <circle
                class="progress-ring-fill"
                cx="40"
                cy="40"
                r="32"
                fill="none"
                :stroke="getProgressColor(progress.percentage)"
                stroke-width="6"
                :stroke-dasharray="`${progress.circumference} ${progress.circumference}`"
                :stroke-dashoffset="progress.offset"
                stroke-linecap="round"
              />
              <text
                x="40"
                y="40"
                text-anchor="middle"
                dy=".3em"
                class="progress-text"
                :fill="getProgressColor(progress.percentage)"
              >
                {{ Math.round(progress.percentage) }}%
              </text>
            </svg>
          </div>
        </div>

        <div class="progress-details">
          <div class="detail-row">
            <span class="label">Duration:</span>
            <span class="value">
              Day {{ progress.currentDay }} of {{ progress.totalDays }}
            </span>
          </div>
          <div class="detail-row">
            <span class="label">Doses Given:</span>
            <span class="value">
              {{ progress.dosesGiven }} / {{ progress.expectedDoses }}
            </span>
          </div>
          <div class="detail-row">
            <span class="label">Adherence:</span>
            <span class="value" :class="getAdherenceClass(progress.adherence)">
              {{ Math.round(progress.adherence) }}%
            </span>
          </div>
          <div class="detail-row">
            <span class="label">Started:</span>
            <span class="value">
              {{ formatDate(progress.startDate) }}
            </span>
          </div>
          <div class="detail-row">
            <span class="label">Est. Completion:</span>
            <span class="value">
              {{ formatDate(progress.endDate) }}
            </span>
          </div>
        </div>

        <div class="progress-bar-container">
          <div class="progress-bar">
            <div
              class="progress-bar-fill"
              :style="{ width: `${progress.percentage}%` }"
              :class="getProgressClass(progress.percentage)"
            ></div>
          </div>
        </div>

        <div class="milestones">
          <div
            v-for="milestone in progress.milestones"
            :key="milestone.day"
            class="milestone"
            :class="{ achieved: milestone.achieved }"
          >
            <span class="milestone-icon">{{ milestone.achieved ? '✓' : '○' }}</span>
            <span class="milestone-label">{{ milestone.label }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { listProtocols, listDoseLogs, type PeptideProtocol } from '../api/peptrack';

interface Milestone {
  day: number;
  label: string;
  achieved: boolean;
}

interface ProtocolProgress {
  protocol: PeptideProtocol;
  startDate: string;
  endDate: string;
  currentDay: number;
  totalDays: number;
  dosesGiven: number;
  expectedDoses: number;
  adherence: number;
  percentage: number;
  circumference: number;
  offset: number;
  milestones: Milestone[];
}

const loading = ref(false);
const protocolProgress = ref<ProtocolProgress[]>([]);

onMounted(async () => {
  await loadProgressData();
});

async function loadProgressData() {
  loading.value = true;
  try {
    const [protocols, doses] = await Promise.all([
      listProtocols(),
      listDoseLogs(),
    ]);

    const progressList: ProtocolProgress[] = [];

    for (const protocol of protocols) {
      const protocolDoses = doses.filter(d => d.protocol_id === protocol.id);

      if (protocolDoses.length === 0) continue;

      // Find first and last dose
      const sortedDoses = protocolDoses.sort((a, b) =>
        new Date(a.logged_at).getTime() - new Date(b.logged_at).getTime()
      );

      const startDate = sortedDoses[0]!.logged_at;
      const today = new Date();

      // Assume 90-day cycle (can be made configurable)
      const totalDays = 90;
      const start = new Date(startDate);
      const daysSinceStart = Math.floor((today.getTime() - start.getTime()) / (1000 * 60 * 60 * 24));
      const currentDay = Math.min(daysSinceStart, totalDays);

      const endDate = new Date(start);
      endDate.setDate(start.getDate() + totalDays);

      // Calculate expected doses (assuming daily dosing)
      const expectedDoses = currentDay;
      const dosesGiven = protocolDoses.length;
      const adherence = expectedDoses > 0 ? (dosesGiven / expectedDoses) * 100 : 0;
      const percentage = (currentDay / totalDays) * 100;

      // Calculate SVG circle metrics
      const radius = 32;
      const circumference = 2 * Math.PI * radius;
      const offset = circumference - (percentage / 100) * circumference;

      // Define milestones
      const milestones: Milestone[] = [
        { day: 7, label: '1 Week', achieved: currentDay >= 7 },
        { day: 30, label: '1 Month', achieved: currentDay >= 30 },
        { day: 60, label: '2 Months', achieved: currentDay >= 60 },
        { day: 90, label: 'Complete', achieved: currentDay >= 90 },
      ];

      progressList.push({
        protocol,
        startDate,
        endDate: endDate.toISOString(),
        currentDay,
        totalDays,
        dosesGiven,
        expectedDoses,
        adherence,
        percentage,
        circumference,
        offset,
        milestones,
      });
    }

    protocolProgress.value = progressList.sort((a, b) => b.percentage - a.percentage);
  } catch (error) {
    console.error('Failed to load protocol progress:', error);
  } finally {
    loading.value = false;
  }
}

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

function getProgressColor(percentage: number): string {
  if (percentage >= 75) return '#4caf50';
  if (percentage >= 50) return '#2196f3';
  if (percentage >= 25) return '#ff9800';
  return '#f44336';
}

function getProgressClass(percentage: number): string {
  if (percentage >= 75) return 'high';
  if (percentage >= 50) return 'medium';
  if (percentage >= 25) return 'low';
  return 'very-low';
}

function getAdherenceClass(adherence: number): string {
  if (adherence >= 90) return 'excellent';
  if (adherence >= 75) return 'good';
  if (adherence >= 60) return 'fair';
  return 'poor';
}
</script>

<style scoped>
.protocol-progress {
  padding: 24px;
}

.protocol-progress h3 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.subtitle {
  font-size: 15px;
  color: #666;
  margin: 0 0 24px 0;
}

.loading {
  text-align: center;
  padding: 40px;
  color: #666;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #999;
}

.empty-icon {
  font-size: 64px;
  display: block;
  margin-bottom: 16px;
}

.hint {
  font-size: 14px;
  margin-top: 8px;
}

.progress-list {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.progress-card {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  padding: 24px;
  transition: all 0.3s;
}

.progress-card:hover {
  border-color: #1976d2;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.protocol-info h4 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 4px 0;
  color: #1a1a1a;
}

.peptide-name {
  font-size: 14px;
  color: #666;
  margin: 0;
}

.progress-ring-container {
  flex-shrink: 0;
}

.progress-ring {
  transform: rotate(-90deg);
}

.progress-ring-fill {
  transition: stroke-dashoffset 1s ease;
}

.progress-text {
  font-size: 16px;
  font-weight: 700;
}

.progress-details {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f9f9f9;
  border-radius: 6px;
}

.label {
  font-size: 13px;
  color: #666;
  font-weight: 500;
}

.value {
  font-size: 14px;
  font-weight: 600;
  color: #1a1a1a;
}

.value.excellent { color: #4caf50; }
.value.good { color: #2196f3; }
.value.fair { color: #ff9800; }
.value.poor { color: #f44336; }

.progress-bar-container {
  margin-bottom: 16px;
}

.progress-bar {
  height: 12px;
  background: #e0e0e0;
  border-radius: 6px;
  overflow: hidden;
}

.progress-bar-fill {
  height: 100%;
  border-radius: 6px;
  transition: width 1s ease;
}

.progress-bar-fill.high {
  background: linear-gradient(90deg, #4caf50 0%, #66bb6a 100%);
}

.progress-bar-fill.medium {
  background: linear-gradient(90deg, #2196f3 0%, #42a5f5 100%);
}

.progress-bar-fill.low {
  background: linear-gradient(90deg, #ff9800 0%, #ffa726 100%);
}

.progress-bar-fill.very-low {
  background: linear-gradient(90deg, #f44336 0%, #ef5350 100%);
}

.milestones {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.milestone {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: #f5f5f5;
  border-radius: 20px;
  font-size: 13px;
  color: #999;
  transition: all 0.3s;
}

.milestone.achieved {
  background: #e8f5e9;
  color: #4caf50;
  font-weight: 600;
}

.milestone-icon {
  font-weight: 700;
}

/* Dark mode */
:global(.dark-mode) .protocol-progress h3 {
  color: #fff;
}

:global(.dark-mode) .subtitle {
  color: #aaa;
}

:global(.dark-mode) .progress-card {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .protocol-info h4 {
  color: #fff;
}

:global(.dark-mode) .peptide-name {
  color: #aaa;
}

:global(.dark-mode) .detail-row {
  background: #1a1a1a;
}

:global(.dark-mode) .label {
  color: #aaa;
}

:global(.dark-mode) .value {
  color: #fff;
}

:global(.dark-mode) .progress-bar {
  background: #404040;
}

:global(.dark-mode) .milestone {
  background: #1a1a1a;
}

:global(.dark-mode) .milestone.achieved {
  background: #1a3a1a;
}

/* Responsive */
@media (max-width: 768px) {
  .protocol-progress {
    padding: 16px;
  }

  .card-header {
    flex-direction: column;
    gap: 16px;
  }

  .progress-details {
    grid-template-columns: 1fr;
  }

  .milestones {
    flex-direction: column;
  }
}
</style>
