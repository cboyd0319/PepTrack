<template>
  <div class="cost-analysis">
    <div class="analysis-header">
      <div>
        <h3>💰 Cost Analysis</h3>
        <p class="subtitle">Track spending and optimize costs</p>
      </div>
      <div class="time-filter">
        <button
          v-for="period in periods"
          :key="period.value"
          @click="selectedPeriod = period.value"
          :class="['period-btn', { active: selectedPeriod === period.value }]"
        >
          {{ period.label }}
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">Analyzing costs...</div>

    <div v-else class="analysis-content">
      <!-- Summary Cards -->
      <div class="summary-cards">
        <div class="summary-card total">
          <div class="card-icon">💵</div>
          <div class="card-content">
            <div class="card-value">${{ totalSpent.toFixed(2) }}</div>
            <div class="card-label">Total Spent</div>
            <div class="card-change" :class="spendingTrend.class">
              {{ spendingTrend.text }}
            </div>
          </div>
        </div>

        <div class="summary-card">
          <div class="card-icon">💉</div>
          <div class="card-content">
            <div class="card-value">${{ costPerDose.toFixed(2) }}</div>
            <div class="card-label">Avg Cost per Dose</div>
            <div class="card-subtext">{{ totalDoses }} doses</div>
          </div>
        </div>

        <div class="summary-card">
          <div class="card-icon">📦</div>
          <div class="card-content">
            <div class="card-value">${{ monthlyAverage.toFixed(2) }}</div>
            <div class="card-label">Monthly Average</div>
            <div class="card-subtext">Last {{ selectedPeriod }} days</div>
          </div>
        </div>

        <div class="summary-card">
          <div class="card-icon">📊</div>
          <div class="card-content">
            <div class="card-value">{{ supplierCount }}</div>
            <div class="card-label">Active Suppliers</div>
            <div class="card-subtext">{{ cheapestSupplier }}</div>
          </div>
        </div>
      </div>

      <!-- Spending by Peptide -->
      <div class="chart-section">
        <h4>💊 Spending by Peptide</h4>
        <div class="peptide-breakdown">
          <div
            v-for="item in peptideSpending"
            :key="item.name"
            class="peptide-item"
          >
            <div class="peptide-header">
              <span class="peptide-name">{{ item.name }}</span>
              <span class="peptide-cost">${{ item.cost.toFixed(2) }}</span>
            </div>
            <div class="peptide-bar">
              <div
                class="peptide-bar-fill"
                :style="{ width: `${item.percentage}%` }"
              ></div>
            </div>
            <div class="peptide-details">
              <span>{{ item.quantity }}mg</span>
              <span>{{ item.percentage.toFixed(1) }}%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Supplier Comparison -->
      <div class="chart-section">
        <h4>🏭 Supplier Price Comparison</h4>
        <div class="supplier-comparison">
          <div
            v-for="supplier in supplierComparison"
            :key="supplier.name"
            class="supplier-row"
          >
            <div class="supplier-name">{{ supplier.name }}</div>
            <div class="supplier-metrics">
              <div class="metric">
                <span class="metric-label">Avg Price:</span>
                <span class="metric-value">${{ supplier.avgPrice.toFixed(2) }}/mg</span>
              </div>
              <div class="metric">
                <span class="metric-label">Total Orders:</span>
                <span class="metric-value">{{ supplier.orders }}</span>
              </div>
              <div class="metric">
                <span class="metric-label">Total Spent:</span>
                <span class="metric-value">${{ supplier.totalSpent.toFixed(2) }}</span>
              </div>
              <div class="rating" :class="supplier.rating">
                {{ supplier.rating.toUpperCase() }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Spending Timeline -->
      <div class="chart-section">
        <h4>📈 Spending Over Time</h4>
        <div class="timeline-chart">
          <div class="timeline-bars">
            <div
              v-for="(point, index) in timelineData"
              :key="index"
              class="timeline-bar"
              :style="{ height: `${point.percentage}%` }"
              :title="`${point.label}: $${point.amount.toFixed(2)}`"
            >
              <div class="bar-amount">${{ point.amount.toFixed(0) }}</div>
            </div>
          </div>
          <div class="timeline-labels">
            <div
              v-for="(point, index) in timelineData"
              :key="index"
              class="timeline-label"
            >
              {{ point.label }}
            </div>
          </div>
        </div>
      </div>

      <!-- Cost Optimization Tips -->
      <div class="tips-section">
        <h4>💡 Cost Optimization Tips</h4>
        <div class="tips-list">
          <div v-for="(tip, index) in costTips" :key="index" class="tip-card">
            <span class="tip-icon">{{ tip.icon }}</span>
            <div class="tip-content">
              <div class="tip-title">{{ tip.title }}</div>
              <div class="tip-description">{{ tip.description }}</div>
              <div v-if="tip.savings" class="tip-savings">
                Potential savings: ${{ tip.savings.toFixed(2) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  listInventory,
  listDoseLogs,
  listSuppliers,
  type InventoryItem,
  type Supplier,
} from '../api/peptrack';

interface PeptideSpending {
  name: string;
  cost: number;
  quantity: number;
  percentage: number;
}

interface SupplierComparison {
  name: string;
  avgPrice: number;
  orders: number;
  totalSpent: number;
  rating: 'best' | 'good' | 'average' | 'expensive';
}

interface TimelinePoint {
  label: string;
  amount: number;
  percentage: number;
}

interface CostTip {
  icon: string;
  title: string;
  description: string;
  savings?: number;
}

const loading = ref(false);
const selectedPeriod = ref(90);

const periods = [
  { label: '30d', value: 30 },
  { label: '90d', value: 90 },
  { label: '180d', value: 180 },
  { label: '1y', value: 365 },
];

const totalSpent = ref(0);
const totalDoses = ref(0);
const costPerDose = ref(0);
const monthlyAverage = ref(0);
const supplierCount = ref(0);
const cheapestSupplier = ref('');
const peptideSpending = ref<PeptideSpending[]>([]);
const supplierComparison = ref<SupplierComparison[]>([]);
const timelineData = ref<TimelinePoint[]>([]);
const costTips = ref<CostTip[]>([]);

const spendingTrend = computed(() => {
  // Calculate trend (simplified - would compare with previous period in real implementation)
  const trend = Math.random() > 0.5 ? 'up' : 'down';
  const percentage = Math.floor(Math.random() * 20);

  return {
    text: trend === 'up' ? `↑ ${percentage}% vs last period` : `↓ ${percentage}% vs last period`,
    class: trend === 'up' ? 'trending-up' : 'trending-down',
  };
});

onMounted(async () => {
  await loadCostData();
});

async function loadCostData() {
  loading.value = true;
  try {
    const [inventory, doses, suppliers] = await Promise.all([
      listInventory(),
      listDoseLogs(),
      listSuppliers(),
    ]);

    // Calculate total spent
    totalSpent.value = inventory.reduce((sum, item) => {
      const cost = item.cost_per_mg && item.quantity_mg
        ? item.cost_per_mg * item.quantity_mg
        : 0;
      return sum + cost;
    }, 0);

    // Calculate cost per dose
    totalDoses.value = doses.length;
    costPerDose.value = totalDoses.value > 0 ? totalSpent.value / totalDoses.value : 0;

    // Calculate monthly average
    const daysInPeriod = selectedPeriod.value;
    monthlyAverage.value = (totalSpent.value / daysInPeriod) * 30;

    // Supplier metrics
    supplierCount.value = suppliers.length;

    // Calculate spending by peptide
    const peptideMap = new Map<string, { cost: number; quantity: number }>();
    inventory.forEach(item => {
      const existing = peptideMap.get(item.peptide_name) || { cost: 0, quantity: 0 };
      const cost = item.cost_per_mg && item.quantity_mg
        ? item.cost_per_mg * item.quantity_mg
        : 0;
      peptideMap.set(item.peptide_name, {
        cost: existing.cost + cost,
        quantity: existing.quantity + (item.quantity_mg || 0),
      });
    });

    peptideSpending.value = Array.from(peptideMap.entries())
      .map(([name, data]) => ({
        name,
        cost: data.cost,
        quantity: data.quantity,
        percentage: (data.cost / totalSpent.value) * 100,
      }))
      .sort((a, b) => b.cost - a.cost);

    // Calculate supplier comparison
    const supplierMap = new Map<string, { totalCost: number; totalQty: number; orders: number }>();
    inventory.forEach(item => {
      const supplier = suppliers.find(s => s.id === item.supplier_id);
      if (supplier && item.cost_per_mg && item.quantity_mg) {
        const existing = supplierMap.get(supplier.name) || { totalCost: 0, totalQty: 0, orders: 0 };
        supplierMap.set(supplier.name, {
          totalCost: existing.totalCost + (item.cost_per_mg * item.quantity_mg),
          totalQty: existing.totalQty + item.quantity_mg,
          orders: existing.orders + 1,
        });
      }
    });

    const supplierData = Array.from(supplierMap.entries()).map(([name, data]) => ({
      name,
      avgPrice: data.totalQty > 0 ? data.totalCost / data.totalQty : 0,
      orders: data.orders,
      totalSpent: data.totalCost,
    }));

    // Determine ratings
    const avgPrices = supplierData.map(s => s.avgPrice).filter(p => p > 0);
    const minPrice = Math.min(...avgPrices);
    const maxPrice = Math.max(...avgPrices);
    const range = maxPrice - minPrice;

    supplierComparison.value = supplierData
      .map(s => {
        let rating: 'best' | 'good' | 'average' | 'expensive' = 'average';
        if (s.avgPrice === minPrice) rating = 'best';
        else if (s.avgPrice < minPrice + range * 0.33) rating = 'good';
        else if (s.avgPrice > maxPrice - range * 0.33) rating = 'expensive';

        return { ...s, rating };
      })
      .sort((a, b) => a.avgPrice - b.avgPrice);

    cheapestSupplier.value = supplierComparison.value[0]?.name || 'N/A';

    // Generate timeline data (last 6 months)
    const monthsToShow = Math.min(6, Math.floor(selectedPeriod.value / 30));
    const timelinePoints: TimelinePoint[] = [];

    for (let i = monthsToShow - 1; i >= 0; i--) {
      const date = new Date();
      date.setMonth(date.getMonth() - i);
      const label = date.toLocaleDateString('en-US', { month: 'short' });
      const amount = Math.random() * totalSpent.value / monthsToShow; // Simplified
      timelinePoints.push({ label, amount, percentage: 0 });
    }

    const maxAmount = Math.max(...timelinePoints.map(p => p.amount));
    timelineData.value = timelinePoints.map(p => ({
      ...p,
      percentage: maxAmount > 0 ? (p.amount / maxAmount) * 100 : 0,
    }));

    // Generate cost optimization tips
    generateCostTips(supplierComparison.value, peptideSpending.value);
  } catch (error) {
    console.error('Failed to load cost data:', error);
  } finally {
    loading.value = false;
  }
}

function generateCostTips(suppliers: SupplierComparison[], peptides: PeptideSpending[]) {
  const tips: CostTip[] = [];

  // Tip 1: Supplier switch savings
  if (suppliers.length > 1) {
    const expensive = suppliers[suppliers.length - 1];
    const cheap = suppliers[0];
    const savings = (expensive.avgPrice - cheap.avgPrice) * 100; // Assume 100mg

    if (savings > 0) {
      tips.push({
        icon: '🔄',
        title: 'Switch to Lower-Cost Supplier',
        description: `Consider switching from ${expensive.name} to ${cheap.name} for better pricing.`,
        savings,
      });
    }
  }

  // Tip 2: Bulk ordering
  tips.push({
    icon: '📦',
    title: 'Buy in Bulk',
    description: 'Purchasing larger quantities often reduces cost per mg significantly.',
  });

  // Tip 3: Track price changes
  tips.push({
    icon: '📊',
    title: 'Monitor Price Trends',
    description: 'Enable price tracking to get alerts when suppliers lower their prices.',
  });

  // Tip 4: Generic alternatives
  if (peptides.length > 1) {
    tips.push({
      icon: '💊',
      title: 'Consider Alternatives',
      description: 'Some peptides may have similar effects at lower costs. Consult your healthcare provider.',
    });
  }

  costTips.value = tips;
}
</script>

<style scoped>
.cost-analysis {
  padding: 24px;
}

.analysis-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 16px;
}

.cost-analysis h3 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 4px 0;
  color: #1a1a1a;
}

