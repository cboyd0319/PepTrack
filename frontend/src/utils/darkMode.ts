import { ref, watch, onMounted } from 'vue';

export type ThemeMode = 'light' | 'dark' | 'auto';

const STORAGE_KEY = 'peptrack-theme-mode';

// Reactive theme state
const themeMode = ref<ThemeMode>('auto');
const isDark = ref(false);

// Initialize theme from localStorage or default to auto
function initializeTheme() {
  try {
    const stored = localStorage?.getItem?.(STORAGE_KEY);
    if (stored && (stored === 'light' || stored === 'dark' || stored === 'auto')) {
      themeMode.value = stored as ThemeMode;
    }
    updateTheme();
  } catch (e) {
    // localStorage not available (e.g., in tests or SSR)
    updateTheme();
  }
}

// Update the actual theme based on mode
function updateTheme() {
  try {
    if (themeMode.value === 'auto') {
      // Use system preference
      isDark.value = window?.matchMedia?.('(prefers-color-scheme: dark)')?.matches ?? false;
    } else {
      isDark.value = themeMode.value === 'dark';
    }

    // Apply theme to document
    if (typeof document !== 'undefined' && document.documentElement) {
      if (isDark.value) {
        document.documentElement.classList.add('dark-mode');
        document.documentElement.setAttribute('data-theme', 'dark');
      } else {
        document.documentElement.classList.remove('dark-mode');
        document.documentElement.setAttribute('data-theme', 'light');
      }
    }
  } catch (e) {
    // DOM not available (e.g., in tests or SSR)
  }
}

// Set theme mode
function setThemeMode(mode: ThemeMode) {
  themeMode.value = mode;
  try {
    localStorage?.setItem?.(STORAGE_KEY, mode);
  } catch (e) {
    // localStorage not available
  }
  updateTheme();
}

// Listen for system theme changes when in auto mode
function listenToSystemTheme() {
  try {
    if (typeof window === 'undefined' || !window.matchMedia) {
      return () => {}; // No-op cleanup in non-browser environments
    }
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = () => {
      if (themeMode.value === 'auto') {
        updateTheme();
      }
    };
    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  } catch (e) {
    return () => {}; // No-op cleanup on error
  }
}

// Composable for using dark mode in components
export function useDarkMode() {
  onMounted(() => {
    initializeTheme();
    const cleanup = listenToSystemTheme();
    return cleanup;
  });

  watch(themeMode, updateTheme);

  return {
    themeMode,
    isDark,
    setThemeMode,
  };
}

// Auto-initialize theme on import
initializeTheme();
listenToSystemTheme();

export { themeMode, isDark, setThemeMode, initializeTheme };
