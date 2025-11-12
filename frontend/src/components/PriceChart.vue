<template>
  <div class="price-chart">
    <div class="chart-header">
      <h4>📈 Price History Chart</h4>
      <div v-if="peptideNames.length > 0" class="peptide-filter">
        <label for="peptide-select">Filter:</label>
        <select id="peptide-select" v-model="selectedPeptide" @change="updateChart">
          <option value="">All Peptides</option>
          <option v-for="name in peptideNames" :key="name" :value="name">
            {{ name }}
          </option>
        </select>
      </div>
    </div>

    <div v-if="filteredData.length === 0" class="no-chart-data">
      No price data available for charting
    </div>

    <div v-else class="chart-container">
      <svg :viewBox="`0 0 ${width} ${height}`" class="chart-svg">
        <!-- Grid lines -->
        <g class="grid">
          <line
            v-for="i in 5"
            :key="`grid-${i}`"
            :x1="padding"
            :y1="padding + ((height - 2 * padding) / 4) * (i - 1)"
            :x2="width - padding"
            :y2="padding + ((height - 2 * padding) / 4) * (i - 1)"
            stroke="#e0e0e0"
            stroke-width="1"
          />
        </g>

        <!-- Y-axis labels (prices) -->
        <g class="y-axis">
          <text
            v-for="(price, i) in yAxisLabels"
            :key="`y-${i}`"
            :x="padding - 10"
            :y="padding + ((height - 2 * padding) / 4) * i + 5"
            text-anchor="end"
            class="axis-label"
          >
            ${{ price.toFixed(2) }}
          </text>
        </g>

        <!-- Data lines by peptide -->
        <g v-for="(peptideData, peptideName) in groupedByPeptide" :key="peptideName" class="data-line">
          <polyline
            :points="getLinePoints(peptideData)"
            :stroke="getColorForPeptide(peptideName)"
            stroke-width="3"
            fill="none"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <!-- Data points -->
          <circle
            v-for="(point, idx) in peptideData"
            :key="`${peptideName}-${idx}`"
            :cx="getX(idx, peptideData.length)"
            :cy="getY(point.cost_per_mg)"
            :r="5"
            :fill="getColorForPeptide(peptideName)"
            class="data-point"
            @mouseenter="showTooltip($event, point, peptideName)"
            @mouseleave="hideTooltip"
          >
            <title>{{ peptideName }}: ${{ point.cost_per_mg.toFixed(2) }}/mg on {{ formatDate(point.recorded_at) }}</title>
          </circle>
        </g>

        <!-- X-axis labels (dates) -->
        <g class="x-axis">
          <text
            v-for="(point, i) in xAxisLabels"
            :key="`x-${i}`"
            :x="padding + ((width - 2 * padding) / (xAxisLabels.length - 1)) * i"
            :y="height - padding + 20"
            text-anchor="middle"
            class="axis-label"
          >
            {{ formatDate(point.recorded_at) }}
          </text>
        </g>
      </svg>

      <!-- Legend -->
      <div class="chart-legend">
        <div
          v-for="(peptideData, peptideName) in groupedByPeptide"
          :key="`legend-${peptideName}`"
          class="legend-item"
        >
          <span class="legend-color" :style="{ backgroundColor: getColorForPeptide(peptideName) }"></span>
          <span class="legend-label">{{ peptideName }}</span>
          <span class="legend-stats">
            Latest: ${{ getLatestPrice(peptideData).toFixed(2) }}/mg
          </span>
        </div>
      </div>
    </div>

    <!-- Tooltip -->
    <div
      v-if="tooltip.visible"
      class="chart-tooltip"
      :style="{ left: tooltip.x + 'px', top: tooltip.y + 'px' }"
    >
      <div class="tooltip-title">{{ tooltip.peptideName }}</div>
      <div class="tooltip-price">${{ tooltip.price }}/mg</div>
      <div class="tooltip-date">{{ tooltip.date }}</div>
      <div v-if="tooltip.inStock !== null" class="tooltip-stock">
        {{ tooltip.inStock ? '✓ In Stock' : '✗ Out of Stock' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { PriceHistory } from '../api/peptrack';

interface Props {
  priceHistory: PriceHistory[];
}

const props = defineProps<Props>();

const width = 800;
const height = 400;
const padding = 60;

const selectedPeptide = ref('');
const tooltip = ref({
  visible: false,
  x: 0,
  y: 0,
  peptideName: '',
  price: '',
  date: '',
  inStock: null as boolean | null,
});

// Filter data by selected peptide
const filteredData = computed(() => {
  if (!selectedPeptide.value) {
    return [...props.priceHistory].sort((a, b) =>
      new Date(a.recorded_at).getTime() - new Date(b.recorded_at).getTime()
    );
  }
  return props.priceHistory
    .filter(p => p.peptide_name === selectedPeptide.value)
    .sort((a, b) =>
      new Date(a.recorded_at).getTime() - new Date(b.recorded_at).getTime()
    );
});

// Get unique peptide names
const peptideNames = computed(() => {
  const names = new Set(props.priceHistory.map(p => p.peptide_name));
  return Array.from(names).sort();
});

// Group data by peptide
const groupedByPeptide = computed(() => {
  const groups: Record<string, PriceHistory[]> = {};
  filteredData.value.forEach(entry => {
    if (!groups[entry.peptide_name]) {
      groups[entry.peptide_name] = [];
    }
    groups[entry.peptide_name]!.push(entry);
  });
  return groups;
});

// Calculate min/max prices for Y-axis
const priceRange = computed(() => {
  if (filteredData.value.length === 0) {
    return { min: 0, max: 100 };
  }
  const prices = filteredData.value.map(p => p.cost_per_mg);
  const min = Math.min(...prices);
  const max = Math.max(...prices);
  const padding = (max - min) * 0.1 || 10; // 10% padding or 10 if all same
  return {
    min: Math.max(0, min - padding),
    max: max + padding,
  };
});

// Y-axis labels (5 price points)
const yAxisLabels = computed(() => {
  const range = priceRange.value.max - priceRange.value.min;
  return Array.from({ length: 5 }, (_, i) => priceRange.value.max - (range / 4) * i);
});

// X-axis labels (dates)
const xAxisLabels = computed(() => {
  if (filteredData.value.length === 0) return [];
  // Show up to 6 evenly spaced dates
  const step = Math.max(1, Math.floor(filteredData.value.length / 6));
  return filteredData.value.filter((_, i) => i % step === 0 || i === filteredData.value.length - 1);
});

// Get X coordinate for a data point
function getX(index: number, totalPoints: number): number {
  const chartWidth = width - 2 * padding;
  return padding + (chartWidth / (totalPoints - 1 || 1)) * index;
}

// Get Y coordinate for a price
function getY(price: number): number {
  const chartHeight = height - 2 * padding;
  const range = priceRange.value.max - priceRange.value.min;
  const normalized = (priceRange.value.max - price) / range;
  return padding + chartHeight * normalized;
}

// Generate polyline points string for a peptide's data
function getLinePoints(peptideData: PriceHistory[]): string {
  return peptideData
    .map((point, idx) => {
      const x = getX(idx, peptideData.length);
      const y = getY(point.cost_per_mg);
      return `${x},${y}`;
    })
    .join(' ');
}

// Color palette for different peptides
const colors = [
  '#3498db', // blue
  '#e74c3c', // red
  '#27ae60', // green
  '#f39c12', // orange
  '#9b59b6', // purple
  '#1abc9c', // turquoise
  '#e67e22', // darker orange
  '#34495e', // dark blue-gray
];

function getColorForPeptide(peptideName: string): string {
  const index = peptideNames.value.indexOf(peptideName);
  return colors[index % colors.length] || '#3498db';
}

function getLatestPrice(peptideData: PriceHistory[]): number {
  if (peptideData.length === 0) return 0;
  return peptideData[peptideData.length - 1]!.cost_per_mg;
}

function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString);
    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return dateString;
  }
}

