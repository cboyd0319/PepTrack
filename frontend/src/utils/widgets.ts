import { ref } from 'vue';

export type WidgetType =
  | 'quick-stats'
  | 'recent-doses'
  | 'protocols'
  | 'inventory'
  | 'alerts'
  | 'price-trends';

export interface WidgetConfig {
  id: string;
  type: WidgetType;
  title: string;
  enabled: boolean;
  order: number;
  size: 'small' | 'medium' | 'large';
}

const STORAGE_KEY = 'peptrack-dashboard-widgets';

const defaultWidgets: WidgetConfig[] = [
  { id: 'quick-stats', type: 'quick-stats', title: 'Quick Stats', enabled: true, order: 0, size: 'large' },
  { id: 'recent-doses', type: 'recent-doses', title: 'Recent Doses', enabled: true, order: 1, size: 'medium' },
  { id: 'protocols', type: 'protocols', title: 'Active Protocols', enabled: true, order: 2, size: 'medium' },
  { id: 'inventory', type: 'inventory', title: 'Inventory Status', enabled: true, order: 3, size: 'medium' },
  { id: 'alerts', type: 'alerts', title: 'Recent Alerts', enabled: true, order: 4, size: 'medium' },
  { id: 'price-trends', type: 'price-trends', title: 'Price Trends', enabled: false, order: 5, size: 'large' },
];

// Load widgets from localStorage or use defaults
function loadWidgets(): WidgetConfig[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored) {
    try {
      return JSON.parse(stored);
    } catch (e) {
      console.error('Failed to parse widget config:', e);
      return [...defaultWidgets];
    }
  }
  return [...defaultWidgets];
}

// Save widgets to localStorage
function saveWidgets(widgets: WidgetConfig[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(widgets));
}

// Reactive widget configuration
export const widgetConfigs = ref<WidgetConfig[]>(loadWidgets());

// Toggle widget enabled state
export function toggleWidget(widgetId: string) {
  const widget = widgetConfigs.value.find((w) => w.id === widgetId);
  if (widget) {
    widget.enabled = !widget.enabled;
    saveWidgets(widgetConfigs.value);
  }
}

// Move widget up in order
export function moveWidgetUp(widgetId: string) {
  const index = widgetConfigs.value.findIndex((w) => w.id === widgetId);
  if (index > 0) {
    const currentWidget = widgetConfigs.value[index];
    const previousWidget = widgetConfigs.value[index - 1];

    if (currentWidget && previousWidget) {
      const temp = currentWidget.order;
      currentWidget.order = previousWidget.order;
      previousWidget.order = temp;

      // Re-sort by order
      widgetConfigs.value.sort((a, b) => a.order - b.order);
      saveWidgets(widgetConfigs.value);
    }
  }
}

// Move widget down in order
export function moveWidgetDown(widgetId: string) {
  const index = widgetConfigs.value.findIndex((w) => w.id === widgetId);
  if (index >= 0 && index < widgetConfigs.value.length - 1) {
    const currentWidget = widgetConfigs.value[index];
    const nextWidget = widgetConfigs.value[index + 1];

    if (currentWidget && nextWidget) {
      const temp = currentWidget.order;
      currentWidget.order = nextWidget.order;
      nextWidget.order = temp;

      // Re-sort by order
      widgetConfigs.value.sort((a, b) => a.order - b.order);
      saveWidgets(widgetConfigs.value);
    }
  }
}

// Change widget size
export function changeWidgetSize(widgetId: string, size: 'small' | 'medium' | 'large') {
  const widget = widgetConfigs.value.find((w) => w.id === widgetId);
  if (widget) {
    widget.size = size;
    saveWidgets(widgetConfigs.value);
  }
}

// Reset widgets to defaults
export function resetWidgets() {
  widgetConfigs.value = [...defaultWidgets];
  saveWidgets(widgetConfigs.value);
}

// Get enabled widgets sorted by order
export function getEnabledWidgets(): WidgetConfig[] {
  return widgetConfigs.value.filter((w) => w.enabled).sort((a, b) => a.order - b.order);
}
