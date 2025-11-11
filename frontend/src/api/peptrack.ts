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
  protocols: any[];
  doseLogs: any[];
  literature: any[];
}

// Backup API calls

export async function exportBackupData() {
  return invoke<BackupData>("export_backup_data");
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
