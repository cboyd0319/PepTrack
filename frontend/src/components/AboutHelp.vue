<template>
  <div class="about-help">
    <h2>ℹ️ About & Help</h2>
    <p class="description">Learn about PepTrack and get help</p>

    <div class="about-content">
      <!-- App Info -->
      <div class="section">
        <h3>📱 Application Information</h3>
        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Version:</span>
            <span class="info-value">0.1.0</span>
          </div>
          <div class="info-item">
            <span class="info-label">Platform:</span>
            <span class="info-value">Tauri Desktop App</span>
          </div>
          <div class="info-item">
            <span class="info-label">License:</span>
            <span class="info-value">Open Source</span>
          </div>
        </div>
      </div>

      <!-- Getting Started -->
      <div class="section">
        <h3>🚀 Getting Started</h3>
        <div class="help-steps">
          <div class="help-step">
            <div class="step-number">1</div>
            <div class="step-content">
              <h4>💊 Create a Protocol</h4>
              <p>Go to the <strong>Protocols</strong> tab and create your first peptide protocol. Add details like the peptide name, target concentration, and any notes.</p>
            </div>
          </div>

          <div class="help-step">
            <div class="step-number">2</div>
            <div class="step-content">
              <h4>💉 Log Your Doses</h4>
              <p>Visit the <strong>Doses</strong> tab to record each dose. Track the injection site, amount, and add any observations.</p>
            </div>
          </div>

          <div class="help-step">
            <div class="step-number">3</div>
            <div class="step-content">
              <h4>📚 Research Literature</h4>
              <p>Use the <strong>Research</strong> tab to search PubMed and OpenAlex for scientific papers about peptides. Get AI summaries of complex research.</p>
            </div>
          </div>

          <div class="help-step">
            <div class="step-number">4</div>
            <div class="step-content">
              <h4>🏢 Manage Inventory</h4>
              <p>Track your suppliers and vial inventory in the <strong>Operations</strong> tab. Monitor expiration dates and stock levels.</p>
            </div>
          </div>

          <div class="help-step">
            <div class="step-number">5</div>
            <div class="step-content">
              <h4>💾 Set Up Backups</h4>
              <p>Configure automatic backups in <strong>Settings</strong>. Connect to Google Drive for cloud sync or use local file backups.</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Features Overview -->
      <div class="section">
        <h3>✨ Key Features</h3>
        <div class="features-grid">
          <div class="feature-card">
            <div class="feature-icon">🧪</div>
            <div class="feature-name">Protocol Management</div>
            <div class="feature-desc">Organize and track multiple peptide protocols</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">💉</div>
            <div class="feature-name">Dose Tracking</div>
            <div class="feature-desc">Log doses with site rotation and timing</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">📚</div>
            <div class="feature-name">Literature Search</div>
            <div class="feature-desc">Search PubMed & OpenAlex databases</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">🤖</div>
            <div class="feature-name">AI Summaries</div>
            <div class="feature-desc">Get plain-language summaries of research</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">📦</div>
            <div class="feature-name">Inventory Management</div>
            <div class="feature-desc">Track vials, suppliers, and expiration dates</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">☁️</div>
            <div class="feature-name">Cloud Backup</div>
            <div class="feature-desc">Auto-sync to Google Drive for safety</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">🔒</div>
            <div class="feature-name">Privacy First</div>
            <div class="feature-desc">All data stays on your device, encrypted</div>
          </div>

          <div class="feature-card">
            <div class="feature-icon">📊</div>
            <div class="feature-name">Dashboard</div>
            <div class="feature-desc">Quick overview of protocols and doses</div>
          </div>
        </div>
      </div>

      <!-- Privacy & Security -->
      <div class="section">
        <h3>🔒 Privacy & Security</h3>
        <div class="privacy-info">
          <div class="privacy-point">
            <span class="check">✅</span>
            <div>
              <strong>Local-First:</strong> All your data is stored locally on your device in an encrypted SQLite database.
            </div>
          </div>
          <div class="privacy-point">
            <span class="check">✅</span>
            <div>
              <strong>No Tracking:</strong> We don't track your usage, collect analytics, or send data to any servers.
            </div>
          </div>
          <div class="privacy-point">
            <span class="check">✅</span>
            <div>
              <strong>No Account Required:</strong> Use PepTrack without creating an account or providing personal information.
            </div>
          </div>
          <div class="privacy-point">
            <span class="check">✅</span>
            <div>
              <strong>Optional Cloud Sync:</strong> Cloud backup is completely optional. You control where your data goes.
            </div>
          </div>
        </div>
      </div>

      <!-- AI Setup (Optional) -->
      <div class="section">
        <h3>⚙️ AI Summary Setup (Optional)</h3>
        <p class="section-note">The AI Summary feature is completely optional. If you want to use it, you'll need to install either:</p>
        <div class="ai-options">
          <div class="ai-option">
            <div class="ai-option-header">
              <strong>Codex CLI</strong>
              <span class="badge">Uses GPT-5</span>
            </div>
            <p>OpenAI's Codex CLI provides high-quality summaries</p>
            <a href="https://developers.openai.com/codex/cli" target="_blank" class="external-link">
              Installation Guide →
            </a>
          </div>
          <div class="ai-option">
            <div class="ai-option-header">
              <strong>Claude CLI</strong>
              <span class="badge">Uses Haiku 4.5</span>
            </div>
            <p>Anthropic's Claude CLI for fast, accurate summaries</p>
            <a href="https://code.claude.com/docs/en/cli-reference" target="_blank" class="external-link">
              Installation Guide →
            </a>
          </div>
        </div>
        <p class="note">💡 Everything else in PepTrack works perfectly without AI!</p>
      </div>

      <!-- Support -->
      <div class="section">
        <h3>💬 Support & Feedback</h3>
        <div class="support-links">
          <div class="support-item">
            <div class="support-icon">📖</div>
            <div>
              <strong>Documentation</strong>
              <p>Read the full documentation and guides</p>
              <a href="https://github.com/cboyd0319/PepTrack" target="_blank" class="external-link">
                View Docs →
              </a>
            </div>
          </div>
          <div class="support-item">
            <div class="support-icon">🐛</div>
            <div>
              <strong>Report Issues</strong>
              <p>Found a bug? Let us know on GitHub</p>
              <a href="https://github.com/cboyd0319/PepTrack/issues" target="_blank" class="external-link">
                Report Issue →
              </a>
            </div>
          </div>
          <div class="support-item">
            <div class="support-icon">💡</div>
            <div>
              <strong>Feature Requests</strong>
              <p>Suggest new features or improvements</p>
              <a href="https://github.com/cboyd0319/PepTrack/discussions" target="_blank" class="external-link">
                Share Ideas →
              </a>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// No state needed for static content
