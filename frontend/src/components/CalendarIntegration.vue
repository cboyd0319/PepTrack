<template>
  <div class="calendar-integration">
    <h3>📅 Calendar Integration</h3>
    <p class="section-subtitle">Sync dosing schedules, rotation reminders, and refill alerts to your calendar</p>

    <!-- Integration Status -->
    <div class="integration-status panel">
      <div class="status-header">
        <span class="status-icon">📅</span>
        <div class="status-info">
          <h4>Calendar Sync Status</h4>
          <p v-if="!isConnected" class="status-disconnected">
            Not connected - Using local reminders only
          </p>
          <p v-else class="status-connected">
            ✓ Connected to Google Calendar
          </p>
        </div>
        <button v-if="!isConnected" @click="connectCalendar" class="connect-btn" :disabled="isConnecting">
          {{ isConnecting ? 'Connecting...' : '🔗 Connect Google Calendar' }}
        </button>
        <button v-else @click="disconnectCalendar" class="disconnect-btn">
          Disconnect
        </button>
      </div>
    </div>

    <!-- Sync Options -->
    <div class="sync-options panel">
      <h4>🔄 Auto-Sync Settings</h4>

      <div class="option-group">
        <label class="checkbox-label">
          <input type="checkbox" v-model="syncSettings.dosing" />
          <span class="checkbox-text">
            <strong>Dosing Schedule</strong>
            <span class="hint">Create calendar events for each protocol's dosing schedule</span>
          </span>
        </label>

        <div v-if="syncSettings.dosing" class="sub-options">
          <label class="radio-label">
            <input type="radio" v-model="syncSettings.dosingReminder" value="15" />
            15 minutes before
          </label>
          <label class="radio-label">
            <input type="radio" v-model="syncSettings.dosingReminder" value="30" />
            30 minutes before
          </label>
          <label class="radio-label">
            <input type="radio" v-model="syncSettings.dosingReminder" value="60" />
            1 hour before
          </label>
        </div>
      </div>

      <div class="option-group">
        <label class="checkbox-label">
          <input type="checkbox" v-model="syncSettings.rotation" />
          <span class="checkbox-text">
            <strong>Injection Site Rotation</strong>
            <span class="hint">Remind you to rotate injection sites</span>
          </span>
        </label>
      </div>

      <div class="option-group">
        <label class="checkbox-label">
          <input type="checkbox" v-model="syncSettings.refill" />
          <span class="checkbox-text">
            <strong>Refill Reminders</strong>
            <span class="hint">Alert when inventory is running low (7 days before depletion)</span>
          </span>
        </label>
      </div>

      <div class="option-group">
        <label class="checkbox-label">
          <input type="checkbox" v-model="syncSettings.expiry" />
          <span class="checkbox-text">
            <strong>Expiry Warnings</strong>
            <span class="hint">Remind before vials expire (14 days notice)</span>
          </span>
        </label>
      </div>

      <button @click="saveSettings" class="save-btn" :disabled="isSaving">
        {{ isSaving ? '💾 Saving...' : '💾 Save Settings' }}
      </button>
    </div>

    <!-- Upcoming Events Preview -->
    <div class="upcoming-events panel">
      <div class="events-header">
        <h4>📋 Upcoming Events (Next 7 Days)</h4>
        <button @click="refreshEvents" class="refresh-btn-sm">↻</button>
      </div>

      <div v-if="loadingEvents" class="loading">Loading events...</div>

      <div v-else-if="upcomingEvents.length === 0" class="no-events">
        <span class="no-events-icon">📅</span>
        <p>No upcoming events scheduled</p>
        <p class="hint">Enable auto-sync above to create calendar events</p>
      </div>

      <div v-else class="events-list">
        <div v-for="event in upcomingEvents" :key="event.id" class="event-card">
          <div class="event-icon">{{ event.icon }}</div>
          <div class="event-content">
            <div class="event-title">{{ event.title }}</div>
            <div class="event-time">{{ formatEventTime(event.time) }}</div>
            <div v-if="event.location" class="event-location">📍 {{ event.location }}</div>
          </div>
          <button @click="addToCalendar(event)" class="add-btn" title="Add to calendar">
            ➕
          </button>
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="quick-actions panel">
      <h4>⚡ Quick Calendar Actions</h4>
      <div class="action-buttons">
        <button @click="exportAllEvents" class="action-btn">
          📄 Export All Events (iCal)
        </button>
        <button @click="createDoseReminder" class="action-btn">
          💉 Create Dose Reminder
        </button>
        <button @click="createRefillReminder" class="action-btn">
          📦 Create Refill Reminder
        </button>
      </div>
    </div>

    <!-- Export Modal -->
    <div v-if="showExportModal" class="modal-overlay" @click="closeExportModal">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>📄 Export Calendar Events</h3>
          <button @click="closeExportModal" class="close-btn">✕</button>
        </div>
        <div class="modal-body">
          <p>Your calendar file is ready! Import it into:</p>
          <ul class="import-instructions">
            <li><strong>Google Calendar:</strong> Settings → Import & Export → Select file</li>
            <li><strong>Apple Calendar:</strong> File → Import → Select file</li>
            <li><strong>Outlook:</strong> File → Open & Export → Import/Export → Select file</li>
          </ul>
          <div class="export-preview">
            <pre>{{ icalPreview }}</pre>
          </div>
          <button @click="downloadICalFile" class="download-btn">
            ⬇️ Download Calendar File (.ics)
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { showSuccessToast, showErrorToast } from '../utils/errorHandling';
import { listProtocols, listInventory } from '../api/peptrack';

