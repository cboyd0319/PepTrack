<template>
  <div class="inventory-management">
    <h2>📦 Inventory Management</h2>
    <p class="subtitle">Track your peptide vials and stock</p>

    <!-- Add/Edit Inventory Form -->
    <div class="inventory-form-section panel">
      <h3>{{ editingItem ? '✏️ Edit Inventory Item' : '➕ Add Inventory Item' }}</h3>
      <form @submit.prevent="handleSaveItem" class="inventory-form">
        <label for="inventory-protocol">
          Protocol * <span class="help-text">(Which peptide is this for?)</span>
        </label>
        <select
          id="inventory-protocol"
          v-model="form.protocolId"
          required
          aria-label="Select protocol"
          :disabled="!!editingItem"
        >
          <option value="">Select a protocol...</option>
          <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
            {{ protocol.name }} ({{ protocol.peptide_name }})
          </option>
        </select>

        <div class="form-row">
          <div class="form-group">
            <label for="inventory-supplier">
              Supplier
            </label>
            <select
              id="inventory-supplier"
              v-model="form.supplierId"
              aria-label="Select supplier"
            >
              <option value="">No supplier</option>
              <option v-for="supplier in suppliers" :key="supplier.id" :value="supplier.id">
                {{ supplier.name }}
              </option>
            </select>
          </div>

          <div class="form-group">
            <label for="inventory-vial-number">
              Vial Number
            </label>
            <input
              id="inventory-vial-number"
              v-model="form.vialNumber"
              type="text"
              placeholder="e.g., V001"
              aria-label="Vial number"
              autocomplete="off"
            />
          </div>
        </div>

        <label for="inventory-status">
          Vial Status *
        </label>
        <select
          id="inventory-status"
          v-model="form.vialStatus"
          required
          aria-label="Select vial status"
        >
          <option value="sealed">🔒 Sealed</option>
          <option value="opened">📂 Opened</option>
          <option value="empty">📭 Empty</option>
          <option value="expired">⚠️ Expired</option>
        </select>

        <div class="form-row">
          <div class="form-group">
            <label for="inventory-purchase-date">
              Purchase Date
            </label>
            <input
              id="inventory-purchase-date"
              v-model="form.purchaseDate"
              type="date"
              aria-label="Purchase date"
            />
          </div>

          <div class="form-group">
            <label for="inventory-expiry-date">
              Expiry Date
            </label>
            <input
              id="inventory-expiry-date"
              v-model="form.expiryDate"
              type="date"
              aria-label="Expiry date"
            />
          </div>
        </div>

        <div class="form-row three-col">
          <div class="form-group">
            <label for="inventory-quantity">
              Quantity (mg)
            </label>
            <input
              id="inventory-quantity"
              v-model.number="form.quantityMg"
              type="number"
              step="0.01"
              min="0"
              placeholder="10.0"
              aria-label="Quantity in milligrams"
              autocomplete="off"
            />
          </div>

          <div class="form-group">
            <label for="inventory-cost">
              Cost per mg ($)
            </label>
            <input
              id="inventory-cost"
              v-model.number="form.costPerMg"
              type="number"
              step="0.01"
              min="0"
              placeholder="1.25"
              aria-label="Cost per milligram"
              autocomplete="off"
            />
          </div>

          <div class="form-group">
            <label for="inventory-concentration">
              Concentration (mg/ml)
            </label>
            <input
              id="inventory-concentration"
              v-model.number="form.concentrationMgMl"
              type="number"
              step="0.01"
              min="0"
              placeholder="5.0"
              aria-label="Concentration"
              autocomplete="off"
            />
          </div>
        </div>

        <div class="form-row">
          <div class="form-group">
            <label for="inventory-batch">
              Batch Number
            </label>
            <input
              id="inventory-batch"
              v-model="form.batchNumber"
              type="text"
              placeholder="e.g., BATCH-2024-001"
              aria-label="Batch number"
              autocomplete="off"
            />
          </div>

          <div class="form-group">
            <label for="inventory-lot">
              Lot Number
            </label>
            <input
              id="inventory-lot"
              v-model="form.lotNumber"
              type="text"
              placeholder="e.g., LOT-2024-001"
              aria-label="Lot number"
              autocomplete="off"
            />
          </div>
        </div>

        <label for="inventory-notes">
          Notes (optional)
        </label>
        <textarea
          id="inventory-notes"
          v-model="form.notes"
          rows="3"
          placeholder="Storage location, observations, etc."
          aria-label="Inventory notes"
        />

        <div class="form-actions">
          <button
            type="submit"
            :disabled="isSaving"
            class="primary-btn"
            :aria-label="editingItem ? 'Update item' : 'Create item'"
            :aria-busy="isSaving"
          >
            {{ isSaving ? '⏳ Saving...' : editingItem ? '💾 Update Item' : '💾 Add Item' }}
          </button>
          <button
            v-if="editingItem"
            type="button"
            @click="cancelEdit"
            class="secondary-btn"
            aria-label="Cancel editing"
          >
            ❌ Cancel
          </button>
        </div>
      </form>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-message">
      ⚠️ {{ error }}
    </div>

    <!-- Success Message -->
    <div v-if="successMessage" class="success-message">
      ✅ {{ successMessage }}
    </div>

    <!-- Inventory List -->
    <div class="inventory-section">
      <div class="inventory-header">
        <h3>📋 Your Inventory</h3>
        <div class="inventory-controls">
          <label for="filter-protocol">
            Filter by protocol:
          </label>
          <select
            id="filter-protocol"
            v-model="filterProtocolId"
            @change="loadInventory"
            aria-label="Filter by protocol"
          >
            <option value="">All Protocols</option>
            <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
              {{ protocol.name }}
            </option>
          </select>
          <button
            @click="loadInventory"
            class="refresh-btn"
            aria-label="Refresh inventory"
          >↻ Refresh</button>
        </div>
      </div>

      <div v-if="isLoading" class="loading">
        ⏳ Loading inventory...
      </div>

      <div v-else-if="inventory.length === 0" class="no-inventory">
        No inventory items yet. Add your first item above!
      </div>

      <div v-else class="inventory-list">
        <div v-for="item in inventory" :key="item.id" class="inventory-card">
          <div class="inventory-header-row">
            <div class="inventory-title">
              <strong>{{ getProtocolName(item.protocol_id) }}</strong>
              <span :class="['status-badge', item.vial_status]">
                {{ getStatusLabel(item.vial_status) }}
              </span>
            </div>
            <div class="inventory-actions">
              <button
                @click="startEdit(item)"
                class="edit-btn"
                :aria-label="`Edit inventory item`"
              >
                ✏️ Edit
              </button>
              <button
                @click="handleDelete(item.id)"
                class="delete-btn"
                :aria-label="`Delete inventory item`"
              >
                🗑️
              </button>
            </div>
          </div>

          <div class="inventory-details">
            <div v-if="item.supplier_id" class="detail-row">
              <span class="detail-label">🏢 Supplier:</span>
              <span>{{ getSupplierName(item.supplier_id) }}</span>
            </div>

            <div v-if="item.vial_number" class="detail-row">
              <span class="detail-label">🔢 Vial:</span>
              <span>{{ item.vial_number }}</span>
            </div>

            <div v-if="item.quantity_mg" class="detail-row">
              <span class="detail-label">⚖️ Quantity:</span>
              <span>{{ item.quantity_mg }} mg</span>
            </div>

            <div v-if="item.concentration_mg_ml" class="detail-row">
              <span class="detail-label">💧 Concentration:</span>
              <span>{{ item.concentration_mg_ml }} mg/ml</span>
            </div>

            <div v-if="item.cost_per_mg" class="detail-row">
              <span class="detail-label">💰 Cost:</span>
              <span>${{ item.cost_per_mg }}/mg</span>
              <span v-if="item.quantity_mg" class="total-cost">
                (Total: ${{ (item.cost_per_mg * item.quantity_mg).toFixed(2) }})
              </span>
            </div>

            <div v-if="item.purchase_date" class="detail-row">
              <span class="detail-label">📅 Purchased:</span>
              <span>{{ formatDate(item.purchase_date) }}</span>
            </div>

            <div v-if="item.expiry_date" class="detail-row">
              <span class="detail-label">⏰ Expires:</span>
              <span :class="{ 'expiry-warning': isExpiringSoon(item.expiry_date) }">
                {{ formatDate(item.expiry_date) }}
                <span v-if="isExpired(item.expiry_date)" class="expired-badge">EXPIRED</span>
                <span v-else-if="isExpiringSoon(item.expiry_date)" class="expiring-badge">EXPIRING SOON</span>
              </span>
            </div>

            <div v-if="item.batch_number || item.lot_number" class="detail-row">
              <span class="detail-label">🏷️ Batch/Lot:</span>
              <span>
                <span v-if="item.batch_number">Batch: {{ item.batch_number }}</span>
                <span v-if="item.batch_number && item.lot_number"> | </span>
                <span v-if="item.lot_number">Lot: {{ item.lot_number }}</span>
              </span>
            </div>
          </div>

          <p v-if="item.notes" class="inventory-notes">
            📝 {{ item.notes }}
          </p>

          <div class="inventory-meta">
            Updated: {{ formatDate(item.updated_at) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type {
  InventoryItem,
  CreateInventoryPayload,
  UpdateInventoryPayload,
  VialStatus,
  PeptideProtocol,
  Supplier
} from '../api/peptrack';
import {
  listInventory,
  listInventoryByProtocol,
  createInventoryItem,
  updateInventoryItem,
  deleteInventoryItem,
  listProtocols,
  listSuppliers
} from '../api/peptrack';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';

const inventory = ref<InventoryItem[]>([]);
const protocols = ref<PeptideProtocol[]>([]);
const suppliers = ref<Supplier[]>([]);
const isLoading = ref(false);
const isSaving = ref(false);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const editingItem = ref<InventoryItem | null>(null);
const filterProtocolId = ref<string>('');

const form = ref({
  protocolId: '',
  supplierId: '',
  vialNumber: '',
  vialStatus: 'sealed' as VialStatus,
  purchaseDate: '',
  expiryDate: '',
  costPerMg: null as number | null,
  quantityMg: null as number | null,
  concentrationMgMl: null as number | null,
  batchNumber: '',
  lotNumber: '',
  notes: '',
});

function resetForm() {
  form.value = {
    protocolId: '',
    supplierId: '',
    vialNumber: '',
    vialStatus: 'sealed',
    purchaseDate: '',
    expiryDate: '',
    costPerMg: null,
    quantityMg: null,
    concentrationMgMl: null,
    batchNumber: '',
    lotNumber: '',
    notes: '',
  };
  editingItem.value = null;
}

function startEdit(item: InventoryItem) {
  editingItem.value = item;
  form.value = {
    protocolId: item.protocol_id,
    supplierId: item.supplier_id || '',
    vialNumber: item.vial_number || '',
    vialStatus: item.vial_status,
    purchaseDate: (item.purchase_date || '').split('T')[0] || '',
    expiryDate: (item.expiry_date || '').split('T')[0] || '',
    costPerMg: item.cost_per_mg !== null && item.cost_per_mg !== undefined ? item.cost_per_mg : null,
    quantityMg: item.quantity_mg !== null && item.quantity_mg !== undefined ? item.quantity_mg : null,
    concentrationMgMl: item.concentration_mg_ml !== null && item.concentration_mg_ml !== undefined ? item.concentration_mg_ml : null,
    batchNumber: item.batch_number || '',
    lotNumber: item.lot_number || '',
    notes: item.notes || '',
  };
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function cancelEdit() {
  resetForm();
}

async function loadProtocols() {
  try {
    protocols.value = await listProtocols();
  } catch (_err) {
    // Failed to load protocols, will show empty list
  }
}

async function loadSuppliers() {
  try {
    suppliers.value = await listSuppliers();
  } catch (_err) {
    // Failed to load suppliers, will show empty list
  }
}

async function loadInventory() {
  isLoading.value = true;
  error.value = null;
  try {
    if (filterProtocolId.value) {
      inventory.value = await listInventoryByProtocol(filterProtocolId.value);
    } else {
      inventory.value = await listInventory();
    }
  } catch (err) {
    const errorMsg = `Failed to load inventory: ${String(err)}`;
    error.value = errorMsg;
    showErrorToast(new Error(errorMsg));
  } finally {
    isLoading.value = false;
  }
}

async function handleSaveItem() {
  if (!form.value.protocolId) {
    error.value = 'Protocol is required';
    return;
  }

  isSaving.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    if (editingItem.value) {
      // Update existing
      const payload: UpdateInventoryPayload = {
        supplierId: form.value.supplierId ? form.value.supplierId : undefined,
        vialNumber: form.value.vialNumber ? form.value.vialNumber : undefined,
        vialStatus: form.value.vialStatus,
        purchaseDate: form.value.purchaseDate ? form.value.purchaseDate : undefined,
        expiryDate: form.value.expiryDate ? form.value.expiryDate : undefined,
        costPerMg: form.value.costPerMg !== null ? form.value.costPerMg : undefined,
        quantityMg: form.value.quantityMg !== null ? form.value.quantityMg : undefined,
        concentrationMgMl: form.value.concentrationMgMl !== null ? form.value.concentrationMgMl : undefined,
        batchNumber: form.value.batchNumber ? form.value.batchNumber : undefined,
        lotNumber: form.value.lotNumber ? form.value.lotNumber : undefined,
        notes: form.value.notes ? form.value.notes : undefined,
      };
      await updateInventoryItem(editingItem.value.id, payload);
      successMessage.value = 'Inventory item updated successfully!';
      showSuccessToast('Success', 'Inventory item updated successfully!');
    } else {
      // Create new
      const payload: CreateInventoryPayload = {
        protocolId: form.value.protocolId,
        supplierId: form.value.supplierId ? form.value.supplierId : undefined,
        vialNumber: form.value.vialNumber ? form.value.vialNumber : undefined,
        vialStatus: form.value.vialStatus,
        purchaseDate: form.value.purchaseDate ? form.value.purchaseDate : undefined,
        expiryDate: form.value.expiryDate ? form.value.expiryDate : undefined,
        costPerMg: form.value.costPerMg !== null ? form.value.costPerMg : undefined,
        quantityMg: form.value.quantityMg !== null ? form.value.quantityMg : undefined,
        concentrationMgMl: form.value.concentrationMgMl !== null ? form.value.concentrationMgMl : undefined,
        batchNumber: form.value.batchNumber ? form.value.batchNumber : undefined,
        lotNumber: form.value.lotNumber ? form.value.lotNumber : undefined,
        notes: form.value.notes ? form.value.notes : undefined,
      };
      await createInventoryItem(payload);
      successMessage.value = 'Inventory item added successfully!';
      showSuccessToast('Success', 'Inventory item added successfully!');
    }

    resetForm();
    await loadInventory();

    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
  } catch (err) {
    const errorMsg = `Failed to save inventory item: ${String(err)}`;
    error.value = errorMsg;
    showErrorToast(new Error(errorMsg));
  } finally {
    isSaving.value = false;
  }
}

async function handleDelete(itemId: string) {
  if (!confirm('Are you sure you want to delete this inventory item?')) {
    return;
  }

  error.value = null;
  successMessage.value = null;

  try {
    await deleteInventoryItem(itemId);
    successMessage.value = 'Inventory item deleted successfully!';
    showSuccessToast('Success', 'Inventory item deleted successfully!');
    await loadInventory();

    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
  } catch (err) {
    const errorMsg = `Failed to delete inventory item: ${String(err)}`;
    error.value = errorMsg;
    showErrorToast(new Error(errorMsg));
  }
}

function getProtocolName(protocolId: string): string {
  const protocol = protocols.value.find(p => p.id === protocolId);
  return protocol ? `${protocol.name} (${protocol.peptide_name})` : 'Unknown Protocol';
}

function getSupplierName(supplierId: string): string {
  const supplier = suppliers.value.find(s => s.id === supplierId);
  return supplier ? supplier.name : 'Unknown Supplier';
}

function getStatusLabel(status: VialStatus): string {
  const labels = {
    sealed: '🔒 Sealed',
    opened: '📂 Opened',
    empty: '📭 Empty',
    expired: '⚠️ Expired',
  };
  return labels[status] || status;
}

function formatDate(dateString: string): string {
  try {
    const date = new Date(dateString);
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  } catch {
    return dateString;
  }
}

function isExpired(expiryDate: string): boolean {
  try {
    const expiry = new Date(expiryDate);
    const now = new Date();
    return expiry < now;
  } catch {
    return false;
  }
}

function isExpiringSoon(expiryDate: string): boolean {
  try {
    const expiry = new Date(expiryDate);
    const now = new Date();
    const thirtyDaysFromNow = new Date(now.getTime() + 30 * 24 * 60 * 60 * 1000);
    return expiry > now && expiry <= thirtyDaysFromNow;
  } catch {
    return false;
  }
}

onMounted(async () => {
  await Promise.all([
    loadProtocols(),
    loadSuppliers(),
    loadInventory(),
  ]);
});
</script>

<style scoped>
.inventory-management {
  max-width: 1200px;
  margin: 0 auto;
}

h2 {
  margin-bottom: 8px;
}

.subtitle {
  margin-top: 0;
  color: #666;
  font-size: 14px;
}

.inventory-form-section {
  margin-bottom: 24px;
}

.inventory-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.form-row.three-col {
  grid-template-columns: 1fr 1fr 1fr;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

label {
  font-weight: 600;
  font-size: 14px;
  color: #333;
}

.help-text {
  font-weight: 400;
  color: #666;
  font-size: 12px;
}

input,
textarea,
select {
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
  transition: border-color 0.2s;
}

input:focus,
textarea:focus,
select:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.form-actions {
  display: flex;
  gap: 12px;
}

.primary-btn {
  flex: 1;
  padding: 12px;
  background-color: #27ae60;
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.primary-btn:hover:not(:disabled) {
  background-color: #229954;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(39, 174, 96, 0.3);
}

.primary-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.secondary-btn {
  padding: 12px 20px;
  background-color: #95a5a6;
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.secondary-btn:hover {
  background-color: #7f8c8d;
  transform: translateY(-1px);
}

.error-message {
  padding: 12px;
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 6px;
  color: #c33;
  margin-bottom: 16px;
}

.success-message {
  padding: 12px;
  background-color: #d4edda;
  border: 1px solid #c3e6cb;
  border-radius: 6px;
  color: #155724;
  margin-bottom: 16px;
}

.inventory-section {
  margin-top: 32px;
}

.inventory-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  flex-wrap: wrap;
  gap: 16px;
}

.inventory-header h3 {
  margin: 0;
}

.inventory-controls {
  display: flex;
  gap: 12px;
  align-items: center;
}

.inventory-controls label {
  font-weight: 500;
  font-size: 14px;
}

.inventory-controls select {
  padding: 6px 10px;
  font-size: 13px;
}

.refresh-btn {
  padding: 8px 16px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.refresh-btn:hover {
  background-color: #2980b9;
  transform: translateY(-1px);
}

.loading,
.no-inventory {
  padding: 24px;
  text-align: center;
  color: #666;
  background-color: #f8f9fa;
  border-radius: 6px;
}

.inventory-list {
  display: grid;
  gap: 16px;
}

.inventory-card {
  padding: 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: white;
  transition: box-shadow 0.2s;
}

.inventory-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.inventory-header-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
  gap: 12px;
}

.inventory-title {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.inventory-title strong {
  font-size: 16px;
  color: #2c3e50;
}

.status-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.status-badge.sealed {
  background-color: #d4edda;
  color: #155724;
}

.status-badge.opened {
  background-color: #cce5ff;
  color: #004085;
}

.status-badge.empty {
  background-color: #f8d7da;
  color: #721c24;
}

.status-badge.expired {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

.inventory-actions {
  display: flex;
  gap: 8px;
}

.edit-btn,
.delete-btn {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.edit-btn {
  background-color: #3498db;
  color: white;
}

.edit-btn:hover {
  background-color: #2980b9;
}

.delete-btn {
  background-color: #e74c3c;
  color: white;
}

.delete-btn:hover {
  background-color: #c0392b;
}

.inventory-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 8px;
}

.detail-row {
  display: flex;
  gap: 8px;
  font-size: 14px;
  color: #555;
  align-items: center;
}

.detail-label {
  font-weight: 600;
  min-width: 120px;
}

.total-cost {
  color: #666;
  font-size: 12px;
  margin-left: 8px;
}

.expiry-warning {
  color: #e67e22;
  font-weight: 600;
}

.expired-badge,
.expiring-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  margin-left: 8px;
}

.expired-badge {
  background-color: #e74c3c;
  color: white;
}

.expiring-badge {
  background-color: #f39c12;
  color: white;
}

.inventory-notes {
  margin: 8px 0;
  padding: 8px;
  background-color: #f8f9fa;
  border-radius: 4px;
  font-size: 13px;
  color: #555;
  line-height: 1.5;
}

.inventory-meta {
  margin-top: 8px;
  font-size: 12px;
  color: #999;
}

@media (max-width: 768px) {
  .form-row,
  .form-row.three-col {
    grid-template-columns: 1fr;
  }

  .inventory-header {
    flex-direction: column;
    align-items: stretch;
  }

  .inventory-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .inventory-header-row {
    flex-direction: column;
  }

  .inventory-actions {
    width: 100%;
    justify-content: flex-end;
  }
}
</style>
