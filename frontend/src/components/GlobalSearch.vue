<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="isOpen" class="search-overlay" @click="close">
        <Transition name="slide-up">
          <div v-if="isOpen" class="search-modal" @click.stop>
            <div class="search-header">
              <span class="search-icon">🔍</span>
              <input
                ref="searchInput"
                v-model="query"
                type="text"
                placeholder="Search anything... (protocols, doses, suppliers, papers)"
                class="search-input"
                @keydown="handleKeyDown"
              />
              <kbd class="kbd">ESC</kbd>
            </div>

            <!-- Results -->
            <div v-if="query.trim()" class="search-results">
              <div v-if="isSearching" class="loading">
                <span class="spinner">⏳</span> Searching...
              </div>

              <template v-else>
                <!-- Protocols -->
                <div v-if="results.protocols.length > 0" class="result-section">
                  <div class="section-header">
                    <span>🧪 Protocols</span>
                    <span class="count">{{ results.protocols.length }}</span>
                  </div>
                  <button
                    v-for="(protocol, idx) in results.protocols"
                    :key="`protocol-${protocol.id}`"
                    :class="['result-item', { selected: selectedIndex === getAbsoluteIndex('protocols', idx) }]"
                    @click="navigateTo('protocols')"
                    @mousemove="selectedIndex = getAbsoluteIndex('protocols', idx)"
                  >
                    <span class="item-icon">🧪</span>
                    <div class="item-content">
                      <div class="item-title">{{ protocol.name }}</div>
                      <div class="item-subtitle">{{ protocol.peptide_name }}</div>
                    </div>
                    <span class="item-arrow">→</span>
                  </button>
                </div>

                <!-- Inventory -->
                <div v-if="results.inventory.length > 0" class="result-section">
                  <div class="section-header">
                    <span>📦 Inventory</span>
                    <span class="count">{{ results.inventory.length }}</span>
                  </div>
                  <button
                    v-for="(item, idx) in results.inventory"
                    :key="`inventory-${item.id}`"
                    :class="['result-item', { selected: selectedIndex === getAbsoluteIndex('inventory', idx) }]"
                    @click="navigateTo('operations')"
                    @mousemove="selectedIndex = getAbsoluteIndex('inventory', idx)"
                  >
                    <span class="item-icon">📦</span>
                    <div class="item-content">
                      <div class="item-title">Vial {{ item.vial_number || 'N/A' }}</div>
                      <div class="item-subtitle">{{ getProtocolName(item.protocol_id) }}</div>
                    </div>
                    <span class="item-arrow">→</span>
                  </button>
                </div>

                <!-- Suppliers -->
                <div v-if="results.suppliers.length > 0" class="result-section">
                  <div class="section-header">
                    <span>🏢 Suppliers</span>
                    <span class="count">{{ results.suppliers.length }}</span>
                  </div>
                  <button
                    v-for="(supplier, idx) in results.suppliers"
                    :key="`supplier-${supplier.id}`"
                    :class="['result-item', { selected: selectedIndex === getAbsoluteIndex('suppliers', idx) }]"
                    @click="navigateTo('operations')"
                    @mousemove="selectedIndex = getAbsoluteIndex('suppliers', idx)"
                  >
                    <span class="item-icon">🏢</span>
                    <div class="item-content">
                      <div class="item-title">{{ supplier.name }}</div>
                      <div class="item-subtitle">{{ supplier.website || 'No website' }}</div>
                    </div>
                    <span class="item-arrow">→</span>
                  </button>
                </div>

                <!-- Papers -->
                <div v-if="results.papers.length > 0" class="result-section">
                  <div class="section-header">
                    <span>📚 Research Papers</span>
                    <span class="count">{{ results.papers.length }}</span>
                  </div>
                  <button
                    v-for="(paper, idx) in results.papers"
                    :key="`paper-${paper.id}`"
                    :class="['result-item', { selected: selectedIndex === getAbsoluteIndex('papers', idx) }]"
                    @click="navigateTo('research')"
                    @mousemove="selectedIndex = getAbsoluteIndex('papers', idx)"
                  >
                    <span class="item-icon">📄</span>
                    <div class="item-content">
                      <div class="item-title">{{ paper.title }}</div>
                      <div class="item-subtitle">{{ paper.source }}</div>
                    </div>
                    <span class="item-arrow">→</span>
                  </button>
                </div>

                <!-- No Results -->
                <div v-if="totalResults === 0" class="no-results">
                  <span class="no-results-icon">🔍</span>
                  <p>No results found for "{{ query }}"</p>
                  <p class="hint">Try searching for protocols, peptides, suppliers, or research papers</p>
                </div>
              </template>
            </div>

            <!-- Quick Actions (shown when empty) -->
            <div v-else class="quick-actions">
              <div class="section-header">
                <span>⚡ Quick Actions</span>
              </div>
              <button
                v-for="(action, idx) in quickActions"
                :key="action.key"
                :class="['result-item', { selected: selectedIndex === idx }]"
                @click="executeAction(action)"
                @mousemove="selectedIndex = idx"
              >
                <span class="item-icon">{{ action.icon }}</span>
                <div class="item-content">
                  <div class="item-title">{{ action.title }}</div>
                  <div class="item-subtitle">{{ action.subtitle }}</div>
                </div>
                <kbd v-if="action.shortcut" class="kbd">{{ action.shortcut }}</kbd>
              </button>
            </div>

            <!-- Footer -->
            <div class="search-footer">
              <div class="footer-hint">
                <kbd class="kbd">↑↓</kbd> Navigate
                <kbd class="kbd">↵</kbd> Select
                <kbd class="kbd">ESC</kbd> Close
              </div>
              <div class="footer-shortcut">
                <kbd class="kbd">Cmd+K</kbd> to open
              </div>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue';
