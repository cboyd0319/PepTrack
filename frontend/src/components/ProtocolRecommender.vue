<template>
  <div class="protocol-recommender">
    <h2>🤖 AI Protocol Assistant</h2>
    <p class="subtitle">Get AI-powered reconstitution and dosage recommendations</p>

    <!-- Peptide Selection -->
    <div class="search-section panel">
      <h3>1️⃣ Select Peptide</h3>
      <div class="peptide-selector">
        <input
          v-model="peptideSearch"
          type="text"
          placeholder="Enter peptide name (e.g., Tirzepatide, BPC-157)"
          class="search-input"
          @keyup.enter="generateRecommendation"
        />
        <button
          @click="generateRecommendation"
          :disabled="isGenerating || !peptideSearch.trim()"
          class="generate-btn"
        >
          {{ isGenerating ? '🔍 Analyzing...' : '✨ Generate Recommendation' }}
        </button>
      </div>

      <!-- Quick Select from Inventory -->
      <div v-if="inventoryPeptides.length > 0" class="quick-select">
        <p class="hint">Or select from your inventory:</p>
        <div class="peptide-chips">
          <button
            v-for="peptide in inventoryPeptides"
            :key="peptide"
            @click="selectPeptide(peptide)"
            class="peptide-chip"
          >
            {{ peptide }}
          </button>
        </div>
      </div>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-message">
      ⚠️ {{ error }}
    </div>

    <!-- Recommendation Result -->
    <div v-if="recommendation" class="recommendation-section">
      <div class="recommendation-card panel">
        <div class="card-header">
          <h3>📋 {{ recommendation.peptide_name }} Protocol</h3>
          <span :class="['confidence-badge', recommendation.confidence_level]">
            {{ recommendation.confidence_level.toUpperCase() }} Confidence
          </span>
        </div>

        <!-- Reconstitution Instructions -->
        <div class="section">
          <h4>💧 Reconstitution</h4>
          <div class="recon-details">
            <div class="detail-row">
              <span class="label">Peptide Amount:</span>
              <span class="value">{{ recommendation.reconstitution.peptide_amount_mg }}mg</span>
            </div>
            <div class="detail-row">
              <span class="label">Bacteriostatic Water:</span>
              <span class="value">{{ recommendation.reconstitution.bac_water_ml }}mL</span>
            </div>
            <div class="detail-row emphasis">
              <span class="label">Final Concentration:</span>
              <span class="value">{{ recommendation.reconstitution.final_concentration }}</span>
            </div>
          </div>
        </div>

        <!-- Dosage Recommendations -->
        <div class="section">
          <h4>💉 Dosage</h4>
          <div class="dosage-details">
            <div class="detail-row">
              <span class="label">Range:</span>
              <span class="value">
                {{ recommendation.dosage.range_min_mcg }}-{{ recommendation.dosage.range_max_mcg }}mcg
              </span>
            </div>
            <div class="detail-row">
              <span class="label">Frequency:</span>
              <span class="value">{{ recommendation.dosage.frequency }}</span>
            </div>
            <div class="detail-row">
              <span class="label">Duration:</span>
              <span class="value">{{ recommendation.dosage.duration }}</span>
            </div>
          </div>
        </div>

        <!-- Safety Notes -->
        <div v-if="recommendation.safety_notes.length > 0" class="section safety-section">
          <h4>⚠️ Safety & Precautions</h4>
          <ul class="safety-list">
            <li v-for="(note, idx) in recommendation.safety_notes" :key="idx">
              {{ note }}
            </li>
          </ul>
        </div>

        <!-- Sources -->
        <div v-if="recommendation.sources.length > 0" class="section sources-section">
          <h4>📚 Based on {{ recommendation.sources.length }} Research Paper(s)</h4>
          <div class="sources-list">
            <div v-for="(source, idx) in recommendation.sources" :key="idx" class="source-item">
              <strong>{{ source.title }}</strong>
              <button @click="openLink(source.url ?? undefined)" class="source-link">View →</button>
            </div>
          </div>
        </div>

        <!-- Actions -->
        <div class="card-actions">
          <button @click="saveAsProtocol" class="save-btn" :disabled="isSaving">
            {{ isSaving ? '💾 Saving...' : '💾 Save as Protocol' }}
          </button>
          <button @click="exportRecommendation" class="export-btn">
            📄 Export PDF
          </button>
          <button @click="shareRecommendation" class="share-btn">
            🔗 Copy Link
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';
import {
  listProtocols,
  searchLiterature,
  summarizeContent,
  openExternalLink,
  type LiteratureEntry,
} from '../api/peptrack';