</script>

<style scoped>
.about-help {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
}

.about-help h2 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.description {
  font-size: 15px;
  color: #666;
  margin: 0 0 32px 0;
}

.about-content {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.section {
  background: white;
  padding: 24px;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
}

.section h3 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: #1a1a1a;
}

.section-note {
  font-size: 14px;
  color: #666;
  margin-bottom: 16px;
}

/* App Info */
.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
}

.info-label {
  font-weight: 600;
  color: #666;
}

.info-value {
  color: #1a1a1a;
}

/* Help Steps */
.help-steps {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.help-step {
  display: flex;
  gap: 16px;
  align-items: flex-start;
}

.step-number {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  background: #1976d2;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 16px;
}

.step-content h4 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 6px 0;
  color: #1a1a1a;
}

.step-content p {
  margin: 0;
  font-size: 14px;
  color: #666;
  line-height: 1.6;
}

/* Features Grid */
.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.feature-card {
  padding: 20px;
  background: #f5f5f5;
  border-radius: 12px;
  text-align: center;
}

.feature-icon {
  font-size: 36px;
  margin-bottom: 12px;
}

.feature-name {
  font-weight: 600;
  margin-bottom: 6px;
  color: #1a1a1a;
  font-size: 14px;
}

.feature-desc {
  font-size: 12px;
  color: #666;
  line-height: 1.4;
}

/* Privacy */
.privacy-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.privacy-point {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 16px;
  background: #f5f5f5;
  border-radius: 8px;
}

.check {
  font-size: 20px;
  flex-shrink: 0;
}

.privacy-point strong {
  color: #1a1a1a;
}

/* AI Options */
.ai-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 16px;
}

.ai-option {
  padding: 16px;
  background: #f5f5f5;
  border-radius: 8px;
  border-left: 4px solid #1976d2;
}

.ai-option-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.badge {
  font-size: 11px;
  padding: 4px 8px;
  background: #1976d2;
  color: white;
  border-radius: 4px;
  font-weight: 600;
}

.ai-option p {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #666;
}

/* Support Links */
.support-links {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.support-item {
  display: flex;
  gap: 16px;
  align-items: flex-start;
  padding: 16px;
  background: #f5f5f5;
  border-radius: 8px;
}

.support-icon {
  font-size: 28px;
  flex-shrink: 0;
}

.support-item strong {
  display: block;
  margin-bottom: 4px;
  color: #1a1a1a;
}

.support-item p {
  margin: 0 0 8px 0;
  font-size: 14px;
  color: #666;
}

.external-link {
  color: #1976d2;
  text-decoration: none;
  font-weight: 600;
  font-size: 14px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.external-link:hover {
  text-decoration: underline;
}

.note {
  font-size: 13px;
  color: #666;
  padding: 12px;
  background: #fff3cd;
  border-radius: 6px;
  border-left: 4px solid #ffc107;
  margin: 0;
}

@media (prefers-color-scheme: dark) {
  .about-help h2,
  .section h3,
  .step-content h4,
  .feature-name,
  .info-value,
  .privacy-point strong,
  .support-item strong {
    color: #fff;
  }

  .description,
  .section-note,
  .step-content p,
  .feature-desc,
  .ai-option p,
  .support-item p {
    color: #aaa;
  }

  .section,
  .info-item,
  .feature-card,
  .privacy-point,
  .ai-option,
  .support-item {
    background: #2a2a2a;
    border-color: #3a3a3a;
  }

  .info-label {
    color: #aaa;
  }

  .note {
    background: #3a3a00;
    border-left-color: #ffc107;
  }
}
</style>