import {
  listProtocols,
  listInventory,
  listSuppliers,
  listLiterature,
  type PeptideProtocol,
  type InventoryItem,
  type Supplier,
  type LiteratureEntry,
} from '../api/peptrack';

const emit = defineEmits<{
  navigate: [view: string];
  close: [];
}>();

interface QuickAction {
  key: string;
  icon: string;
  title: string;
  subtitle: string;
  shortcut?: string;
  action: () => void;
}

const isOpen = ref(false);
const query = ref('');
const searchInput = ref<HTMLInputElement | null>(null);
const isSearching = ref(false);
const selectedIndex = ref(0);

const results = ref<{
  protocols: PeptideProtocol[];
  inventory: InventoryItem[];
  suppliers: Supplier[];
  papers: LiteratureEntry[];
}>({
  protocols: [],
  inventory: [],
  suppliers: [],
  papers: [],
});

const allProtocols = ref<PeptideProtocol[]>([]);
const protocolsMap = ref<Map<string, PeptideProtocol>>(new Map());

const quickActions: QuickAction[] = [
  {
    key: 'log-dose',
    icon: '💉',
    title: 'Log Dose',
    subtitle: 'Record a new dose',
    action: () => navigateTo('doses'),
  },
  {
    key: 'add-protocol',
    icon: '🧪',
    title: 'Add Protocol',
    subtitle: 'Create a new peptide protocol',
    action: () => navigateTo('protocols'),
  },
  {
    key: 'ai-assistant',
    icon: '🤖',
    title: 'AI Protocol Assistant',
    subtitle: 'Get dosing recommendations',
    action: () => navigateTo('ai-assistant'),
  },
  {
    key: 'search-papers',
    icon: '📚',
    title: 'Search Research',
    subtitle: 'Find scientific papers',
    action: () => navigateTo('research'),
  },
  {
    key: 'manage-inventory',
    icon: '📦',
    title: 'Manage Inventory',
    subtitle: 'Add or view inventory',
    action: () => navigateTo('operations'),
  },
  {
    key: 'settings',
    icon: '⚙️',
    title: 'Settings',
    subtitle: 'Configure PepTrack',
    action: () => navigateTo('settings'),
  },
];

const totalResults = computed(() => {
  return (
    results.value.protocols.length +
    results.value.inventory.length +
    results.value.suppliers.length +
    results.value.papers.length
  );
});

const maxResults = computed(() => {
  if (query.value.trim()) {
    return totalResults.value;
  }
  return quickActions.length;
});

function open() {
  isOpen.value = true;
  nextTick(() => {
    searchInput.value?.focus();
  });
}

function close() {
  isOpen.value = false;
  query.value = '';
  selectedIndex.value = 0;
  results.value = {
    protocols: [],
    inventory: [],
    suppliers: [],
    papers: [],
  };
  emit('close');
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault();
    close();
  } else if (e.key === 'ArrowDown') {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, maxResults.value - 1);
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
  } else if (e.key === 'Enter') {
    e.preventDefault();
    handleEnter();
  }
}

function handleEnter() {
  if (query.value.trim()) {
    // Navigate to first result section
    if (results.value.protocols.length > 0) {
      navigateTo('protocols');
    } else if (results.value.inventory.length > 0) {
      navigateTo('operations');
    } else if (results.value.suppliers.length > 0) {
      navigateTo('operations');
    } else if (results.value.papers.length > 0) {
      navigateTo('research');
    }
  } else {
    // Execute selected quick action
    const action = quickActions[selectedIndex.value];
    if (action) {
      executeAction(action);
    }
  }
}

function getAbsoluteIndex(section: string, relativeIndex: number): number {
  let offset = 0;

  if (section === 'protocols') return relativeIndex;

  offset += results.value.protocols.length;
  if (section === 'inventory') return offset + relativeIndex;

  offset += results.value.inventory.length;
  if (section === 'suppliers') return offset + relativeIndex;

  offset += results.value.suppliers.length;
  if (section === 'papers') return offset + relativeIndex;

  return 0;
}

function navigateTo(view: string) {
  emit('navigate', view);
  close();
}

