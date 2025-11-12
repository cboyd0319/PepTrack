<template>
  <div class="dose-heatmap">
    <div class="heatmap-header">
      <h3>📅 Dose Calendar</h3>
      <div class="heatmap-controls">
        <select v-model="selectedProtocol" @change="loadDoseData" class="protocol-select">
          <option value="">All Protocols</option>
          <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
            {{ protocol.name }}
          </option>
        </select>
        <div class="legend">
          <span class="legend-label">Less</span>
          <div class="legend-box level-0"></div>
          <div class="legend-box level-1"></div>
          <div class="legend-box level-2"></div>
          <div class="legend-box level-3"></div>
          <div class="legend-box level-4"></div>
          <span class="legend-label">More</span>
        </div>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading dose history...</div>

    <div v-else class="heatmap-container">
      <!-- Month labels -->
      <div class="month-labels">
        <div
          v-for="month in monthLabels"
          :key="month.label"
          class="month-label"
          :style="{ gridColumn: `${month.startCol} / span ${month.span}` }"
        >
          {{ month.label }}
        </div>
      </div>

      <!-- Weekday labels -->
      <div class="weekday-labels">
        <div class="weekday-label">Mon</div>
        <div class="weekday-label"></div>
        <div class="weekday-label">Wed</div>
        <div class="weekday-label"></div>
        <div class="weekday-label">Fri</div>
        <div class="weekday-label"></div>
        <div class="weekday-label">Sun</div>
      </div>

      <!-- Heatmap grid -->
      <div class="heatmap-grid">
        <div
          v-for="day in days"
          :key="day.date"
          class="day-cell"
          :class="`level-${day.level}`"
          :data-date="day.date"
          :data-count="day.count"
          :title="getDayTooltip(day)"
          @mouseenter="showTooltip($event, day)"
          @mouseleave="hideTooltip"
        >
        </div>
      </div>

      <!-- Tooltip -->
      <div v-if="tooltip.visible" class="heatmap-tooltip" :style="tooltipStyle">
        <div class="tooltip-date">{{ tooltip.date }}</div>
        <div class="tooltip-count">
          {{ tooltip.count }} dose{{ tooltip.count !== 1 ? 's' : '' }}
        </div>
        <div v-if="tooltip.streak" class="tooltip-streak">
          🔥 {{ tooltip.streak }} day streak
        </div>
      </div>
    </div>

    <!-- Stats summary -->
    <div class="heatmap-stats">
      <div class="stat-card">
        <div class="stat-value">{{ stats.totalDoses }}</div>
        <div class="stat-label">Total Doses</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.currentStreak }}</div>
        <div class="stat-label">Current Streak</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.longestStreak }}</div>
        <div class="stat-label">Longest Streak</div>
      </div>
      <div class="stat-card">
        <div class="stat-value">{{ stats.adherenceRate }}%</div>
        <div class="stat-label">Adherence Rate</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { listProtocols, listDoseLogs, type PeptideProtocol, type DoseLog } from '../api/peptrack';

interface DayData {
  date: string;
  count: number;
  level: number; // 0-4 for color intensity
  weekday: number; // 0-6 (Sun-Sat)
}

interface MonthLabel {
  label: string;
  startCol: number;
  span: number;
}

interface Stats {
  totalDoses: number;
  currentStreak: number;
  longestStreak: number;
  adherenceRate: number;
}

const protocols = ref<PeptideProtocol[]>([]);
const selectedProtocol = ref<string>('');
const loading = ref(false);
const days = ref<DayData[]>([]);
const stats = ref<Stats>({
  totalDoses: 0,
  currentStreak: 0,
  longestStreak: 0,
  adherenceRate: 0,
});

const tooltip = ref({
  visible: false,
  date: '',
  count: 0,
  streak: 0,
  x: 0,
  y: 0,
});

const tooltipStyle = computed(() => ({
  left: `${tooltip.value.x}px`,
  top: `${tooltip.value.y}px`,
}));