interface ReconstitutionInfo {
  peptide_amount_mg: number;
  bac_water_ml: number;
  final_concentration: string;
}

interface DosageInfo {
  range_min_mcg: number;
  range_max_mcg: number;
  frequency: string;
  duration: string;
}

interface ProtocolRecommendation {
  peptide_name: string;
  reconstitution: ReconstitutionInfo;
  dosage: DosageInfo;
  safety_notes: string[];
  sources: LiteratureEntry[];
  confidence_level: 'high' | 'medium' | 'low';
}

const peptideSearch = ref('');
const isGenerating = ref(false);
const isSaving = ref(false);
const error = ref<string | null>(null);
const recommendation = ref<ProtocolRecommendation | null>(null);
const inventoryPeptides = ref<string[]>([]);

onMounted(async () => {
  await loadInventoryPeptides();
});

async function loadInventoryPeptides() {
  try {
    const protocols = await listProtocols();
    const peptides = new Set(protocols.map(p => p.name));
    inventoryPeptides.value = Array.from(peptides).slice(0, 5);
  } catch (err) {
    // Silently fail, not critical
  }
}

function selectPeptide(peptide: string) {
  peptideSearch.value = peptide;
  generateRecommendation();
}

async function generateRecommendation() {
  if (!peptideSearch.value.trim()) return;

  isGenerating.value = true;
  error.value = null;
  recommendation.value = null;

  try {
    // Step 1: Search literature
    const literatureResults = await searchLiterature({
      query: `${peptideSearch.value} dosage reconstitution clinical`,
      maxResults: 5,
      sources: ['pubmed', 'openalex'],
    });

    // Combine all abstracts
    const abstracts = literatureResults
      .flatMap(result => result.results)
      .map(paper => `Title: ${paper.title}\nAbstract: ${paper.abstract_text ?? 'N/A'}`)
      .join('\n\n---\n\n');

    // Step 2: Generate AI recommendation
    const prompt = `Based on the following research papers about ${peptideSearch.value}, provide a protocol recommendation.

Research Papers:
${abstracts}

Please provide a structured recommendation in the following format:

RECONSTITUTION:
- Peptide Amount (mg): [typical vial size]
- Bacteriostatic Water (mL): [recommended amount]
- Final Concentration: [mg/mL]

DOSAGE:
- Range: [min]-[max] mcg
- Frequency: [daily/2x weekly/etc]
- Duration: [typical cycle length]

SAFETY NOTES:
- [List 3-5 important safety considerations, contraindications, or side effects]

CONFIDENCE: [HIGH/MEDIUM/LOW based on quality and quantity of research]

Be specific with numbers. If research is limited, note this in confidence level.`;

    const aiResponse = await summarizeContent({
      title: `Protocol Recommendation: ${peptideSearch.value}`,
      content: prompt,
      format: 'Markdown',
    });

    // Step 3: Parse AI response
    const parsed = parseAIRecommendation(aiResponse.output, peptideSearch.value);

    // Add sources
    parsed.sources = literatureResults
      .flatMap(result => result.results)
      .slice(0, 3)
      .map(paper => ({
        id: Math.random().toString(),
        title: paper.title,
        url: paper.url ?? '',
        source: 'pubmed',
        summary: paper.abstract_text ?? '',
        indexed_at: new Date().toISOString(),
      }));

    recommendation.value = parsed;
    showSuccessToast('Success', 'Protocol recommendation generated!');
  } catch (err) {
    error.value = `Failed to generate recommendation: ${String(err)}`;
    showErrorToast(new Error(String(err)));
  } finally {
    isGenerating.value = false;
  }
}

