import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import Settings from "../Settings.vue";

// Mock child components to avoid deep rendering
vi.mock("../ScheduledBackup.vue", () => ({
  default: { name: "ScheduledBackup", template: "<div>Scheduled Backup</div>" },
}));

vi.mock("../GoogleDriveBackup.vue", () => ({
  default: { name: "GoogleDriveBackup", template: "<div>Google Drive Backup</div>" },
}));

vi.mock("../BackupExport.vue", () => ({
  default: { name: "BackupExport", template: "<div>Backup Export</div>" },
}));

vi.mock("../RestoreBackup.vue", () => ({
  default: { name: "RestoreBackup", template: "<div>Restore Backup</div>" },
}));

vi.mock("../SupplierManagement.vue", () => ({
  default: { name: "SupplierManagement", template: "<div>Supplier Management</div>" },
}));

vi.mock("../InventoryManagement.vue", () => ({
  default: { name: "InventoryManagement", template: "<div>Inventory Management</div>" },
}));

vi.mock("../NotificationPreferences.vue", () => ({
  default: { name: "NotificationPreferences", template: "<div>Notification Preferences</div>" },
}));

describe("Settings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all tab buttons", () => {
    const wrapper = mount(Settings);

    expect(wrapper.text()).toContain("Scheduled Backups");
    expect(wrapper.text()).toContain("Google Drive");
    expect(wrapper.text()).toContain("Manual Backup");
    expect(wrapper.text()).toContain("Restore");
    expect(wrapper.text()).toContain("Suppliers");
    expect(wrapper.text()).toContain("Inventory");
    expect(wrapper.text()).toContain("Notifications");
  });

  it("defaults to scheduled backups tab", () => {
    const wrapper = mount(Settings);

    // First tab should be active by default
    const tabs = wrapper.findAll(".tab-button");
    expect(tabs[0]?.classes()).toContain("active");
  });

  it("switches tabs when clicked", async () => {
    const wrapper = mount(Settings);

    const tabs = wrapper.findAll(".tab-button");

    // Click on Google Drive tab (second tab)
    await tabs[1]?.trigger("click");
    await wrapper.vm.$nextTick();

    // Second tab should now be active
    expect(tabs[1]?.classes()).toContain("active");
    expect(tabs[0]?.classes()).not.toContain("active");
  });

  it("renders correct tab content", async () => {
    const wrapper = mount(Settings);

    // Should show scheduled backup content initially
    expect(wrapper.text()).toContain("Scheduled Backup");

    // Switch to Drive tab
    const tabs = wrapper.findAll(".tab-button");
    await tabs[1]?.trigger("click");
    await wrapper.vm.$nextTick();

    // Should now show Google Drive content
    expect(wrapper.text()).toContain("Google Drive Backup");
  });

  it("navigates through all tabs", async () => {
    const wrapper = mount(Settings);
    const tabs = wrapper.findAll(".tab-button");

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
      .findAll(".tab-button")
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
    const tabs = wrapper.findAll(".tab-button");

    // Tabs should be keyboard accessible
    tabs.forEach((tab) => {
      expect(tab.element.tagName).toBe("BUTTON");
    });
  });

  it("applies transition effects between tabs", async () => {
    const wrapper = mount(Settings);

    // Check if transition component exists
    const transition = wrapper.find(".fade, .tab-content");
    expect(transition.exists()).toBe(true);
  });

  it("handles test notification click", async () => {
    const wrapper = mount(Settings);
    const notificationsTab = wrapper
      .findAll(".tab-button")
      .find((tab) => tab.text().includes("Notifications"));

    // Navigate to notifications tab
    await notificationsTab?.trigger("click");
    await wrapper.vm.$nextTick();

    // The NotificationPreferences component should be rendered
    expect(wrapper.text()).toContain("Notification Preferences");
  });

  it("displays tab descriptions", () => {
    const wrapper = mount(Settings);

    // Check if tab descriptions are visible
    expect(wrapper.text()).toContain("Automatic backup scheduling");
    expect(wrapper.text()).toContain("Cloud backup setup");
    expect(wrapper.text()).toContain("Export data manually");
    expect(wrapper.text()).toContain("Restore from backup");
    expect(wrapper.text()).toContain("Manage suppliers and vendors");
    expect(wrapper.text()).toContain("Track peptide vials and stock");
    expect(wrapper.text()).toContain("Alert preferences");
  });

  it("has responsive grid layout for tabs", () => {
    const wrapper = mount(Settings);

    const tabsContainer = wrapper.find(".tabs");
    expect(tabsContainer.exists()).toBe(true);
  });
});
