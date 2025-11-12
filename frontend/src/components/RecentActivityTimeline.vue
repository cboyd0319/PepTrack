<template>
  <div class="activity-timeline">
    <div class="timeline-header">
      <h3>🕐 Recent Activity</h3>
      <div class="timeline-filters">
        <select v-model="selectedFilter" @change="loadActivity" class="filter-select">
          <option value="all">All Activity</option>
          <option value="doses">Doses Only</option>
          <option value="inventory">Inventory Only</option>
          <option value="protocols">Protocols Only</option>
        </select>
        <button @click="loadActivity" class="refresh-btn" title="Refresh">
          ↻
        </button>
      </div>
    </div>

    <div v-if="loading" class="loading">Loading activity...</div>

    <div v-else-if="activities.length === 0" class="empty-state">
      <span class="empty-icon">📋</span>
      <p>No recent activity</p>
      <p class="hint">Your actions will appear here</p>
    </div>

    <div v-else class="timeline-container">
      <!-- Group by date -->
      <div v-for="group in groupedActivities" :key="group.date" class="timeline-group">
        <div class="date-header">
          <span class="date-label">{{ group.label }}</span>
          <span class="date-count">{{ group.activities.length }} {{ group.activities.length === 1 ? 'activity' : 'activities' }}</span>
        </div>

        <div class="timeline-items">
          <div
            v-for="activity in group.activities"
            :key="activity.id"
            class="timeline-item"
            :class="`type-${activity.type}`"
          >
            <div class="item-icon" :class="`icon-${activity.type}`">
              {{ getActivityIcon(activity.type) }}
            </div>

            <div class="item-line"></div>

            <div class="item-content">
              <div class="item-header">
                <div class="item-title">{{ activity.title }}</div>
                <div class="item-time">{{ formatTime(activity.timestamp) }}</div>
              </div>

              <div class="item-description">{{ activity.description }}</div>

              <div v-if="activity.metadata" class="item-metadata">
                <span
                  v-for="(value, key) in activity.metadata"
                  :key="key"
                  class="metadata-tag"
                >
                  {{ key }}: <strong>{{ value }}</strong>
                </span>
              </div>

              <div v-if="activity.action" class="item-actions">
                <button @click="handleAction(activity.action)" class="action-btn">
                  {{ activity.action.label }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Load more -->
      <div v-if="hasMore" class="load-more">
        <button @click="loadMore" class="load-more-btn" :disabled="loadingMore">
          {{ loadingMore ? 'Loading...' : 'Load More' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import {
  listDoseLogs,
  listInventory,
  listProtocols,
} from '../api/peptrack';

type ActivityType = 'dose' | 'inventory' | 'protocol' | 'alert' | 'backup';

interface Activity {
  id: string;
  type: ActivityType;
  title: string;
  description: string;
  timestamp: string;
  metadata?: Record<string, string | number>;
  action?: {
    label: string;
    handler: () => void;
  };
}

interface ActivityGroup {
  date: string;
  label: string;
  activities: Activity[];
}

const loading = ref(false);
const loadingMore = ref(false);
const hasMore = ref(false);
const selectedFilter = ref('all');
const activities = ref<Activity[]>([]);
const page = ref(1);
const pageSize = 20;

const groupedActivities = computed<ActivityGroup[]>(() => {
  const groups = new Map<string, Activity[]>();
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);

  activities.value.forEach(activity => {
    const date = new Date(activity.timestamp);
    date.setHours(0, 0, 0, 0);

    let dateKey: string;

    if (date.getTime() === today.getTime()) {
      dateKey = 'today';
    } else if (date.getTime() === yesterday.getTime()) {
      dateKey = 'yesterday';
    } else {
      dateKey = date.toISOString().split('T')[0]!;
    }

    if (!groups.has(dateKey)) {
      groups.set(dateKey, []);
    }
    groups.get(dateKey)!.push(activity);
  });

  return Array.from(groups.entries())
    .map(([date, activities]) => ({
      date,
      label: activities[0] ? (date === 'today' ? 'Today' : date === 'yesterday' ? 'Yesterday' : new Date(activities[0].timestamp).toLocaleDateString('en-US', { month: 'long', day: 'numeric' })) : date,
      activities: activities.sort((a, b) =>
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
      ),
    }))
    .sort((a, b) => {
      if (a.date === 'today') return -1;
      if (b.date === 'today') return 1;
      if (a.date === 'yesterday') return -1;
      if (b.date === 'yesterday') return 1;
      return new Date(b.date).getTime() - new Date(a.date).getTime();
    });
});

onMounted(async () => {
  await loadActivity();
});

async function loadActivity() {
  loading.value = true;
  page.value = 1;
  try {
    const [doses, inventory, protocols] = await Promise.all([
      listDoseLogs(),
      listInventory(),
      listProtocols(),
    ]);

    const activityList: Activity[] = [];

    // Convert doses to activities
    if (selectedFilter.value === 'all' || selectedFilter.value === 'doses') {
      doses.forEach(dose => {
        const protocol = protocols.find(p => p.id === dose.protocol_id);
        activityList.push({
          id: `dose-${dose.id}`,
          type: 'dose',
          title: 'Dose Administered',
          description: protocol
            ? `${protocol.name} - ${dose.amount_mg}mcg`
            : `Dose of ${dose.amount_mg}mcg`,
          timestamp: dose.logged_at,
          metadata: {
            Amount: `${dose.amount_mg}mcg`,
            ...(dose.site && { Site: dose.site }),
          },
        });
      });
    }

    // Convert inventory to activities
    if (selectedFilter.value === 'all' || selectedFilter.value === 'inventory') {
      inventory.forEach(item => {
        const protocol = protocols.find(p => p.id === item.protocol_id);
        const peptideName = protocol?.peptide_name || 'Unknown Peptide';
        activityList.push({
          id: `inventory-${item.id}`,
          type: 'inventory',
          title: 'Inventory Added',
          description: `${peptideName} - ${item.quantity_mg || 0}mg`,
          timestamp: item.purchase_date || item.created_at || new Date().toISOString(),
          metadata: {
            Quantity: `${item.quantity_mg || 0}mg`,
            ...(item.cost_per_mg && { Cost: `$${item.cost_per_mg}/mg` }),
          },
        });
      });
    }

    // Convert protocols to activities
    if (selectedFilter.value === 'all' || selectedFilter.value === 'protocols') {
      protocols.forEach(protocol => {
        activityList.push({
          id: `protocol-${protocol.id}`,
          type: 'protocol',
          title: 'Protocol Created',
          description: `${protocol.name} - ${protocol.peptide_name}`,
          timestamp: protocol.created_at || new Date().toISOString(),
          metadata: {
            Peptide: protocol.peptide_name,
            ...(protocol.target_concentration_mg_ml && {
              Concentration: `${protocol.target_concentration_mg_ml}mg/mL`,
            }),
          },
        });
      });
    }

    // Sort by timestamp
    activities.value = activityList
      .sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
      .slice(0, pageSize);

    hasMore.value = activityList.length > pageSize;
  } catch (error) {
    console.error('Failed to load activity:', error);
  } finally {
    loading.value = false;
  }
}

function loadMore() {
  loadingMore.value = true;
  page.value++;

  setTimeout(() => {
    // In a real implementation, fetch more data here
    loadingMore.value = false;
    hasMore.value = false; // No more data for now
  }, 500);
}

function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('en-US', {
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  });
}

function getActivityIcon(type: ActivityType): string {
  const icons: Record<ActivityType, string> = {
    dose: '💉',
    inventory: '📦',
    protocol: '📋',
    alert: '🔔',
    backup: '💾',
  };
  return icons[type] || '📌';
}

function handleAction(action: { label: string; handler: () => void }) {
  action.handler();
}
</script>

<style scoped>
.activity-timeline {
  padding: 24px;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  flex-wrap: wrap;
  gap: 16px;
}

.activity-timeline h3 {
  font-size: 24px;
  font-weight: 700;
  margin: 0;
  color: #1a1a1a;
}

.timeline-filters {
  display: flex;
  gap: 8px;
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  background: white;
  cursor: pointer;
}

.refresh-btn {
  padding: 8px 12px;
  background: white;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 18px;
  cursor: pointer;
  transition: all 0.2s;
}

.refresh-btn:hover {
  background: #f5f5f5;
  transform: rotate(90deg);
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

/* Timeline Groups */
.timeline-group {
  margin-bottom: 32px;
}

.date-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: 2px solid #e0e0e0;
}

.date-label {
  font-size: 16px;
  font-weight: 700;
  color: #1a1a1a;
}

.date-count {
  font-size: 13px;
  color: #666;
}

/* Timeline Items */
.timeline-items {
  position: relative;
  padding-left: 40px;
}

.timeline-item {
  position: relative;
  margin-bottom: 24px;
  padding-bottom: 24px;
}

.timeline-item:last-child .item-line {
  display: none;
}

.item-icon {
  position: absolute;
  left: -40px;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;
  background: white;
  border: 3px solid #e0e0e0;
  border-radius: 50%;
  z-index: 2;
}

.item-icon.icon-dose {
  border-color: #2196f3;
  background: #e3f2fd;
}

.item-icon.icon-inventory {
  border-color: #ff9800;
  background: #fff3e0;
}

.item-icon.icon-protocol {
  border-color: #4caf50;
  background: #e8f5e9;
}

.item-icon.icon-alert {
  border-color: #f44336;
  background: #ffebee;
}

.item-line {
  position: absolute;
  left: -21px;
  top: 40px;
  bottom: -24px;
  width: 2px;
  background: #e0e0e0;
  z-index: 1;
}

.item-content {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 10px;
  padding: 16px;
  transition: all 0.3s;
}

.item-content:hover {
  border-color: #1976d2;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  transform: translateX(4px);
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
  gap: 16px;
}

.item-title {
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.item-time {
  font-size: 13px;
  color: #999;
  white-space: nowrap;
}

.item-description {
  font-size: 14px;
  color: #666;
  margin-bottom: 12px;
}

.item-metadata {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 12px;
}

.metadata-tag {
  padding: 4px 10px;
  background: #f0f0f0;
  border-radius: 12px;
  font-size: 12px;
  color: #666;
}

.metadata-tag strong {
  color: #1a1a1a;
}

.item-actions {
  margin-top: 12px;
}

.action-btn {
  padding: 6px 14px;
  background: #1976d2;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: #1565c0;
  transform: translateY(-2px);
}

/* Load More */
.load-more {
  text-align: center;
  padding: 20px;
}

.load-more-btn {
  padding: 12px 32px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  color: #666;
  cursor: pointer;
  transition: all 0.2s;
}

.load-more-btn:hover:not(:disabled) {
  border-color: #1976d2;
  color: #1976d2;
  transform: translateY(-2px);
}

.load-more-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Dark mode */
:global(.dark-mode) .activity-timeline h3,
:global(.dark-mode) .date-label,
:global(.dark-mode) .item-title {
  color: #fff;
}

:global(.dark-mode) .filter-select,
:global(.dark-mode) .refresh-btn {
  background: #2a2a2a;
  border-color: #404040;
  color: #fff;
}

:global(.dark-mode) .date-header {
  border-bottom-color: #404040;
}

:global(.dark-mode) .item-icon {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .item-line {
  background: #404040;
}

:global(.dark-mode) .item-content {
  background: #2a2a2a;
  border-color: #404040;
}

:global(.dark-mode) .item-content:hover {
  border-color: #64b5f6;
}

:global(.dark-mode) .metadata-tag {
  background: #1a1a1a;
  color: #aaa;
}

:global(.dark-mode) .metadata-tag strong {
  color: #fff;
}

:global(.dark-mode) .load-more-btn {
  background: #2a2a2a;
  border-color: #404040;
  color: #aaa;
}

/* Responsive */
@media (max-width: 768px) {
  .activity-timeline {
    padding: 16px;
  }

  .timeline-header {
    flex-direction: column;
    align-items: stretch;
  }

  .timeline-items {
    padding-left: 32px;
  }

  .item-icon {
    left: -32px;
    width: 32px;
    height: 32px;
    font-size: 16px;
  }

  .item-line {
    left: -17px;
  }

  .item-header {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}
</style>