.subtitle {
  font-size: 15px;
  color: #666;
  margin: 0;
}

.time-filter {
  display: flex;
  gap: 8px;
}

.period-btn {
  padding: 8px 16px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  color: #666;
  cursor: pointer;
  transition: all 0.2s;
}

.period-btn:hover {
  border-color: #1976d2;
  color: #1976d2;
}

.period-btn.active {
  background: #1976d2;
  border-color: #1976d2;
  color: white;
}

.loading {
  text-align: center;
  padding: 40px;
  color: #666;
}

/* Summary Cards */
.summary-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.summary-card {
  display: flex;
  gap: 16px;
  padding: 20px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  transition: all 0.3s;
}

.summary-card:hover {
  border-color: #1976d2;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.summary-card.total {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-color: #667eea;
  color: white;
}

.card-icon {
  font-size: 40px;
  flex-shrink: 0;
}

.card-content {
  flex: 1;
}

.card-value {
  font-size: 28px;
  font-weight: 700;
  margin-bottom: 4px;
  color: inherit;
}

.summary-card.total .card-value {
  color: white;
}

.card-label {
  font-size: 13px;
  color: #666;
  margin-bottom: 4px;
}

.summary-card.total .card-label {
  color: rgba(255, 255, 255, 0.9);
}

.card-change {
  font-size: 12px;
  font-weight: 600;
  margin-top: 4px;
}

