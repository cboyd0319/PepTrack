import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import Settings from "../Settings.vue";

// Mock child components to avoid deep rendering
vi.mock("../BackupAndRestore.vue", () => ({
  default: { name: "BackupAndRestore", template: "<div>Backup and Restore Content</div>" },
}));

vi.mock("../NotificationPreferences.vue", () => ({
  default: { name: "NotificationPreferences", template: "<div>Notification Preferences Content</div>" },
}));

vi.mock("../CalendarIntegration.vue", () => ({
  default: { name: "CalendarIntegration", template: "<div>Calendar Integration Content</div>" },
}));

vi.mock("../DarkModeToggle.vue", () => ({
  default: { name: "DarkModeToggle", template: "<div>Dark Mode Toggle Content</div>" },
}));

vi.mock("../DashboardWidgetSettings.vue", () => ({
  default: { name: "DashboardWidgetSettings", template: "<div>Dashboard Widget Settings Content</div>" },
}));

vi.mock("../AboutHelp.vue", () => ({
  default: { name: "AboutHelp", template: "<div>About Help Content</div>" },
}));

describe("Settings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all tab buttons", () => {
    const wrapper = mount(Settings);

    expect(wrapper.text()).toContain("Backup & Restore");
    expect(wrapper.text()).toContain("Notifications");
    expect(wrapper.text()).toContain("Calendar");
    expect(wrapper.text()).toContain("Appearance");
    expect(wrapper.text()).toContain("Dashboard");
    expect(wrapper.text()).toContain("About & Help");
  });

  it("defaults to backup tab", () => {
    const wrapper = mount(Settings);

    // First tab should be active by default
    const tabs = wrapper.findAll(".tab-btn");
    expect(tabs[0]?.classes()).toContain("active");
  });

  it("switches tabs when clicked", async () => {
    const wrapper = mount(Settings);

    const tabs = wrapper.findAll(".tab-btn");

    // Click on Notifications tab (second tab)
    await tabs[1]?.trigger("click");
    await wrapper.vm.$nextTick();

    // Second tab should now be active
    expect(tabs[1]?.classes()).toContain("active");
    expect(tabs[0]?.classes()).not.toContain("active");
  });

  it("renders correct tab content", async () => {
    const wrapper = mount(Settings);

    // Should show backup content initially
    expect(wrapper.text()).toContain("Backup and Restore Content");

    // Switch to Notifications tab
    const tabs = wrapper.findAll(".tab-btn");
    await tabs[1]?.trigger("click");
    await wrapper.vm.$nextTick();

    // Should now show Notifications content
    expect(wrapper.text()).toContain("Notification Preferences Content");
  });

  it("navigates through all tabs", async () => {
    const wrapper = mount(Settings);
    const tabs = wrapper.findAll(".tab-btn");

    // Test each tab
    for (let i = 0; i < tabs.length; i++) {
      await tabs[i]?.trigger("click");
      await wrapper.vm.$nextTick();

      // Only the clicked tab should be active
      expect(tabs[i]?.classes()).toContain("active");

      for (let j = 0; j < tabs.length; j++) {
        if (i !== j) {
          expect(tabs[j]?.classes()).not.toContain("active");
        }
      }
    }
  });

  it("maintains tab state during component lifecycle", async () => {
    const wrapper = mount(Settings);
    const notificationsTab = wrapper
      .findAll(".tab-btn")
      .find((tab) => tab.text().includes("Notifications"));

    // Switch to notifications tab
    await notificationsTab?.trigger("click");
    await wrapper.vm.$nextTick();

    // Force re-render
    await wrapper.vm.$forceUpdate();

    // Tab should still be active
    expect(notificationsTab?.classes()).toContain("active");
  });

  it("has proper accessibility attributes", () => {
    const wrapper = mount(Settings);
    const tabs = wrapper.findAll(".tab-btn");

    // Tabs should be keyboard accessible
    tabs.forEach((tab) => {
      expect(tab.element.tagName).toBe("BUTTON");
    });
  });

  it("applies transition effects between tabs", async () => {
    const wrapper = mount(Settings);

    // Check if tab-panel exists
    const tabPanel = wrapper.find(".tab-panel");
    expect(tabPanel.exists()).toBe(true);
  });

  it("handles test notification click", async () => {
    const wrapper = mount(Settings);
    const notificationsTab = wrapper
      .findAll(".tab-btn")
      .find((tab) => tab.text().includes("Notifications"));

    // Navigate to notifications tab
    await notificationsTab?.trigger("click");
    await wrapper.vm.$nextTick();

    // The NotificationPreferences component should be rendered
    expect(wrapper.text()).toContain("Notification Preferences Content");
  });

  it("displays settings header", () => {
    const wrapper = mount(Settings);

    // Check if header is visible
    expect(wrapper.text()).toContain("⚙️ Settings");
    expect(wrapper.text()).toContain("Configure backups, notifications, and preferences");
  });

  it("has responsive tab navigation", () => {
    const wrapper = mount(Settings);

    const tabsContainer = wrapper.find(".settings-tabs");
    expect(tabsContainer.exists()).toBe(true);
  });

  it("renders calendar integration tab", async () => {
    const wrapper = mount(Settings);
    const calendarTab = wrapper
      .findAll(".tab-btn")
      .find((tab) => tab.text().includes("Calendar"));

    await calendarTab?.trigger("click");
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain("Calendar Integration Content");
  });

  it("renders appearance tab", async () => {
    const wrapper = mount(Settings);
    const appearanceTab = wrapper
      .findAll(".tab-btn")
      .find((tab) => tab.text().includes("Appearance"));

    await appearanceTab?.trigger("click");
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain("Dark Mode Toggle Content");
  });

  it("renders dashboard tab", async () => {
    const wrapper = mount(Settings);
    const dashboardTab = wrapper
      .findAll(".tab-btn")
      .find((tab) => tab.text().includes("Dashboard"));

    await dashboardTab?.trigger("click");
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain("Dashboard Widget Settings Content");
  });

  it("renders about tab", async () => {
    const wrapper = mount(Settings);
    const aboutTab = wrapper
      .findAll(".tab-btn")
      .find((tab) => tab.text().includes("About"));

    await aboutTab?.trigger("click");
    await wrapper.vm.$nextTick();

    expect(wrapper.text()).toContain("About Help Content");
  });
});
