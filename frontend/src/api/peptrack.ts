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