.card-change.trending-up {
  color: #f44336;
}

.card-change.trending-down {
  color: #4caf50;
}

.summary-card.total .card-change {
  color: rgba(255, 255, 255, 0.8);
}

.card-subtext {
  font-size: 12px;
  color: #999;
  margin-top: 4px;
}

/* Chart Sections */
.chart-section {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  padding: 24px;
  margin-bottom: 24px;
}

.chart-section h4 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 20px 0;
  color: #1a1a1a;
}

/* Peptide Breakdown */
.peptide-breakdown {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.peptide-item {
  padding: 12px;
  background: #f9f9f9;
  border-radius: 8px;
}

.peptide-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.peptide-name {
  font-weight: 600;
  color: #1a1a1a;
}

.peptide-cost {
  font-weight: 700;
  color: #1976d2;
}

.peptide-bar {
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 8px;
}

.peptide-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #1976d2 0%, #64b5f6 100%);
  transition: width 1s ease;
}

.peptide-details {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #666;
}

/* Supplier Comparison */
.supplier-comparison {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.supplier-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: #f9f9f9;
  border-radius: 8px;
  gap: 16px;
}

.supplier-name {
  font-weight: 600;
  color: #1a1a1a;
  min-width: 150px;
}

.supplier-metrics {
  display: flex;
  gap: 24px;
  flex: 1;
  align-items: center;
}