function parseAIRecommendation(text: string, peptideName: string): ProtocolRecommendation {
  // Extract reconstitution
  const peptideAmountMatch = text.match(/Peptide Amount.*?(\d+(?:\.\d+)?)\s*mg/i);
  const bacWaterMatch = text.match(/Bacteriostatic Water.*?(\d+(?:\.\d+)?)\s*mL/i);
  const concMatch = text.match(/Final Concentration:\s*(.+?)(?:\n|$)/i);

  // Extract dosage
  const rangeMatch = text.match(/Range:\s*(\d+(?:\.\d+)?)\s*-\s*(\d+(?:\.\d+)?)\s*mcg/i);
  const freqMatch = text.match(/Frequency:\s*(.+?)(?:\n|$)/i);
  const durationMatch = text.match(/Duration:\s*(.+?)(?:\n|$)/i);

  // Extract safety notes
  const safetySection = text.match(/SAFETY NOTES?:(.+?)(?=\n\n|CONFIDENCE:|$)/is);
  const safetyNotes = safetySection?.[1]
    ? safetySection[1]
        .split('\n')
        .map(line => line.trim())
        .filter(line => line.startsWith('-') || line.startsWith('•'))
        .map(line => line.replace(/^[-•]\s*/, ''))
        .filter(line => line.length > 0)
    : [];

  // Extract confidence
  const confMatch = text.match(/CONFIDENCE:\s*(HIGH|MEDIUM|LOW)/i);
  const confidence = confMatch?.[1] ? confMatch[1].toLowerCase() as 'high' | 'medium' | 'low' : 'medium';

  return {
    peptide_name: peptideName,
    reconstitution: {
      peptide_amount_mg: peptideAmountMatch?.[1] ? parseFloat(peptideAmountMatch[1]) : 5,
      bac_water_ml: bacWaterMatch?.[1] ? parseFloat(bacWaterMatch[1]) : 2,
      final_concentration: concMatch?.[1]?.trim() ?? '2.5mg/mL',
    },
    dosage: {
      range_min_mcg: rangeMatch?.[1] ? parseFloat(rangeMatch[1]) : 100,
      range_max_mcg: rangeMatch?.[2] ? parseFloat(rangeMatch[2]) : 500,
      frequency: freqMatch?.[1]?.trim() ?? 'Daily',
      duration: durationMatch?.[1]?.trim() ?? '8-12 weeks',
    },
    safety_notes: safetyNotes.length > 0 ? safetyNotes : ['Consult healthcare provider before use'],
    sources: [],
    confidence_level: confidence,
  };
}

async function saveAsProtocol() {
  if (!recommendation.value) return;

  isSaving.value = true;
  try {
    // TODO: Implement createProtocol API call
    // For now, just copy to clipboard
    const protocolText = `Protocol: ${recommendation.value.peptide_name}

Reconstitution:
- ${recommendation.value.reconstitution.peptide_amount_mg}mg in ${recommendation.value.reconstitution.bac_water_ml}mL BAC water
- Final: ${recommendation.value.reconstitution.final_concentration}

Dosage:
- ${recommendation.value.dosage.range_min_mcg}-${recommendation.value.dosage.range_max_mcg}mcg
- ${recommendation.value.dosage.frequency}
- Duration: ${recommendation.value.dosage.duration}

Safety Notes:
${recommendation.value.safety_notes.map(note => `- ${note}`).join('\n')}`;

    await navigator.clipboard.writeText(protocolText);
    showSuccessToast('Success', 'Protocol copied to clipboard! You can paste it into the Protocols section.');
  } catch (err) {
    showErrorToast(new Error('Failed to save protocol'));
  } finally {
    isSaving.value = false;
  }
}