function showTooltip(event: MouseEvent, point: PriceHistory, peptideName: string) {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  tooltip.value = {
    visible: true,
    x: rect.left + window.scrollX + 10,
    y: rect.top + window.scrollY - 10,
    peptideName,
    price: point.cost_per_mg.toFixed(2),
    date: formatDate(point.recorded_at),
    inStock: point.in_stock !== undefined && point.in_stock !== null ? point.in_stock : null,
  };
}

function hideTooltip() {
  tooltip.value.visible = false;
}

function updateChart() {
  // Chart updates automatically via computed properties
}
</script>

<style scoped>
.price-chart {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.chart-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.chart-header h4 {
  margin: 0;
  color: #2c3e50;
}

.peptide-filter {
  display: flex;
  align-items: center;
  gap: 8px;
}

.peptide-filter label {
  font-size: 14px;
  font-weight: 600;
  color: #555;
}

.peptide-filter select {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
}

.no-chart-data {
  text-align: center;
  padding: 40px;
  color: #999;
  font-style: italic;
}

.chart-container {
  position: relative;
}

.chart-svg {
  width: 100%;
  height: auto;
  max-width: 100%;
}

.axis-label {
  font-size: 12px;
  fill: #666;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.data-point {
  cursor: pointer;
  transition: r 0.2s;
}

.data-point:hover {
  r: 7;
}

.chart-legend {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  margin-top: 20px;
  padding-top: 20px;
  border-top: 2px solid #e0e0e0;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.legend-color {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  display: inline-block;
}

.legend-label {
  font-weight: 600;
  color: #2c3e50;
}

.legend-stats {
  color: #666;
  font-size: 13px;
}

.chart-tooltip {
  position: fixed;
  background: rgba(0, 0, 0, 0.9);
  color: white;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
  pointer-events: none;
  z-index: 1000;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.tooltip-title {
  font-weight: 700;
  margin-bottom: 4px;
  font-size: 14px;
}

.tooltip-price {
  font-size: 16px;
  font-weight: 600;
  color: #4CAF50;
  margin-bottom: 2px;
}

.tooltip-date {
  font-size: 12px;
  opacity: 0.8;
  margin-bottom: 4px;
}

.tooltip-stock {
  font-size: 12px;
  margin-top: 4px;
  padding-top: 4px;
  border-top: 1px solid rgba(255, 255, 255, 0.2);
}

@media (max-width: 768px) {
  .chart-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .chart-legend {
    flex-direction: column;
  }
}

@media (prefers-color-scheme: dark) {
  .price-chart {
    background: #2a2a2a;
  }

  .chart-header h4 {
    color: #e0e0e0;
  }

  .peptide-filter label {
    color: #aaa;
  }

  .peptide-filter select {
    background: #3a3a3a;
    color: #e0e0e0;
    border-color: #4a4a4a;
  }

  .axis-label {
    fill: #aaa;
  }

  .legend-label {
    color: #e0e0e0;
  }

  .legend-stats {
    color: #aaa;
  }

  .chart-legend {
    border-top-color: #4a4a4a;
  }
}
</style>