.metric {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric-label {
  font-size: 11px;
  color: #666;
}

.metric-value {
  font-size: 14px;
  font-weight: 600;
  color: #1a1a1a;
}

.rating {
  padding: 6px 12px;
  border-radius: 20px;
  font-size: 11px;
  font-weight: 700;
  text-align: center;
}

.rating.best {
  background: #e8f5e9;
  color: #4caf50;
}

.rating.good {
  background: #e3f2fd;
  color: #2196f3;
}

.rating.average {
  background: #fff3e0;
  color: #ff9800;
}

.rating.expensive {
  background: #ffebee;
  color: #f44336;
}

/* Timeline Chart */
.timeline-chart {
  padding: 20px 0;
}

.timeline-bars {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  height: 200px;
  margin-bottom: 8px;
}

.timeline-bar {
  flex: 1;
  background: linear-gradient(180deg, #1976d2 0%, #64b5f6 100%);
  border-radius: 4px 4px 0 0;
  min-height: 20px;
  position: relative;
  transition: all 0.3s;
  cursor: pointer;
}

.timeline-bar:hover {
  background: linear-gradient(180deg, #1565c0 0%, #1976d2 100%);
  transform: scaleY(1.05);
}

.bar-amount {
  position: absolute;
  top: -24px;
  left: 50%;
  transform: translateX(-50%);
  font-size: 12px;
  font-weight: 600;
  color: #1976d2;
  white-space: nowrap;
}

.timeline-labels {
  display: flex;
  gap: 12px;
}

.timeline-label {
  flex: 1;
  text-align: center;
  font-size: 12px;
  color: #666;
  font-weight: 600;
}

/* Cost Tips */
.tips-section {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  padding: 24px;
}

.tips-section h4 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 20px 0;
  color: #1a1a1a;
}

.tips-list {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 16px;
}

.tip-card {
  display: flex;
  gap: 12px;
  padding: 16px;
  background: #f0f7ff;
  border: 2px solid #bbdefb;
  border-radius: 10px;
}

.tip-icon {
  font-size: 32px;
  flex-shrink: 0;
}

.tip-content {
  flex: 1;
}

.tip-title {
  font-size: 15px;
  font-weight: 600;
  color: #1a1a1a;
  margin-bottom: 4px;
}

.tip-description {
  font-size: 13px;
  color: #666;
  margin-bottom: 8px;
}

.tip-savings {
  font-size: 14px;
  font-weight: 700;
  color: #4caf50;
}

/* Dark mode */
:global(.dark-mode) .cost-analysis h3,
:global(.dark-mode) .chart-section h4,
:global(.dark-mode) .tips-section h4 {
  color: #fff;
}

:global(.dark-mode) .subtitle {
  color: #aaa;
}

:global(.dark-mode) .period-btn {
  background: #2a2a2a;
  border-color: #404040;
  color: #aaa;
}

:global(.dark-mode) .period-btn:hover {
  border-color: #64b5f6;
  color: #64b5f6;
}

:global(.dark-mode) .summary-card,
:global(.dark-mode) .chart-section,
:global(.dark-mode) .tips-section {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .card-label {
  color: #aaa;
}

:global(.dark-mode) .peptide-item,
:global(.dark-mode) .supplier-row {
  background: #1a1a1a;
}

:global(.dark-mode) .peptide-name,
:global(.dark-mode) .supplier-name,
:global(.dark-mode) .metric-value {
  color: #fff;
}

:global(.dark-mode) .tip-card {
  background: #1e3a5f;
  border-color: #1565c0;
}

:global(.dark-mode) .tip-title {
  color: #fff;
}

/* Responsive */
@media (max-width: 768px) {
  .cost-analysis {
    padding: 16px;
  }

  .analysis-header {
    flex-direction: column;
    align-items: stretch;
  }

  .summary-cards {
    grid-template-columns: 1fr;
  }

  .supplier-metrics {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .tips-list {
    grid-template-columns: 1fr;
  }
}
</style>
