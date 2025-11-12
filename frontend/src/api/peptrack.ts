import { invoke } from "@tauri-apps/api/core";

export interface PeptideProtocol {
  id: string;
  name: string;
  peptide_name: string;
  notes?: string | null;
  target_concentration_mg_ml?: number | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProtocolPayload {
  name: string;
  peptideName: string;
  notes?: string;
  targetConcentrationMgMl?: number | null;
}

export type SummaryFormat = "Markdown" | "Json";

export interface SummarizeResponse {
  provider: string;
  output: string;
}

export async function listProtocols() {
  return invoke<PeptideProtocol[]>("list_protocols");
}

export async function saveProtocol(payload: CreateProtocolPayload) {
  return invoke<PeptideProtocol>("save_protocol", {
    payload: {
      name: payload.name,
      peptideName: payload.peptideName,
      notes: payload.notes,
      targetConcentrationMgMl: payload.targetConcentrationMgMl,
    },
  });
}

export async function summarizeContent(params: {
  title: string;
  content: string;
  format?: SummaryFormat;
}) {
  return invoke<SummarizeResponse>("summarize_text", {
    payload: params,
  });
}

// Literature types

export interface LiteratureEntry {
  id: string;
  source: string;
  title: string;
  url?: string | null;
  summary?: string | null;
  relevance_score?: number | null;
  indexed_at: string;
}

export interface LiteratureResult {
  source: string;
  title: string;
  url?: string | null;
  doi?: string | null;
  authors?: string | null;
  published_date?: string | null;
  journal?: string | null;
  abstract_text?: string | null;
}

export interface LiteratureSearchResult {
  source: string;
  results: LiteratureResult[];
}

export interface SearchLiteraturePayload {
  query: string;
  maxResults?: number;
  sources?: string[];
}

// Literature API calls

export async function listLiterature() {
  return invoke<LiteratureEntry[]>("list_literature");
}

export async function searchCachedLiterature(query: string) {
  return invoke<LiteratureEntry[]>("search_cached_literature", { query });
}

export async function searchLiterature(payload: SearchLiteraturePayload) {
  return invoke<LiteratureSearchResult[]>("search_literature", { payload });
}

// Dose logging types

export interface DoseLog {
  id: string;
  protocol_id: string;
  site: string;
  amount_mg: number;
  notes?: string | null;
  logged_at: string;
}

export interface LogDosePayload {
  protocolId: string;
  site: string;
  amountMg: number;
  notes?: string;
}

// Dose logging API calls

export async function logDose(payload: LogDosePayload) {
  return invoke<DoseLog>("log_dose", { payload });
}

export async function listDoseLogs() {
  return invoke<DoseLog[]>("list_dose_logs");
}

export async function listDoseLogsForProtocol(protocolId: string) {
  return invoke<DoseLog[]>("list_dose_logs_for_protocol", { protocolId });
}

export async function deleteDoseLog(logId: string) {
  return invoke<void>("delete_dose_log", { logId });
}

// AI availability types

export interface AiAvailabilityStatus {
  codexAvailable: boolean;
  claudeAvailable: boolean;
  anyAvailable: boolean;
  preferredProvider?: string | null;
}

// AI availability check

export async function checkAiAvailability() {
  return invoke<AiAvailabilityStatus>("check_ai_availability");
}

// Backup types

export interface BackupMetadata {
  exportDate: string;
  protocolsCount: number;
  dosesCount: number;
  literatureCount: number;
  appVersion: string;
}

export interface BackupData {
  metadata: BackupMetadata;
  protocols: PeptideProtocol[];
  doseLogs: DoseLog[];
  literature: LiteratureEntry[];
}

// Backup API calls

export async function exportBackupData(password?: string) {
  return invoke<string>("export_backup_data", { password: password || null });
}

export async function getBackupFilePath() {
  return invoke<string>("get_backup_file_path");
}

// Google Drive types

export interface DriveOAuthConfig {
  clientId: string;
  clientSecret: string;
}

export interface DriveTokens {
  accessToken: string;
  refreshToken?: string | null;
  expiresIn?: number | null;
}

export interface DriveStatus {
  connected: boolean;
  email?: string | null;
}

export interface AuthUrlResponse {
  authUrl: string;
  state: string;
}

// Google Drive API calls

export async function startDriveOAuth(config: DriveOAuthConfig) {
  return invoke<AuthUrlResponse>("start_drive_oauth", { config });
}

export async function completeDriveOAuth(
  config: DriveOAuthConfig,
  code: string,
  stateParam: string
) {
  return invoke<DriveStatus>("complete_drive_oauth", {
    config,
    code,
    stateParam,
  });
}

export async function checkDriveStatus() {
  return invoke<DriveStatus>("check_drive_status");
}

export async function disconnectDrive() {
  return invoke<void>("disconnect_drive");
}

export async function uploadToDrive(filename: string, content: string) {
  return invoke<string>("upload_to_drive", { filename, content });
}

export async function openExternalLink(url: string) {
  return invoke<void>("open_external_url", { url });
}

// Scheduled Backup types

export type BackupFrequency =
  | "Hourly"
  | "Weekly"
  | "Manual"
  | { DailyAt: { hour: number } };

export type BackupDestination = "Local" | "GoogleDrive";

export interface CleanupSettings {
  enabled: boolean;
  keepLastN?: number | null;
  olderThanDays?: number | null;
}

export interface BackupSchedule {
  enabled: boolean;
  frequency: BackupFrequency;
  destinations: BackupDestination[];
  lastBackup?: string | null;
  nextBackup?: string | null;
  backupOnClose?: boolean;
  compress?: boolean;
  cleanupSettings?: CleanupSettings;
  maxRetries?: number;
}

export interface BackupHistoryEntry {
  timestamp: string;
  destinations: BackupDestination[];
  success: boolean;
  errorMessage?: string | null;
  sizeBytes?: number | null;
  compressed: boolean;
}

export interface BackupProgress {
  isRunning: boolean;
  currentStep: string;
  completedSteps: string[];
  failedSteps: string[];
}

export interface RestoreCounts {
  protocols: number;
  doseLogs: number;
  literature: number;
}

export interface RestoreResult {
  success: boolean;
  counts: RestoreCounts;
  metadata: BackupMetadata;
}

export interface BackupPreview {
  metadata: BackupMetadata;
  protocolsCount: number;
  doseLogsCount: number;
  literatureCount: number;
}

// Scheduled Backup API calls

export async function getBackupSchedule() {
  return invoke<BackupSchedule>("get_backup_schedule");
}

export async function updateBackupSchedule(schedule: BackupSchedule) {
  return invoke<BackupSchedule>("update_backup_schedule", { schedule });
}

export async function triggerManualBackup() {
  return invoke<string>("trigger_manual_backup");
}

export async function getBackupHistory() {
  return invoke<BackupHistoryEntry[]>("get_backup_history");
}

export async function getBackupProgress() {
  return invoke<BackupProgress>("get_backup_progress");
}

// Restore API calls

export async function restoreFromBackup(filePath: string, password?: string) {
  return invoke<RestoreResult>("restore_from_backup", { filePath, password: password || null });
}

export async function previewBackup(filePath: string, password?: string) {
  return invoke<BackupPreview>("preview_backup", { filePath, password: password || null });
}

// Supplier types

export interface Supplier {
  id: string;
  name: string;
  contact_email?: string | null;
  contact_phone?: string | null;
  website?: string | null;
  notes?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateSupplierPayload {
  name: string;
  contactEmail?: string;
  contactPhone?: string;
  website?: string;
  notes?: string;
}

export interface UpdateSupplierPayload {
  name?: string;
  contactEmail?: string;
  contactPhone?: string;
  website?: string;
  notes?: string;
}

// Inventory types

export type VialStatus = "sealed" | "opened" | "empty" | "expired";

export interface InventoryItem {
  id: string;
  protocol_id: string;
  supplier_id?: string | null;
  vial_number?: string | null;
  vial_status: VialStatus;
  purchase_date?: string | null;
  expiry_date?: string | null;
  cost_per_mg?: number | null;
  quantity_mg?: number | null;
  quantity_remaining_mg?: number | null; // NEW: Track remaining quantity
  concentration_mg_ml?: number | null;
  batch_number?: string | null;
  lot_number?: string | null;
  low_stock_threshold_mg?: number | null; // NEW: Alert threshold
  notes?: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateInventoryPayload {
  protocolId: string;
  supplierId?: string;
  vialNumber?: string;
  vialStatus?: VialStatus;
  purchaseDate?: string;
  expiryDate?: string;
  costPerMg?: number;
  quantityMg?: number;
  quantityRemainingMg?: number;
  concentrationMgMl?: number;
  batchNumber?: string;
  lotNumber?: string;
  lowStockThresholdMg?: number;
  notes?: string;
}

export interface UpdateInventoryPayload {
  supplierId?: string;
  vialNumber?: string;
  vialStatus?: VialStatus;
  purchaseDate?: string;
  expiryDate?: string;
  costPerMg?: number;
  quantityMg?: number;
  quantityRemainingMg?: number;
  concentrationMgMl?: number;
  batchNumber?: string;
  lotNumber?: string;
  lowStockThresholdMg?: number;
  notes?: string;
}

// Supplier API calls

export async function createSupplier(payload: CreateSupplierPayload) {
  return invoke<Supplier>("create_supplier", { payload });
}

export async function listSuppliers() {
  return invoke<Supplier[]>("list_suppliers");
}

export async function getSupplier(supplierId: string) {
  return invoke<Supplier | null>("get_supplier", { supplierId });
}

export async function updateSupplier(
  supplierId: string,
  payload: UpdateSupplierPayload
) {
  return invoke<Supplier>("update_supplier", { supplierId, payload });
}

export async function deleteSupplier(supplierId: string) {
  return invoke<void>("delete_supplier", { supplierId });
}

// Inventory API calls

export async function createInventoryItem(payload: CreateInventoryPayload) {
  return invoke<InventoryItem>("create_inventory_item", { payload });
}

export async function listInventory() {
  return invoke<InventoryItem[]>("list_inventory");
}

export async function listInventoryByProtocol(protocolId: string) {
  return invoke<InventoryItem[]>("list_inventory_by_protocol", { protocolId });
}

export async function getInventoryItem(itemId: string) {
  return invoke<InventoryItem | null>("get_inventory_item", { itemId });
}

export async function updateInventoryItem(
  itemId: string,
  payload: UpdateInventoryPayload
) {
  return invoke<InventoryItem>("update_inventory_item", { itemId, payload });
}

export async function deleteInventoryItem(itemId: string) {
  return invoke<void>("delete_inventory_item", { itemId });
}

// ========== Analytics & Price History ==========

export interface PriceHistory {
  id: string;
  supplier_id: string;
  peptide_name: string;
  cost_per_mg: number;
  url?: string | null;
  in_stock?: boolean | null;
  notes?: string | null;
  recorded_at: string;
}

export interface AddPricePayload {
  supplierId: string;
  peptideName: string;
  costPerMg: number;
  url?: string;
  inStock?: boolean;
  notes?: string;
}

export interface PriceComparison {
  peptide_name: string;
  suppliers: SupplierPrice[];
  lowest_price: number;
  highest_price: number;
  average_price: number;
}

export interface SupplierPrice {
  supplier_id: string;
  supplier_name: string;
  cost_per_mg: number;
  in_stock?: boolean | null;
  recorded_at: string;
}

// Price History API calls

export async function addPriceHistory(payload: AddPricePayload) {
  return invoke<PriceHistory>("add_price_history", { payload });
}

export async function listPriceHistory(
  supplierId: string,
  peptideName?: string
) {
  return invoke<PriceHistory[]>("list_price_history", {
    supplierId,
    peptideName,
  });
}

export async function getLatestPrice(supplierId: string, peptideName: string) {
  return invoke<PriceHistory | null>("get_latest_price", {
    supplierId,
    peptideName,
  });
}

export async function comparePrices(peptideName: string) {
  return invoke<PriceComparison>("compare_prices", { peptideName });
}

// ========== Website Scraper ==========

export interface PriceMatch {
  pricePerMg: number;
  context: string;
  patternType: string;
}

export async function scrapeSupplierWebsite(url: string, peptideName?: string) {
  return invoke<PriceMatch[]>("scrape_supplier_website", {
    url,
    peptideName,
  });
}

// ========== Alerts System ==========

export type AlertType =
  | "low_stock"
  | "expiring_soon"
  | "expired"
  | "price_increase"
  | "price_decrease"
  | "out_of_stock";

export type AlertSeverity = "info" | "warning" | "critical";

export interface Alert {
  id: string;
  alert_type: AlertType;
  severity: AlertSeverity;
  title: string;
  message: string;
  related_id?: string | null;
  related_type?: string | null;
  is_read: boolean;
  is_dismissed: boolean;
  created_at: string;
}

export interface CreateAlertPayload {
  alertType: AlertType;
  severity: AlertSeverity;
  title: string;
  message: string;
  relatedId?: string;
  relatedType?: string;
}

// Alert API calls

export async function createAlert(payload: CreateAlertPayload) {
  return invoke<Alert>("create_alert", { payload });
}

export async function listAlerts(includeDismissed?: boolean) {
  return invoke<Alert[]>("list_alerts", { includeDismissed });
}

export async function markAlertRead(alertId: string) {
  return invoke<void>("mark_alert_read", { alertId });
}

export async function dismissAlert(alertId: string) {
  return invoke<void>("dismiss_alert", { alertId });
}

export async function clearAllAlerts() {
  return invoke<void>("clear_all_alerts");
}

// ========== AI Summary History ==========

export interface SummaryHistory {
  id: string;
  title: string;
  original_content: string;
  summary_output: string;
  format: string;
  provider: string;
  created_at: string;
}

export interface SaveSummaryPayload {
  title: string;
  originalContent: string;
  summaryOutput: string;
  format: string;
  provider: string;
}

// Summary History API calls

export async function saveSummary(payload: SaveSummaryPayload) {
  return invoke<SummaryHistory>("save_summary", { payload });
}

export async function listSummaryHistory(limit?: number) {
  return invoke<SummaryHistory[]>("list_summary_history", { limit });
}

export async function deleteSummaryFromHistory(summaryId: string) {
  return invoke<void>("delete_summary", { summaryId });
}

// Dose Schedule types

export interface DoseSchedule {
  id: string;
  protocolId: string;
  protocolName: string;
  peptideName: string;
  amountMg: number;
  site?: string | null;
  timeOfDay: string; // Format: "HH:MM"
  daysOfWeek: number[]; // 0=Sunday, 1=Monday, ..., 6=Saturday
  enabled: boolean;
  notes?: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateSchedulePayload {
  protocolId: string;
  amountMg: number;
  site?: string;
  timeOfDay: string;
  daysOfWeek: number[];
  notes?: string;
}

export interface UpdateSchedulePayload {
  id: string;
  amountMg?: number;
  site?: string;
  timeOfDay?: string;
  daysOfWeek?: number[];
  enabled?: boolean;
  notes?: string;
}

// Dose Schedule API calls

export async function createDoseSchedule(payload: CreateSchedulePayload) {
  return invoke<DoseSchedule>("create_dose_schedule", { payload });
}

export async function listDoseSchedules() {
  return invoke<DoseSchedule[]>("list_dose_schedules");
}

export async function updateDoseSchedule(payload: UpdateSchedulePayload) {
  return invoke<DoseSchedule>("update_dose_schedule", { payload });
}

export async function deleteDoseSchedule(scheduleId: string) {
  return invoke<void>("delete_dose_schedule", { scheduleId });
}

export async function getPendingDoseReminders() {
  return invoke<DoseSchedule[]>("get_pending_dose_reminders");
}