const monthLabels = computed<MonthLabel[]>(() => {
  const labels: MonthLabel[] = [];
  let currentMonth = '';
  let startCol = 1;
  let span = 0;

  days.value.forEach((day, index) => {
    const date = new Date(day.date);
    const monthName = date.toLocaleDateString('en-US', { month: 'short' });

    if (monthName !== currentMonth) {
      if (currentMonth !== '') {
        labels.push({ label: currentMonth, startCol, span });
      }
      currentMonth = monthName;
      startCol = Math.floor(index / 7) + 1;
      span = 1;
    } else {
      span++;
    }
  });

  if (currentMonth !== '') {
    labels.push({ label: currentMonth, startCol, span });
  }

  return labels;
});

onMounted(async () => {
  await loadProtocols();
  await loadDoseData();
});

async function loadProtocols() {
  try {
    protocols.value = await listProtocols();
  } catch (error) {
    console.error('Failed to load protocols:', error);
  }
}

async function loadDoseData() {
  loading.value = true;
  try {
    const doses = await listDoseLogs();

    // Filter by selected protocol if any
    const filteredDoses = selectedProtocol.value
      ? doses.filter(d => d.protocol_id === selectedProtocol.value)
      : doses;

    // Generate last 365 days
    const daysMap = new Map<string, number>();
    const today = new Date();
    const startDate = new Date(today);
    startDate.setDate(today.getDate() - 364); // 365 days including today

    // Initialize all days with 0
    for (let i = 0; i < 365; i++) {
      const date = new Date(startDate);
      date.setDate(startDate.getDate() + i);
      const dateStr = date.toISOString().split('T')[0]!;
      daysMap.set(dateStr, 0);
    }

    // Count doses per day
    filteredDoses.forEach(dose => {
      const dateStr = dose.logged_at.split('T')[0]!;
      if (daysMap.has(dateStr)) {
        daysMap.set(dateStr, (daysMap.get(dateStr) || 0) + 1);
      }
    });

    // Convert to array and calculate levels
    const maxCount = Math.max(...Array.from(daysMap.values()), 1);
    const daysArray: DayData[] = [];

    // Start from the first Sunday before our range to align weeks
    const firstDate = new Date(startDate);
    const dayOfWeek = firstDate.getDay();
    firstDate.setDate(firstDate.getDate() - dayOfWeek);

    for (let i = 0; i < 371; i++) { // 53 weeks * 7 days
      const date = new Date(firstDate);
      date.setDate(firstDate.getDate() + i);
      const dateStr = date.toISOString().split('T')[0]!;
      const count = daysMap.get(dateStr) || 0;

      let level = 0;
      if (count > 0) {
        level = Math.min(4, Math.ceil((count / maxCount) * 4));
      }

      daysArray.push({
        date: dateStr,
        count,
        level,
        weekday: date.getDay(),
      });
    }

    days.value = daysArray;

    // Calculate stats
    calculateStats(filteredDoses);
  } catch (error) {
    console.error('Failed to load dose data:', error);
  } finally {
    loading.value = false;
  }
}

function calculateStats(doses: DoseLog[]) {
  stats.value.totalDoses = doses.length;

  // Calculate streaks
  const doseDates = new Set(doses.map(d => d.logged_at.split('T')[0]));
  let currentStreak = 0;
  let longestStreak = 0;
  let streak = 0;

  const today = new Date();
  for (let i = 0; i < 365; i++) {
    const date = new Date(today);
    date.setDate(today.getDate() - i);
    const dateStr = date.toISOString().split('T')[0];

    if (doseDates.has(dateStr)) {
      streak++;
      if (i === 0 || currentStreak > 0) {
        currentStreak = streak;
      }
      longestStreak = Math.max(longestStreak, streak);
    } else {
      if (currentStreak > 0) break; // Stop counting current streak
      streak = 0;
    }
  }

  stats.value.currentStreak = currentStreak;
  stats.value.longestStreak = longestStreak;

  // Calculate adherence rate (assuming daily dosing is expected)
  const daysWithDoses = doseDates.size;
  stats.value.adherenceRate = Math.round((daysWithDoses / 365) * 100);
}

function getDayTooltip(day: DayData): string {
  const date = new Date(day.date).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric'
  });
  return `${date}: ${day.count} dose${day.count !== 1 ? 's' : ''}`;
}