function exportRecommendation() {
  if (!recommendation.value) return;

  const content = `PROTOCOL RECOMMENDATION: ${recommendation.value.peptide_name}

RECONSTITUTION:
Peptide Amount: ${recommendation.value.reconstitution.peptide_amount_mg}mg
Bacteriostatic Water: ${recommendation.value.reconstitution.bac_water_ml}mL
Final Concentration: ${recommendation.value.reconstitution.final_concentration}

DOSAGE:
Range: ${recommendation.value.dosage.range_min_mcg}-${recommendation.value.dosage.range_max_mcg}mcg
Frequency: ${recommendation.value.dosage.frequency}
Duration: ${recommendation.value.dosage.duration}

SAFETY & PRECAUTIONS:
${recommendation.value.safety_notes.map(note => `• ${note}`).join('\n')}

SOURCES:
${recommendation.value.sources.map(s => `• ${s.title}`).join('\n')}

Confidence Level: ${recommendation.value.confidence_level.toUpperCase()}

Generated by PepTrack AI Protocol Assistant
Date: ${new Date().toLocaleDateString()}`;

  const blob = new Blob([content], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${recommendation.value.peptide_name}_protocol.txt`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);

  showSuccessToast('Exported', 'Protocol downloaded as text file');
}

async function shareRecommendation() {
  if (!recommendation.value) return;

  const shareText = `${recommendation.value.peptide_name} Protocol:\n${recommendation.value.reconstitution.peptide_amount_mg}mg in ${recommendation.value.reconstitution.bac_water_ml}mL = ${recommendation.value.reconstitution.final_concentration}\nDosage: ${recommendation.value.dosage.range_min_mcg}-${recommendation.value.dosage.range_max_mcg}mcg ${recommendation.value.dosage.frequency}`;

  try {
    await navigator.clipboard.writeText(shareText);
    showSuccessToast('Copied', 'Protocol summary copied to clipboard');
  } catch (err) {
    showErrorToast(new Error('Failed to copy'));
  }
}

async function openLink(url?: string) {
  if (!url) return;
  try {
    await openExternalLink(url);
  } catch {
    window.open(url, '_blank', 'noopener,noreferrer');
  }
}
</script>

<style scoped>
.protocol-recommender {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

h2 {
  margin-bottom: 8px;
  color: #2c3e50;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin-bottom: 24px;
}

.panel {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin-bottom: 24px;
}

.search-section h3 {
  margin-top: 0;
  margin-bottom: 16px;
  color: #2c3e50;
  font-size: 18px;
}

.peptide-selector {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.search-input {
  flex: 1;
  padding: 12px 16px;
  border: 2px solid #ddd;
  border-radius: 8px;
  font-size: 15px;
  transition: border-color 0.2s;
}

.search-input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.generate-btn {
  padding: 12px 24px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 15px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.generate-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.generate-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.quick-select {
  padding-top: 16px;
  border-top: 1px solid #eee;
}

.hint {
  font-size: 13px;
  color: #666;
  margin-bottom: 12px;
}

.peptide-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.peptide-chip {
  padding: 8px 16px;
  background-color: #f0f0f0;
  border: 1px solid #ddd;
  border-radius: 20px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.peptide-chip:hover {
  background-color: #667eea;
  color: white;
  border-color: #667eea;
}

.error-message {
  padding: 16px;
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 8px;
  color: #c33;
  margin-bottom: 24px;
}

.recommendation-card {
  animation: slideIn 0.3s ease-out;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 16px;
  border-bottom: 2px solid #eee;
}

.card-header h3 {
  margin: 0;
  color: #2c3e50;
  font-size: 24px;
}

.confidence-badge {
  padding: 6px 12px;
  border-radius: 6px;
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
}

.confidence-badge.high {
  background-color: #d4edda;
  color: #155724;
}

.confidence-badge.medium {
  background-color: #fff3cd;
  color: #856404;
}

.confidence-badge.low {
  background-color: #f8d7da;
  color: #721c24;
}

.section {
  margin-bottom: 24px;
}

.section h4 {
  margin-top: 0;
  margin-bottom: 12px;
  color: #2c3e50;
  font-size: 18px;
}

.recon-details,
.dosage-details {
  background-color: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #e0e0e0;
}

.detail-row:last-child {
  border-bottom: none;
}

.detail-row.emphasis {
  background-color: #667eea;
  color: white;
  padding: 12px;
  margin: 8px -16px -16px -16px;
  border-radius: 0 0 8px 8px;
  border-bottom: none;
}

.detail-row.emphasis .label,
.detail-row.emphasis .value {
  font-weight: 700;
  font-size: 16px;
}

.label {
  font-weight: 600;
  color: #666;
}

.value {
  font-weight: 700;
  color: #2c3e50;
}

.safety-section {
  background-color: #fff3cd;
  border-left: 4px solid #f39c12;
  padding: 16px;
  border-radius: 8px;
}

.safety-list {
  margin: 0;
  padding-left: 20px;
}

.safety-list li {
  margin-bottom: 8px;
  color: #856404;
  line-height: 1.5;
}

.sources-section {
  background-color: #e3f2fd;
  border-left: 4px solid #3498db;
  padding: 16px;
  border-radius: 8px;
}

.sources-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.source-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background-color: white;
  border-radius: 6px;
  font-size: 14px;
}

.source-link {
  padding: 6px 12px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.source-link:hover {
  background-color: #2980b9;
}

.card-actions {
  display: flex;
  gap: 12px;
  padding-top: 24px;
  border-top: 2px solid #eee;
}

.save-btn,
.export-btn,
.share-btn {
  flex: 1;
  padding: 14px;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 15px;
  cursor: pointer;
  transition: all 0.2s;
}

.save-btn {
  background: linear-gradient(135deg, #27ae60, #229954);
  color: white;
}

.save-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(39, 174, 96, 0.3);
}

.save-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.export-btn {
  background-color: #3498db;
  color: white;
}

.export-btn:hover {
  background-color: #2980b9;
  transform: translateY(-2px);
}

.share-btn {
  background-color: #95a5a6;
  color: white;
}

.share-btn:hover {
  background-color: #7f8c8d;
  transform: translateY(-2px);
}

@media (max-width: 768px) {
  .peptide-selector {
    flex-direction: column;
  }

  .card-actions {
    flex-direction: column;
  }

  .detail-row {
    flex-direction: column;
    gap: 4px;
  }
}
</style>
