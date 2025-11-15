<template>
  <div class="report-generator">
    <h2>📄 Report Generator</h2>
    <p class="subtitle">Generate comprehensive PDF reports of your data</p>

    <!-- Report Configuration -->
    <div class="panel config-section">
      <h3>Report Configuration</h3>

      <div class="form-group">
        <label for="report-title">
          Report Title
          <input
            id="report-title"
            v-model="reportTitle"
            type="text"
            placeholder="e.g., Monthly Protocol Summary"
          />
        </label>
      </div>

      <div class="form-group">
        <label>Include in Report:</label>
        <div class="checkbox-group">
          <label>
            <input v-model="includeProtocols" type="checkbox" />
            Protocols & Details
          </label>
          <label>
            <input v-model="includeDoseLogs" type="checkbox" />
            Dose Logs
          </label>
          <label>
            <input v-model="includeBodyMetrics" type="checkbox" />
            Body Metrics
          </label>
          <label>
            <input v-model="includeSideEffects" type="checkbox" />
            Side Effects
          </label>
          <label>
            <input v-model="includeInventory" type="checkbox" />
            Inventory Status
          </label>
        </div>
      </div>

      <div class="form-group">
        <label for="date-range">
          Date Range (for dose logs)
          <select id="date-range" v-model="dateRange">
            <option value="7">Last 7 days</option>
            <option value="30">Last 30 days</option>
            <option value="90">Last 90 days</option>
            <option value="all">All time</option>
          </select>
        </label>
      </div>

      <button
        @click="generateReport"
        class="generate-btn"
        :disabled="isGenerating || !hasSelection"
      >
        {{ isGenerating ? '⏳ Generating...' : '📊 Generate Report' }}
      </button>
    </div>

    <!-- Report Preview/Output -->
    <div v-if="reportGenerated" class="report-actions">
      <button @click="printReport" class="print-btn">
        🖨️ Print / Export to PDF
      </button>
      <button @click="clearReport" class="clear-btn">
        Clear Report
      </button>
    </div>

    <!-- Print-Ready Report -->
    <div v-if="reportGenerated" id="printable-report" class="report-content">
      <div class="report-header">
        <h1>{{ reportTitle || 'PepTrack Report' }}</h1>
        <p class="report-date">Generated on {{ formatDate(new Date().toISOString()) }}</p>
      </div>

      <!-- Protocols Section -->
      <div v-if="includeProtocols && protocols.length > 0" class="report-section">
        <h2>Protocols</h2>
        <table class="data-table">
          <thead>
            <tr>
              <th>Protocol Name</th>
              <th>Peptide</th>
              <th>Dosage</th>
              <th>Frequency</th>
              <th>Route</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="protocol in protocols" :key="protocol.id">
              <td>{{ protocol.name }}</td>
              <td>{{ protocol.peptide_name }}</td>
              <td>{{ protocol.dose_amount_mg }} {{ protocol.dose_unit || 'mg' }}</td>
              <td>{{ protocol.frequency || 'N/A' }}</td>
              <td>{{ protocol.route_of_administration || 'N/A' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Dose Logs Section -->
      <div v-if="includeDoseLogs && filteredDoseLogs.length > 0" class="report-section">
        <h2>Dose Logs ({{ dateRangeLabel }})</h2>
        <table class="data-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>Protocol</th>
              <th>Amount</th>
              <th>Site</th>
              <th>Notes</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="log in filteredDoseLogs" :key="log.id">
              <td>{{ formatDate(log.timestamp) }}</td>
              <td>{{ getProtocolName(log.protocol_id) }}</td>
              <td>{{ log.amount_mg }} mg</td>
              <td>{{ log.injection_site || 'N/A' }}</td>
              <td>{{ log.notes || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Body Metrics Section -->
      <div v-if="includeBodyMetrics && bodyMetrics.length > 0" class="report-section">
        <h2>Body Metrics</h2>
        <table class="data-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>Weight (kg)</th>
              <th>Body Fat (%)</th>
              <th>Muscle Mass (kg)</th>
              <th>Waist (cm)</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="metric in bodyMetrics" :key="metric.id">
              <td>{{ formatDate(metric.date) }}</td>
              <td>{{ metric.weight_kg || '-' }}</td>
              <td>{{ metric.body_fat_percentage || '-' }}</td>
              <td>{{ metric.muscle_mass_kg || '-' }}</td>
              <td>{{ metric.waist_cm || '-' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Side Effects Section -->
      <div v-if="includeSideEffects && sideEffects.length > 0" class="report-section">
        <h2>Side Effects</h2>
        <table class="data-table">
          <thead>
            <tr>
              <th>Date</th>
              <th>Symptom</th>
              <th>Severity</th>
              <th>Duration</th>
              <th>Resolved</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="effect in sideEffects" :key="effect.id">
              <td>{{ formatDate(effect.date) }}</td>
              <td>{{ effect.symptom }}</td>
              <td>{{ effect.severity }}</td>
              <td>{{ effect.duration_minutes ? `${effect.duration_minutes} min` : 'N/A' }}</td>
              <td>{{ effect.resolved ? 'Yes' : 'No' }}</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- Inventory Section -->
      <div v-if="includeInventory && inventory.length > 0" class="report-section">
        <h2>Inventory Status</h2>
        <table class="data-table">
          <thead>
            <tr>
              <th>Protocol</th>
              <th>Status</th>
              <th>Quantity Remaining</th>
              <th>Expiry Date</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="item in inventory" :key="item.id">
              <td>{{ getProtocolName(item.protocol_id) }}</td>
              <td>{{ item.vial_status || 'Unknown' }}</td>
              <td>{{ item.quantity_remaining_mg ? `${item.quantity_remaining_mg} mg` : 'N/A' }}</td>
              <td>{{ item.expiry_date ? formatDate(item.expiry_date) : 'N/A' }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';
import {
  listProtocols,
  listDoseLogs,
  listBodyMetrics,
  listSideEffects,
  listInventory,
  type PeptideProtocol,
  type DoseLog,
  type BodyMetric,
  type SideEffect,
  type InventoryItem,
} from '../api/peptrack';

// State
const reportTitle = ref('');
const includeProtocols = ref(true);
const includeDoseLogs = ref(true);
const includeBodyMetrics = ref(false);
const includeSideEffects = ref(false);
const includeInventory = ref(false);
const dateRange = ref('30');
const isGenerating = ref(false);
const reportGenerated = ref(false);

// Data
const protocols = ref<PeptideProtocol[]>([]);
const doseLogs = ref<DoseLog[]>([]);
const bodyMetrics = ref<BodyMetric[]>([]);
const sideEffects = ref<SideEffect[]>([]);
const inventory = ref<InventoryItem[]>([]);

// Computed
const hasSelection = computed(() => {
  return includeProtocols.value || includeDoseLogs.value || includeBodyMetrics.value ||
    includeSideEffects.value || includeInventory.value;
});

const dateRangeLabel = computed(() => {
  if (dateRange.value === 'all') return 'All Time';
  return `Last ${dateRange.value} Days`;
});

const filteredDoseLogs = computed(() => {
  if (dateRange.value === 'all') return doseLogs.value;

  const days = parseInt(dateRange.value);
  const cutoff = new Date();
  cutoff.setDate(cutoff.getDate() - days);

  return doseLogs.value.filter(log => {
    const logDate = new Date(log.timestamp);
    return logDate >= cutoff;
  });
});

async function generateReport() {
  isGenerating.value = true;

  try {
    const promises: Promise<any>[] = [];

    if (includeProtocols.value) {
      promises.push(listProtocols().then(data => { protocols.value = data; }));
    }
    if (includeDoseLogs.value) {
      promises.push(listDoseLogs().then(data => { doseLogs.value = data; }));
    }
    if (includeBodyMetrics.value) {
      promises.push(listBodyMetrics().then(data => { bodyMetrics.value = data; }));
    }
    if (includeSideEffects.value) {
      promises.push(listSideEffects().then(data => { sideEffects.value = data; }));
    }
    if (includeInventory.value) {
      promises.push(listInventory().then(data => { inventory.value = data; }));
    }

    await Promise.all(promises);

    reportGenerated.value = true;
    showSuccessToast('Success', 'Report generated successfully!');

    // Scroll to report
    setTimeout(() => {
      document.getElementById('printable-report')?.scrollIntoView({ behavior: 'smooth' });
    }, 100);
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'generate report' });
  } finally {
    isGenerating.value = false;
  }
}

function printReport() {
  window.print();
}

function clearReport() {
  reportGenerated.value = false;
  protocols.value = [];
  doseLogs.value = [];
  bodyMetrics.value = [];
  sideEffects.value = [];
  inventory.value = [];
}

function getProtocolName(protocolId: string): string {
  const protocol = protocols.value.find(p => p.id === protocolId);
  return protocol ? `${protocol.name} (${protocol.peptide_name})` : 'Unknown';
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return dateStr;
  }
}
</script>

<style scoped>
.report-generator {
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

.config-section h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #2c3e50;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-weight: 500;
  color: #2c3e50;
  font-size: 14px;
}

.form-group input[type="text"],
.form-group select {
  padding: 10px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.form-group input[type="text"]:focus,
.form-group select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.checkbox-group {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 8px;
}

.checkbox-group label {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  font-weight: normal;
  cursor: pointer;
}

.checkbox-group input[type="checkbox"] {
  width: auto;
  cursor: pointer;
}

.generate-btn {
  padding: 12px 24px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: bold;
  transition: background-color 0.2s;
}

.generate-btn:hover:not(:disabled) {
  background-color: #2980b9;
}

.generate-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.report-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 20px;
}

.print-btn,
.clear-btn {
  padding: 10px 20px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
}

.print-btn {
  background-color: #27ae60;
  color: white;
}

.print-btn:hover {
  background-color: #229954;
}

.clear-btn {
  background-color: #95a5a6;
  color: white;
}

.clear-btn:hover {
  background-color: #7f8c8d;
}

.report-content {
  background: white;
  padding: 40px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.report-header {
  text-align: center;
  margin-bottom: 40px;
  padding-bottom: 20px;
  border-bottom: 2px solid #3498db;
}

.report-header h1 {
  margin: 0 0 10px 0;
  color: #2c3e50;
  font-size: 28px;
}

.report-date {
  color: #7f8c8d;
  font-size: 14px;
  margin: 0;
}

.report-section {
  margin-bottom: 40px;
  page-break-inside: avoid;
}

.report-section h2 {
  color: #2c3e50;
  font-size: 20px;
  margin-bottom: 15px;
  padding-bottom: 8px;
  border-bottom: 1px solid #ddd;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  margin-top: 10px;
}

.data-table th,
.data-table td {
  padding: 10px 12px;
  text-align: left;
  border: 1px solid #ddd;
}

.data-table thead th {
  background-color: #f5f5f5;
  font-weight: 600;
  color: #2c3e50;
}

.data-table tbody tr:nth-child(even) {
  background-color: #fafafa;
}

.data-table tbody tr:hover {
  background-color: #f0f8ff;
}

@media print {
  .report-generator > *:not(#printable-report) {
    display: none;
  }

  .report-content {
    border: none;
    padding: 0;
  }

  .report-section {
    page-break-inside: avoid;
  }

  .data-table {
    page-break-inside: auto;
  }

  .data-table tr {
    page-break-inside: avoid;
    page-break-after: auto;
  }

  .data-table thead {
    display: table-header-group;
  }

  @page {
    margin: 1.5cm;
  }
}

@media (max-width: 768px) {
  .report-content {
    padding: 20px;
  }

  .data-table {
    font-size: 12px;
  }

  .data-table th,
  .data-table td {
    padding: 6px 8px;
  }
}
</style>