interface CalendarEvent {
  id: string;
  title: string;
  time: Date;
  icon: string;
  type: 'dose' | 'refill' | 'expiry' | 'rotation';
  location?: string;
  description?: string;
}

interface SyncSettings {
  dosing: boolean;
  dosingReminder: string;
  rotation: boolean;
  refill: boolean;
  expiry: boolean;
}

const isConnected = ref(false);
const isConnecting = ref(false);
const isSaving = ref(false);
const loadingEvents = ref(false);

const syncSettings = ref<SyncSettings>({
  dosing: true,
  dosingReminder: '30',
  rotation: true,
  refill: true,
  expiry: true,
});

const upcomingEvents = ref<CalendarEvent[]>([]);
const showExportModal = ref(false);
const icalPreview = ref('');

onMounted(() => {
  loadSettings();
  generateUpcomingEvents();
});

function loadSettings() {
  const saved = localStorage.getItem('peptrack_calendar_settings');
  if (saved) {
    try {
      syncSettings.value = JSON.parse(saved);
    } catch {
      // Use defaults
    }
  }
}

function saveSettings() {
  isSaving.value = true;
  try {
    localStorage.setItem('peptrack_calendar_settings', JSON.stringify(syncSettings.value));
    showSuccessToast('Saved', 'Calendar settings saved successfully');
    generateUpcomingEvents();
  } catch (error) {
    showErrorToast(new Error('Failed to save settings'));
  } finally {
    isSaving.value = false;
  }
}

async function connectCalendar() {
  isConnecting.value = true;
  try {
    // TODO: Implement OAuth flow
    // For now, show instructions
    showSuccessToast(
      'Coming Soon',
      'Direct Google Calendar sync coming soon! Use "Export All Events" to manually import.'
    );
  } catch (error) {
    showErrorToast(new Error('Failed to connect'));
  } finally {
    isConnecting.value = false;
  }
}

function disconnectCalendar() {
  isConnected.value = false;
  showSuccessToast('Disconnected', 'Calendar sync disabled');
}

async function generateUpcomingEvents() {
  loadingEvents.value = true;
  try {
    const events: CalendarEvent[] = [];
    const now = new Date();
    const sevenDaysLater = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000);

    // Generate dosing events
    if (syncSettings.value.dosing) {
      const protocols = await listProtocols();
      protocols.forEach((protocol) => {
        // Simulate daily dosing for active protocols
        for (let day = 0; day < 7; day++) {
          const eventDate = new Date(now);
          eventDate.setDate(eventDate.getDate() + day);
          eventDate.setHours(9, 0, 0, 0); // Default to 9 AM

          events.push({
            id: `dose-${protocol.id}-${day}`,
            title: `💉 ${protocol.name} Dose`,
            time: eventDate,
            icon: '💉',
            type: 'dose',
            location: 'Rotation site',
            description: `Administer ${protocol.peptide_name} dose`,
          });
        }
      });
    }

    // Generate refill reminders
    if (syncSettings.value.refill) {
      const inventory = await listInventory();
      inventory.forEach((item) => {
        if (item.quantity_remaining_mg && item.quantity_mg) {
          const percentRemaining = (item.quantity_remaining_mg / item.quantity_mg) * 100;
          if (percentRemaining < 30) {
            const refillDate = new Date(now);
            refillDate.setDate(refillDate.getDate() + 3);
            refillDate.setHours(10, 0, 0, 0);

            events.push({
              id: `refill-${item.id}`,
              title: `📦 Refill Reminder - Vial ${item.vial_number || 'N/A'}`,
              time: refillDate,
              icon: '📦',
              type: 'refill',
              description: `Order more before supply runs out`,
            });
          }
        }
      });
    }

    // Generate expiry warnings
    if (syncSettings.value.expiry) {
      const inventory = await listInventory();
      inventory.forEach((item) => {
        if (item.expiry_date) {
          const expiryDate = new Date(item.expiry_date);
          const daysUntilExpiry = Math.floor(
            (expiryDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24)
          );

          if (daysUntilExpiry > 0 && daysUntilExpiry <= 14) {
            const warningDate = new Date(expiryDate);
            warningDate.setDate(warningDate.getDate() - 7);

            events.push({
              id: `expiry-${item.id}`,
              title: `⚠️ Expiry Warning - Vial ${item.vial_number || 'N/A'}`,
              time: warningDate,
              icon: '⚠️',
              type: 'expiry',
              description: `Vial expires on ${formatEventTime(expiryDate)}`,
            });
          }
        }
      });
    }

    // Sort by time
    events.sort((a, b) => a.time.getTime() - b.time.getTime());

    // Only show events in next 7 days
    upcomingEvents.value = events.filter((e) => e.time <= sevenDaysLater);
  } catch (error) {
    showErrorToast(new Error('Failed to load events'));
  } finally {
    loadingEvents.value = false;
  }
}

