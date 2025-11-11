<template>
  <div class="supplier-management">
    <h2>🏢 Supplier Management</h2>
    <p class="subtitle">Track your peptide suppliers and vendors</p>

    <!-- Add/Edit Supplier Form -->
    <div class="supplier-form-section panel">
      <h3>{{ editingSupplier ? '✏️ Edit Supplier' : '➕ Add Supplier' }}</h3>
      <form @submit.prevent="handleSaveSupplier" class="supplier-form">
        <label for="supplier-name">
          Supplier Name *
        </label>
        <input
          id="supplier-name"
          v-model="form.name"
          type="text"
          placeholder="e.g., Peptide Sciences"
          required
          aria-label="Supplier name"
          autocomplete="off"
        />

        <div class="form-row">
          <div class="form-group">
            <label for="supplier-email">
              Email
            </label>
            <input
              id="supplier-email"
              v-model="form.contactEmail"
              type="email"
              placeholder="contact@supplier.com"
              aria-label="Supplier email"
              autocomplete="off"
            />
          </div>

          <div class="form-group">
            <label for="supplier-phone">
              Phone
            </label>
            <input
              id="supplier-phone"
              v-model="form.contactPhone"
              type="tel"
              placeholder="+1-555-0123"
              aria-label="Supplier phone"
              autocomplete="off"
            />
          </div>
        </div>

        <label for="supplier-website">
          Website
        </label>
        <input
          id="supplier-website"
          v-model="form.website"
          type="url"
          placeholder="https://supplier.com"
          aria-label="Supplier website"
          autocomplete="off"
        />

        <label for="supplier-notes">
          Notes (optional)
        </label>
        <textarea
          id="supplier-notes"
          v-model="form.notes"
          rows="3"
          placeholder="Quality, shipping info, prices, etc."
          aria-label="Supplier notes"
        />

        <div class="form-actions">
          <button
            type="submit"
            :disabled="isSaving"
            class="primary-btn"
            :aria-label="editingSupplier ? 'Update supplier' : 'Create supplier'"
            :aria-busy="isSaving"
          >
            {{ isSaving ? '⏳ Saving...' : editingSupplier ? '💾 Update Supplier' : '💾 Add Supplier' }}
          </button>
          <button
            v-if="editingSupplier"
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

    <!-- Supplier List -->
    <div class="suppliers-section">
      <div class="suppliers-header">
        <h3>📋 Your Suppliers</h3>
        <button
          @click="loadSuppliers"
          class="refresh-btn"
          aria-label="Refresh supplier list"
        >↻ Refresh</button>
      </div>

      <div v-if="isLoading" class="loading">
        ⏳ Loading suppliers...
      </div>

      <div v-else-if="suppliers.length === 0" class="no-suppliers">
        No suppliers yet. Add your first supplier above!
      </div>

      <div v-else class="supplier-list">
        <div v-for="supplier in suppliers" :key="supplier.id" class="supplier-card">
          <div class="supplier-header">
            <div class="supplier-info">
              <strong>{{ supplier.name }}</strong>
            </div>
            <div class="supplier-actions">
              <button
                @click="startEdit(supplier)"
                class="edit-btn"
                :aria-label="`Edit ${supplier.name}`"
              >
                ✏️ Edit
              </button>
              <button
                @click="handleDelete(supplier.id)"
                class="delete-btn"
                :aria-label="`Delete ${supplier.name}`"
              >
                🗑️
              </button>
            </div>
          </div>

          <div v-if="supplier.contact_email || supplier.contact_phone" class="supplier-contact">
            <div v-if="supplier.contact_email" class="contact-item">
              📧 <a :href="`mailto:${supplier.contact_email}`">{{ supplier.contact_email }}</a>
            </div>
            <div v-if="supplier.contact_phone" class="contact-item">
              📞 {{ supplier.contact_phone }}
            </div>
          </div>

          <div v-if="supplier.website" class="supplier-website">
            🌐 <a :href="supplier.website" target="_blank" rel="noopener noreferrer">{{ supplier.website }}</a>
          </div>

          <p v-if="supplier.notes" class="supplier-notes">
            📝 {{ supplier.notes }}
          </p>

          <div class="supplier-meta">
            Updated: {{ formatDate(supplier.updated_at) }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import type { Supplier, CreateSupplierPayload, UpdateSupplierPayload } from '../api/peptrack';
import { listSuppliers, createSupplier, updateSupplier, deleteSupplier } from '../api/peptrack';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';

const suppliers = ref<Supplier[]>([]);
const isLoading = ref(false);
const isSaving = ref(false);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const editingSupplier = ref<Supplier | null>(null);

const form = ref({
  name: '',
  contactEmail: '',
  contactPhone: '',
  website: '',
  notes: '',
});

function resetForm() {
  form.value = {
    name: '',
    contactEmail: '',
    contactPhone: '',
    website: '',
    notes: '',
  };
  editingSupplier.value = null;
}

function startEdit(supplier: Supplier) {
  editingSupplier.value = supplier;
  form.value = {
    name: supplier.name,
    contactEmail: supplier.contact_email || '',
    contactPhone: supplier.contact_phone || '',
    website: supplier.website || '',
    notes: supplier.notes || '',
  };
  // Scroll to form
  window.scrollTo({ top: 0, behavior: 'smooth' });
}

function cancelEdit() {
  resetForm();
}

async function loadSuppliers() {
  isLoading.value = true;
  error.value = null;
  try {
    suppliers.value = await listSuppliers();
  } catch (err) {
    const errorMsg = `Failed to load suppliers: ${String(err)}`;
    error.value = errorMsg;
    showErrorToast(new Error(errorMsg));
  } finally {
    isLoading.value = false;
  }
}

async function handleSaveSupplier() {
  if (!form.value.name.trim()) {
    error.value = 'Supplier name is required';
    return;
  }

  isSaving.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    if (editingSupplier.value) {
      // Update existing
      const payload: UpdateSupplierPayload = {
        name: form.value.name,
        contactEmail: form.value.contactEmail || undefined,
        contactPhone: form.value.contactPhone || undefined,
        website: form.value.website || undefined,
        notes: form.value.notes || undefined,
      };
      await updateSupplier(editingSupplier.value.id, payload);
      successMessage.value = 'Supplier updated successfully!';
      showSuccessToast('Success', 'Supplier updated successfully!');
    } else {
      // Create new
      const payload: CreateSupplierPayload = {
        name: form.value.name,
        contactEmail: form.value.contactEmail || undefined,
        contactPhone: form.value.contactPhone || undefined,
        website: form.value.website || undefined,
        notes: form.value.notes || undefined,
      };
      await createSupplier(payload);
      successMessage.value = 'Supplier added successfully!';
      showSuccessToast('Success', 'Supplier added successfully!');
    }

    resetForm();
    await loadSuppliers();

    // Clear success message after 3 seconds
    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
  } catch (err) {
    const errorMsg = `Failed to save supplier: ${String(err)}`;
    error.value = errorMsg;
    showErrorToast(new Error(errorMsg));
  } finally {
    isSaving.value = false;
  }
}