function showTooltip(event: MouseEvent, day: DayData) {
  const rect = (event.target as HTMLElement).getBoundingClientRect();
  tooltip.value = {
    visible: true,
    date: new Date(day.date).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    }),
    count: day.count,
    streak: 0, // Could calculate mini-streak here
    x: rect.left + rect.width / 2,
    y: rect.top - 10,
  };
}

function hideTooltip() {
  tooltip.value.visible = false;
}
</script>

<style scoped>
.dose-heatmap {
  padding: 24px;
  background: white;
  border-radius: 12px;
  border: 2px solid #e0e0e0;
}

.heatmap-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 16px;
}

.heatmap-header h3 {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: #1a1a1a;
}

.heatmap-controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.protocol-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  background: white;
  cursor: pointer;
}

.legend {
  display: flex;
  align-items: center;
  gap: 4px;
}

.legend-label {
  font-size: 12px;
  color: #666;
}

.legend-box {
  width: 12px;
  height: 12px;
  border: 1px solid #d0d7de;
  border-radius: 2px;
}

.legend-box.level-0 { background: #ebedf0; }
.legend-box.level-1 { background: #9be9a8; }
.legend-box.level-2 { background: #40c463; }
.legend-box.level-3 { background: #30a14e; }
.legend-box.level-4 { background: #216e39; }

.loading {
  text-align: center;
  padding: 40px;
  color: #666;
}

.heatmap-container {
  position: relative;
  overflow-x: auto;
  padding: 16px 0;
}

.month-labels {
  display: grid;
  grid-template-columns: repeat(53, 12px);
  gap: 3px;
  margin-bottom: 4px;
  margin-left: 32px;
}

.month-label {
  font-size: 11px;
  color: #666;
  text-align: left;
}

.weekday-labels {
  position: absolute;
  left: 0;
  top: 36px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.weekday-label {
  height: 12px;
  font-size: 10px;
  color: #666;
  line-height: 12px;
}

.heatmap-grid {
  display: grid;
  grid-template-columns: repeat(53, 12px);
  grid-template-rows: repeat(7, 12px);
  gap: 3px;
  grid-auto-flow: column;
  margin-left: 32px;
}

.day-cell {
  width: 12px;
  height: 12px;
  border: 1px solid #d0d7de;
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.2s;
}

.day-cell:hover {
  border-color: #1976d2;
  transform: scale(1.3);
  z-index: 10;
}

.day-cell.level-0 { background: #ebedf0; }
.day-cell.level-1 { background: #9be9a8; }
.day-cell.level-2 { background: #40c463; }
.day-cell.level-3 { background: #30a14e; }
.day-cell.level-4 { background: #216e39; }

/* Tooltip */
.heatmap-tooltip {
  position: fixed;
  background: #1a1a1a;
  color: white;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 12px;
  pointer-events: none;
  z-index: 1000;
  transform: translate(-50%, -100%);
  white-space: nowrap;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tooltip-date {
  font-weight: 600;
  margin-bottom: 4px;
}

.tooltip-count {
  color: #ccc;
}

.tooltip-streak {
  margin-top: 4px;
  color: #ff9800;
  font-weight: 600;
}

/* Stats */
.heatmap-stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
  margin-top: 24px;
}

.stat-card {
  padding: 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 10px;
  text-align: center;
  color: white;
}

.stat-value {
  font-size: 32px;
  font-weight: 700;
  margin-bottom: 4px;
}

.stat-label {
  font-size: 13px;
  opacity: 0.9;
}

/* Dark mode */
:global(.dark-mode) .dose-heatmap {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .heatmap-header h3 {
  color: #fff;
}

:global(.dark-mode) .protocol-select {
  background: #1a1a1a;
  border-color: #404040;
  color: #fff;
}

:global(.dark-mode) .day-cell {
  border-color: #404040;
}

/* Responsive */
@media (max-width: 768px) {
  .dose-heatmap {
    padding: 16px;
  }

  .heatmap-header {
    flex-direction: column;
    align-items: stretch;
  }

  .heatmap-container {
    overflow-x: scroll;
  }

  .heatmap-stats {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