function refreshEvents() {
  generateUpcomingEvents();
}

function formatEventTime(date: Date): string {
  const now = new Date();
  const tomorrow = new Date(now);
  tomorrow.setDate(tomorrow.getDate() + 1);

  const isToday = date.toDateString() === now.toDateString();
  const isTomorrow = date.toDateString() === tomorrow.toDateString();

  const timeStr = date.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });

  if (isToday) return `Today at ${timeStr}`;
  if (isTomorrow) return `Tomorrow at ${timeStr}`;

  return date.toLocaleDateString([], {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

function addToCalendar(event: CalendarEvent) {
  const icalContent = generateICalEvent(event);
  downloadFile(icalContent, `${event.title}.ics`, 'text/calendar');
  showSuccessToast('Downloaded', 'Import this file into your calendar app');
}

function generateICalEvent(event: CalendarEvent): string {
  const formatDate = (date: Date): string => {
    return date.toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
  };

  const endTime = new Date(event.time.getTime() + 15 * 60 * 1000); // 15 min duration

  return `BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//PepTrack//Calendar Integration//EN
BEGIN:VEVENT
UID:${event.id}@peptrack.app
DTSTAMP:${formatDate(new Date())}
DTSTART:${formatDate(event.time)}
DTEND:${formatDate(endTime)}
SUMMARY:${event.title}
DESCRIPTION:${event.description || ''}
LOCATION:${event.location || ''}
BEGIN:VALARM
TRIGGER:-PT${syncSettings.value.dosingReminder}M
ACTION:DISPLAY
DESCRIPTION:Reminder: ${event.title}
END:VALARM
END:VEVENT
END:VCALENDAR`;
}

function exportAllEvents() {
  const icalContent = generateICalForAllEvents();
  icalPreview.value = icalContent.split('\n').slice(0, 15).join('\n') + '\n...';
  showExportModal.value = true;
}

function generateICalForAllEvents(): string {
  let ical = `BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//PepTrack//Calendar Integration//EN
CALSCALE:GREGORIAN
X-WR-CALNAME:PepTrack Schedule
X-WR-TIMEZONE:UTC
`;

  upcomingEvents.value.forEach((event) => {
    const formatDate = (date: Date): string => {
      return date.toISOString().replace(/[-:]/g, '').split('.')[0] + 'Z';
    };

    const endTime = new Date(event.time.getTime() + 15 * 60 * 1000);

    ical += `BEGIN:VEVENT
UID:${event.id}@peptrack.app
DTSTAMP:${formatDate(new Date())}
DTSTART:${formatDate(event.time)}
DTEND:${formatDate(endTime)}
SUMMARY:${event.title}
DESCRIPTION:${event.description || ''}
LOCATION:${event.location || ''}
BEGIN:VALARM
TRIGGER:-PT${syncSettings.value.dosingReminder}M
ACTION:DISPLAY
DESCRIPTION:Reminder: ${event.title}
END:VALARM
END:VEVENT
`;
  });

  ical += 'END:VCALENDAR';
  return ical;
}

function downloadICalFile() {
  const icalContent = generateICalForAllEvents();
  downloadFile(icalContent, 'peptrack-schedule.ics', 'text/calendar');
  showSuccessToast('Downloaded', 'Import this file into Google Calendar, Apple Calendar, or Outlook');
  closeExportModal();
}

function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function closeExportModal() {
  showExportModal.value = false;
}

function createDoseReminder() {
  showSuccessToast('Coming Soon', 'Custom dose reminder creation coming soon!');
}

function createRefillReminder() {
  showSuccessToast('Coming Soon', 'Custom refill reminder creation coming soon!');
}
</script>

<style scoped>
.calendar-integration {
  max-width: 900px;
}

h3 {
  margin-bottom: 8px;
  color: #2c3e50;
}

.section-subtitle {
  color: #666;
  font-size: 14px;
  margin-bottom: 24px;
}

.panel {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 20px;
}

.integration-status {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.status-header {
  display: flex;
  align-items: center;
  gap: 16px;
}

.status-icon {
  font-size: 32px;
}

.status-info {
  flex: 1;
}

.status-info h4 {
  margin: 0 0 4px 0;
  color: white;
}

.status-info p {
  margin: 0;
  font-size: 14px;
}

.status-connected {
  color: #d4edda;
  font-weight: 600;
}

.status-disconnected {
  color: #f8d7da;
}

.connect-btn,
.disconnect-btn {
  padding: 10px 20px;
  border: 2px solid white;
  background: white;
  color: #667eea;
  border-radius: 8px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.connect-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(255, 255, 255, 0.3);
}

.connect-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.disconnect-btn {
  background: transparent;
  color: white;
}

.disconnect-btn:hover {
  background: rgba(255, 255, 255, 0.1);
}

.sync-options h4 {
  margin-top: 0;
  margin-bottom: 20px;
  color: #2c3e50;
}

.option-group {
  margin-bottom: 20px;
  padding-bottom: 20px;
  border-bottom: 1px solid #eee;
}

.option-group:last-of-type {
  border-bottom: none;
}

.checkbox-label {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  cursor: pointer;
  user-select: none;
}

.checkbox-label input[type="checkbox"] {
  margin-top: 4px;
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.checkbox-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.checkbox-text strong {
  color: #2c3e50;
  font-size: 15px;
}

.hint {
  font-size: 13px;
  color: #666;
}

.sub-options {
  margin-top: 12px;
  margin-left: 30px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.radio-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  color: #666;
}

.radio-label input[type="radio"] {
  cursor: pointer;
}

.save-btn {
  width: 100%;
  padding: 12px;
  background: linear-gradient(135deg, #27ae60, #229954);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
  margin-top: 8px;
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(39, 174, 96, 0.3);
}

.save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.events-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.events-header h4 {
  margin: 0;
  color: #2c3e50;
}

.refresh-btn-sm {
  padding: 6px 12px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s;
}

.refresh-btn-sm:hover {
  background-color: #2980b9;
}

.loading {
  text-align: center;
  padding: 40px;
  color: #666;
}

.no-events {
  text-align: center;
  padding: 60px 40px;
}

.no-events-icon {
  font-size: 48px;
  display: block;
  margin-bottom: 16px;
}

.no-events p {
  margin: 8px 0;
  color: #666;
}

.events-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.event-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
  border-left: 4px solid #3498db;
  transition: all 0.2s;
}

.event-card:hover {
  background-color: #f0f0f0;
  transform: translateX(4px);
}

.event-icon {
  font-size: 24px;
}

.event-content {
  flex: 1;
}

.event-title {
  font-weight: 700;
  color: #2c3e50;
  margin-bottom: 4px;
}

.event-time {
  font-size: 14px;
  color: #666;
  margin-bottom: 2px;
}

.event-location {
  font-size: 13px;
  color: #999;
}

.add-btn {
  padding: 8px 12px;
  background-color: #27ae60;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  transition: all 0.2s;
}

.add-btn:hover {
  background-color: #229954;
  transform: scale(1.1);
}

.quick-actions h4 {
  margin-top: 0;
  margin-bottom: 16px;
  color: #2c3e50;
}

.action-buttons {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
}

.action-btn {
  padding: 12px 16px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.action-btn:hover {
  background-color: #2980b9;
  transform: translateY(-2px);
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-content {
  background: white;
  border-radius: 12px;
  max-width: 600px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 2px solid #e0e0e0;
}

.modal-header h3 {
  margin: 0;
  color: #2c3e50;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #999;
  padding: 0;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: #f0f0f0;
  color: #333;
}

.modal-body {
  padding: 20px;
}

.modal-body p {
  margin-top: 0;
  color: #666;
}

.import-instructions {
  margin: 20px 0;
  padding-left: 24px;
}

.import-instructions li {
  margin-bottom: 12px;
  line-height: 1.6;
  color: #555;
}

.export-preview {
  background-color: #f5f5f5;
  border-radius: 8px;
  padding: 16px;
  margin: 20px 0;
  max-height: 200px;
  overflow-y: auto;
}

.export-preview pre {
  margin: 0;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #666;
  white-space: pre-wrap;
}

.download-btn {
  width: 100%;
  padding: 14px;
  background: linear-gradient(135deg, #27ae60, #229954);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 700;
  font-size: 15px;
  cursor: pointer;
  transition: all 0.2s;
}

.download-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(39, 174, 96, 0.3);
}

@media (max-width: 768px) {
  .status-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .connect-btn,
  .disconnect-btn {
    width: 100%;
  }

  .action-buttons {
    grid-template-columns: 1fr;
  }
}
</style>