async function handleDelete(supplierId: string) {
  if (!confirm('Are you sure you want to delete this supplier? This will not delete inventory items linked to this supplier.')) {
    return;
  }

  error.value = null;
  successMessage.value = null;

  try {
    await deleteSupplier(supplierId);
    successMessage.value = 'Supplier deleted successfully!';
    showSuccessToast('Success', 'Supplier deleted successfully!');
    await loadSuppliers();

    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
  } catch (err) {
    const errorMsg = `Failed to delete supplier: ${String(err)}`;
    error.value = errorMsg;
    showErrorToast(new Error(errorMsg));
  }
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

onMounted(() => {
  loadSuppliers();
});
</script>

<style scoped>
.supplier-management {
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

.supplier-form-section {
  margin-bottom: 24px;
}

.supplier-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
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

input,
textarea {
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
  transition: border-color 0.2s;
}

input:focus,
textarea:focus {
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

.suppliers-section {
  margin-top: 32px;
}

.suppliers-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.suppliers-header h3 {
  margin: 0;
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
}

.refresh-btn:hover {
  background-color: #2980b9;
  transform: translateY(-1px);
}

.loading,
.no-suppliers {
  padding: 24px;
  text-align: center;
  color: #666;
  background-color: #f8f9fa;
  border-radius: 6px;
}

.supplier-list {
  display: grid;
  gap: 16px;
}

.supplier-card {
  padding: 16px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background-color: white;
  transition: box-shadow 0.2s;
}

.supplier-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.supplier-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.supplier-info strong {
  font-size: 16px;
  color: #2c3e50;
}

.supplier-actions {
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

.supplier-contact {
  margin-bottom: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.contact-item {
  font-size: 14px;
  color: #555;
}

.contact-item a {
  color: #3498db;
  text-decoration: none;
}

.contact-item a:hover {
  text-decoration: underline;
}

.supplier-website {
  margin-bottom: 8px;
  font-size: 14px;
  color: #555;
}

.supplier-website a {
  color: #3498db;
  text-decoration: none;
}

.supplier-website a:hover {
  text-decoration: underline;
}

.supplier-notes {
  margin: 8px 0;
  padding: 8px;
  background-color: #f8f9fa;
  border-radius: 4px;
  font-size: 13px;
  color: #555;
  line-height: 1.5;
}

.supplier-meta {
  margin-top: 8px;
  font-size: 12px;
  color: #999;
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .supplier-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
  }

  .supplier-actions {
    width: 100%;
    justify-content: flex-end;
  }
}
</style>