function executeAction(action: QuickAction) {
  action.action();
  close();
}

function getProtocolName(protocolId: string): string {
  return protocolsMap.value.get(protocolId)?.name || 'Unknown Protocol';
}

// Search functionality
watch(query, async (newQuery) => {
  if (!newQuery.trim()) {
    results.value = {
      protocols: [],
      inventory: [],
      suppliers: [],
      papers: [],
    };
    return;
  }

  isSearching.value = true;
  selectedIndex.value = 0;

  try {
    const searchTerm = newQuery.toLowerCase();

    // Search protocols
    const protocols = await listProtocols();
    results.value.protocols = protocols.filter(
      (p) =>
        p.name.toLowerCase().includes(searchTerm) ||
        p.peptide_name.toLowerCase().includes(searchTerm) ||
        (p.notes && p.notes.toLowerCase().includes(searchTerm))
    );

    // Search inventory
    const inventory = await listInventory();
    results.value.inventory = inventory.filter(
      (i) =>
        (i.vial_number && i.vial_number.toLowerCase().includes(searchTerm)) ||
        (i.batch_number && i.batch_number.toLowerCase().includes(searchTerm)) ||
        (i.notes && i.notes.toLowerCase().includes(searchTerm))
    );

    // Search suppliers
    const suppliers = await listSuppliers();
    results.value.suppliers = suppliers.filter(
      (s) =>
        s.name.toLowerCase().includes(searchTerm) ||
        (s.website && s.website.toLowerCase().includes(searchTerm)) ||
        (s.notes && s.notes.toLowerCase().includes(searchTerm))
    );

    // Search papers
    const papers = await listLiterature();
    results.value.papers = papers.filter(
      (p) =>
        p.title.toLowerCase().includes(searchTerm) ||
        (p.summary && p.summary.toLowerCase().includes(searchTerm))
    );
  } catch (error) {
    console.error('Search failed:', error);
  } finally {
    isSearching.value = false;
  }
});

// Load protocols for display
onMounted(async () => {
  try {
    allProtocols.value = await listProtocols();
    protocolsMap.value = new Map(allProtocols.value.map((p) => [p.id, p]));
  } catch (err) {
    // Silently fail
  }
});

// Keyboard shortcut handler
function handleGlobalKeyDown(e: KeyboardEvent) {
  if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
    e.preventDefault();
    if (isOpen.value) {
      close();
    } else {
      open();
    }
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleGlobalKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleGlobalKeyDown);
});

// Expose open method to parent
defineExpose({
  open,
  close,
});
</script>

<style scoped>
.search-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  z-index: 9999;
  padding-top: 10vh;
}

.search-modal {
  background: white;
  border-radius: 12px;
  width: 90%;
  max-width: 640px;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 16px 70px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.search-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid #e0e0e0;
}

.search-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  font-size: 16px;
  outline: none;
  background: transparent;
}

.search-input::placeholder {
  color: #999;
}

.kbd {
  padding: 4px 8px;
  background-color: #f0f0f0;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 11px;
  font-family: monospace;
  color: #666;
  font-weight: 600;
}

.search-results,
.quick-actions {
  overflow-y: auto;
  max-height: 50vh;
  padding: 8px 0;
}

.result-section {
  margin-bottom: 8px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 20px;
  font-size: 12px;
  font-weight: 700;
  color: #666;
  text-transform: uppercase;
}

.count {
  background-color: #e0e0e0;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  width: 100%;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: background-color 0.1s;
  text-align: left;
}

.result-item:hover,
.result-item.selected {
  background-color: #f5f5f5;
}

.result-item.selected {
  background-color: #e8f0fe;
}

.item-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-title {
  font-weight: 600;
  font-size: 14px;
  color: #2c3e50;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-subtitle {
  font-size: 12px;
  color: #666;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 2px;
}

.item-arrow {
  color: #999;
  font-size: 16px;
}

.loading {
  padding: 40px;
  text-align: center;
  color: #666;
  font-size: 14px;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.no-results {
  padding: 60px 40px;
  text-align: center;
}

.no-results-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
}

.no-results p {
  margin: 8px 0;
  color: #666;
}

.no-results .hint {
  font-size: 13px;
  color: #999;
}

.search-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 20px;
  border-top: 1px solid #e0e0e0;
  background-color: #f8f9fa;
  font-size: 12px;
  color: #666;
}

.footer-hint,
.footer-shortcut {
  display: flex;
  gap: 8px;
  align-items: center;
}

/* Transitions */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.2s ease-out;
}

.slide-up-enter-from {
  opacity: 0;
  transform: translateY(-20px) scale(0.95);
}

.slide-up-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.98);
}

@media (max-width: 768px) {
  .search-modal {
    width: 95%;
    max-height: 80vh;
    margin-top: 5vh;
  }

  .search-overlay {
    padding-top: 5vh;
  }

  .footer-shortcut {
    display: none;
  }
}
</style>
