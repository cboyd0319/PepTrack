# TOP 10 REALISTIC HIGH-IMPACT Implementation Plan

**Generated**: 2025-11-15
**Status**: Ready for Implementation
**Estimated Total Time**: 8-12 weeks for all 10 features

---

## Selection Criteria

The TOP 10 were selected based on:

1. **High Impact**: Dramatically improves user experience, safety, or insights
2. **Realistic Complexity**: Can be implemented with existing tech stack (2 days - 2 weeks each)
3. **No External Dependencies**: Doesn't require third-party APIs or partnerships
4. **Builds on Existing Foundation**: Leverages current architecture
5. **User Value**: Addresses real pain points or unlocks new capabilities

---

## THE TOP 10

### 🥇 #1: Bulk Operations (Idea #2)
**Category**: Quick Win - User Experience
**Impact**: ⭐⭐⭐⭐⭐ (Massive time savings)
**Complexity**: Medium (3-5 days)
**Dependencies**: None

#### Why This is #1
- **Pain Point**: Users with many dose logs or inventory items struggle with one-by-one deletion/editing
- **Time Savings**: 10x faster for managing large datasets
- **Professional Feel**: Every modern app has bulk operations
- **Foundation for Future**: Enables batch export, batch updates, etc.

#### Implementation Details
**Frontend Changes:**
- Add checkbox selection mode to tables (DoseTracker, InventoryList)
- Shift+click for range selection
- "Select All" / "Deselect All" buttons
- Bulk action toolbar (Delete, Export, Edit attributes)
- Confirmation modal showing count of affected items

**Backend Changes:**
- Batch delete commands: `delete_doses_batch(ids: Vec<String>)`
- Batch update commands: `update_doses_batch(updates: Vec<DoseUpdate>)`
- Transaction support for atomicity

**Files to Modify:**
- `frontend/src/components/DoseTracker.vue` - Add selection UI
- `frontend/src/components/SupplierManagement.vue` - Add to inventory table
- `src-tauri/src/commands/doses.rs` - Batch operations
- `src-tauri/src/commands/suppliers.rs` - Batch inventory operations
- `crates/core/src/db.rs` - Batch DB methods

**Testing:**
- Select and delete 10+ doses at once
- Shift+click range selection
- "Select All" with 100+ items
- Undo bulk deletion (if #1b Undo/Redo implemented)

---

### 🥈 #2: Tags & Labels for Protocols (Idea #9)
**Category**: Quick Win - Organization
**Impact**: ⭐⭐⭐⭐⭐ (Organization, filtering, discovery)
**Complexity**: Medium (4-6 days)
**Dependencies**: Database schema update

#### Why This is #2
- **Organization**: Users can categorize protocols (cutting, bulking, research, stack, etc.)
- **Filtering**: Find protocols by purpose instantly
- **Scalability**: As users add more protocols, tags become essential
- **Visual Appeal**: Color-coded tags look professional

#### Implementation Details
**Database Schema:**
```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL, -- hex color code
    created_at INTEGER NOT NULL
);

CREATE TABLE protocol_tags (
    protocol_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (protocol_id, tag_id),
    FOREIGN KEY (protocol_id) REFERENCES protocols(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

**Frontend Changes:**
- Tag management modal (create, edit, delete tags)
- Tag picker component (multi-select dropdown)
- Tag chips on protocol cards
- Filter protocols by tag(s)
- Tag color picker (predefined palette)

**Backend Changes:**
- CRUD commands for tags
- Assign/remove tags from protocols
- List protocols by tag filter
- Tag usage statistics

**Files to Create:**
- `frontend/src/components/TagManagement.vue` - Tag CRUD UI
- `frontend/src/components/TagPicker.vue` - Reusable tag selector
- `src-tauri/src/commands/tags.rs` - Tag IPC commands
- Migration: `crates/core/src/migrations/add_tags.sql`

**Files to Modify:**
- `frontend/src/components/ProtocolForm.vue` - Add tag picker
- `frontend/src/components/Dashboard.vue` - Show tags on protocol cards
- `frontend/src/stores/protocols.ts` - Tag filtering logic
- `crates/core/src/models.rs` - Add Tag and ProtocolTag structs
- `crates/core/src/db.rs` - Tag queries

**Testing:**
- Create tag "Cutting" with blue color
- Assign to 3 protocols
- Filter dashboard by "Cutting" tag
- Delete tag (should unassign from protocols)
- Multi-tag filtering (AND/OR logic)

---

### 🥉 #3: Smart Defaults for Dose Logging (Idea #14)
**Category**: Quick Win - Data Entry
**Impact**: ⭐⭐⭐⭐ (Faster daily workflow)
**Complexity**: Low (2-3 days)
**Dependencies**: None

#### Why This is #3
- **Daily Use**: Dose logging is the most frequent action
- **Speed**: Pre-filled forms = 50% less typing
- **Accuracy**: Reduces entry errors from copy/paste
- **Learning**: App learns your patterns

#### Implementation Details
**Frontend Changes:**
- Store last dose details per protocol in localStorage
- Auto-populate injection site, amount, time when protocol selected
- "Use last dose as template" button
- Show recent doses for reference (last 3)

**Backend Changes:**
- Query last N doses for protocol: `get_recent_doses(protocol_id, limit: 5)`
- Return typical dose amount (median/mode of last 10 doses)

**Files to Modify:**
- `frontend/src/components/DoseForm.vue` - Auto-fill logic
- `frontend/src/stores/doses.ts` - Add recent doses cache
- `src-tauri/src/commands/doses.rs` - Add `get_protocol_dose_stats` command

**Testing:**
- Log dose with site="abdomen", amount=0.25mg
- Open dose form for same protocol → should pre-fill
- Change defaults, verify new defaults stick
- Switch protocols, verify separate defaults

---

### 🏅 #4: Duplicate Dose Detection (Idea #16)
**Category**: Quick Win - Safety
**Impact**: ⭐⭐⭐⭐⭐ (Prevents double-dosing accidents)
**Complexity**: Low (1-2 days)
**Dependencies**: None

#### Why This is #4
- **Safety**: Critical safety feature - prevents accidental double logging/dosing
- **Peace of Mind**: Confirmation reduces anxiety
- **Smart UX**: Non-intrusive warning, allows override
- **Quick Implement**: Very simple logic

#### Implementation Details
**Frontend Changes:**
- Before saving dose, check last dose timestamp for protocol
- If last dose < 2 hours ago, show warning modal
- "Yes, I meant to dose again" / "Cancel" options
- Configurable time threshold in settings (default 2 hours)

**Backend Changes:**
- Add to existing `log_dose` command: check last dose time
- Return warning flag if duplicate detected
- Frontend handles warning display

**Files to Modify:**
- `frontend/src/components/DoseForm.vue` - Duplicate check before submit
- `frontend/src/stores/doses.ts` - Add duplicate detection logic
- `src-tauri/src/commands/doses.rs` - Return last dose timestamp in response

**Testing:**
- Log dose at 10:00 AM
- Try to log again at 10:30 AM → warning should appear
- Click "Cancel" → dose not saved
- Click "Yes, continue" → dose saved
- Log dose at 2:01 PM → no warning (>2 hours)

---

### 🏅 #5: Body Metrics Tracking (Idea #17)
**Category**: Strategic - Advanced Tracking
**Impact**: ⭐⭐⭐⭐⭐ (Holistic health tracking, correlations)
**Complexity**: Medium (4-6 days)
**Dependencies**: New table, charting

#### Why This is #5
- **Holistic View**: Track weight, body fat, blood pressure alongside protocols
- **Correlations**: See how peptides affect body composition
- **Goal Tracking**: Monitor progress toward fitness goals
- **Foundation**: Enables future correlation analysis (#20)

#### Implementation Details
**Database Schema:**
```sql
CREATE TABLE body_metrics (
    id TEXT PRIMARY KEY,
    metric_type TEXT NOT NULL, -- 'weight', 'body_fat_pct', 'blood_pressure_systolic', etc.
    value REAL NOT NULL,
    unit TEXT NOT NULL, -- 'lbs', 'kg', '%', 'mmHg'
    notes TEXT,
    measured_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_body_metrics_measured_at ON body_metrics(measured_at);
CREATE INDEX idx_body_metrics_type ON body_metrics(metric_type);
```

**Metric Types:**
- Weight (lbs/kg)
- Body Fat % (%)
- Lean Mass (lbs/kg)
- Blood Pressure (systolic/diastolic mmHg)
- Resting Heart Rate (bpm)
- Waist Circumference (inches/cm)
- Custom (user-defined)

**Frontend Changes:**
- Body Metrics component with chart + data entry
- Line charts showing trends over time
- Overlay dose logs on metric charts (see correlation visually)
- Quick log modal (floating action button)
- Metric goal setting (target weight, etc.)

**Backend Changes:**
- CRUD commands for body metrics
- Query metrics by type and date range
- Calculate trends (7-day average, rate of change)

**Files to Create:**
- `frontend/src/components/BodyMetrics.vue` - Main component
- `frontend/src/components/MetricChart.vue` - Reusable chart
- `frontend/src/components/QuickMetricLog.vue` - Quick entry modal
- `frontend/src/stores/bodyMetrics.ts` - Pinia store
- `src-tauri/src/commands/body_metrics.rs` - IPC commands
- `crates/core/src/migrations/add_body_metrics.sql` - Migration

**Files to Modify:**
- `frontend/src/App.vue` - Add route for body metrics
- `crates/core/src/models.rs` - Add BodyMetric struct
- `crates/core/src/db.rs` - Body metrics queries

**Testing:**
- Log weight: 180 lbs on 2025-01-01
- Log weight: 175 lbs on 2025-01-15
- View weight chart → should show downward trend
- Overlay dose logs → see correlation with protocol start
- Set goal weight: 170 lbs → show progress bar

---

### 🏅 #6: Side Effect Tracker (Idea #18)
**Category**: Strategic - Safety & Insights
**Impact**: ⭐⭐⭐⭐⭐ (Safety monitoring, protocol adjustment)
**Complexity**: Medium (5-7 days)
**Dependencies**: New table, symptom library

#### Why This is #6
- **Safety**: Critical for monitoring adverse reactions
- **Protocol Optimization**: Identify patterns (e.g., headaches always on day 3)
- **Decision Support**: Data-driven decisions to continue/stop protocols
- **Medical Value**: Shareable with doctors

#### Implementation Details
**Database Schema:**
```sql
CREATE TABLE side_effects (
    id TEXT PRIMARY KEY,
    protocol_id TEXT NOT NULL,
    symptom TEXT NOT NULL, -- 'headache', 'nausea', 'insomnia', custom
    severity INTEGER NOT NULL CHECK(severity >= 1 AND severity <= 10),
    duration_minutes INTEGER, -- how long it lasted
    notes TEXT,
    occurred_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (protocol_id) REFERENCES protocols(id) ON DELETE CASCADE
);

CREATE INDEX idx_side_effects_occurred_at ON side_effects(occurred_at);
CREATE INDEX idx_side_effects_protocol ON side_effects(protocol_id);
```

**Symptom Library (Pre-populated):**
- Headache
- Nausea
- Insomnia
- Fatigue
- Injection site reaction (redness, swelling)
- Dizziness
- Heart palpitations
- Water retention
- Custom (user text input)

**Frontend Changes:**
- Side Effect logging component
- Symptom picker (autocomplete with common symptoms)
- Severity slider (1-10)
- Duration picker (minutes)
- Timeline view of side effects
- Heatmap showing frequency by time/day
- Filter by symptom type

**Backend Changes:**
- CRUD commands for side effects
- Correlate side effects with doses (time proximity)
- Aggregate statistics (most common symptoms, average severity)

**Files to Create:**
- `frontend/src/components/SideEffectTracker.vue` - Main component
- `frontend/src/components/SideEffectForm.vue` - Logging form
- `frontend/src/components/SideEffectTimeline.vue` - Visual timeline
- `frontend/src/stores/sideEffects.ts` - Pinia store
- `src-tauri/src/commands/side_effects.rs` - IPC commands
- `crates/core/src/migrations/add_side_effects.sql` - Migration

**Files to Modify:**
- `frontend/src/App.vue` - Add route
- `crates/core/src/models.rs` - Add SideEffect struct
- `crates/core/src/db.rs` - Side effect queries

**Testing:**
- Log headache (severity 6) 2 hours after BPC-157 dose
- Log nausea (severity 3) next day
- View timeline → see both side effects
- Filter by "headache" → only show headache entries
- Check correlation → "Headaches often occur 2 hours after dosing"

---

### 🏅 #7: Automatic Reconstitution Calculator (Idea #84)
**Category**: Quick Win - Utility
**Impact**: ⭐⭐⭐⭐ (Eliminates math errors, education)
**Complexity**: Low (2-3 days)
**Dependencies**: None

#### Why This is #7
- **Safety**: Math errors can lead to wrong doses
- **Education**: Shows the math, teaches users
- **Convenience**: No more manual calculations
- **Onboarding**: Helps beginners feel confident

#### Implementation Details
**Calculator Features:**
- Input: Vial size (mg), Bacteriostatic Water (ml), Target dose (mg or mcg)
- Output: How many units on syringe, concentration (mg/ml)
- Presets for common ratios (5mg → 2ml, 10mg → 2ml)
- Reverse calculator: "I want 0.25mg per injection, how much water?"
- Unit conversion (mcg ↔ mg ↔ IU for insulin syringes)

**Formula:**
```
Concentration (mg/ml) = Vial Size (mg) / Water Volume (ml)
Dose Volume (ml) = Target Dose (mg) / Concentration (mg/ml)
Syringe Units (for insulin syringe) = Dose Volume (ml) * 100
```

**Frontend Changes:**
- Standalone calculator component (accessible from menu)
- Embedded calculator in protocol form
- Step-by-step walkthrough mode
- Save calculations to protocol notes
- Common presets (BPC-157 5mg → 2ml, etc.)

**Files to Create:**
- `frontend/src/components/ReconstitutionCalculator.vue` - Main calculator
- `frontend/src/components/CalculatorPresets.vue` - Preset templates
- `frontend/src/utils/dosageCalculations.ts` - Calculation logic

**Files to Modify:**
- `frontend/src/components/ProtocolForm.vue` - Embed calculator button
- `frontend/src/App.vue` - Add calculator to command palette

**Testing:**
- Calculate: 5mg vial + 2ml water = 2.5mg/ml
- Target dose: 0.25mg = 0.1ml = 10 units
- Reverse: Want 0.5mg per 10 units → need 1ml water
- Test edge cases: 0mg vial, negative values → show validation errors

---

### 🏅 #8: Favorites/Pinning for Protocols (Idea #8)
**Category**: Quick Win - UX
**Impact**: ⭐⭐⭐⭐ (Faster access, personalization)
**Complexity**: Low (1-2 days)
**Dependencies**: Database column addition

#### Why This is #8
- **Speed**: Active protocols always at top
- **Focus**: Reduce clutter, highlight what matters
- **Simple**: Very easy to implement, high user satisfaction

#### Implementation Details
**Database Schema:**
```sql
ALTER TABLE protocols ADD COLUMN is_favorite INTEGER DEFAULT 0;
CREATE INDEX idx_protocols_favorite ON protocols(is_favorite);
```

**Frontend Changes:**
- Star icon on protocol cards (filled = favorited)
- Click star to toggle favorite status
- Sort protocols: favorites first, then by name/date
- "Show only favorites" filter toggle
- Keyboard shortcut (F key) to favorite selected protocol

**Backend Changes:**
- Update protocol: `toggle_favorite(protocol_id)`
- List protocols: sort by `is_favorite DESC`

**Files to Modify:**
- `crates/core/src/db.rs` - Add migration, update queries
- `frontend/src/components/Dashboard.vue` - Add star icon, sort logic
- `frontend/src/components/ProtocolCard.vue` - Star toggle button
- `frontend/src/stores/protocols.ts` - Toggle favorite action
- `src-tauri/src/commands/protocols.rs` - Add toggle command

**Testing:**
- Click star on "BPC-157 Protocol" → moves to top
- Click again → unfavorite, returns to normal position
- Filter "Show only favorites" → see only starred
- Keyboard shortcut "F" → toggles favorite

---

### 🏅 #9: Export to PDF Reports (Idea #30)
**Category**: Strategic - Professional Use
**Impact**: ⭐⭐⭐⭐⭐ (Medical sharing, legitimacy)
**Complexity**: Medium (5-7 days)
**Dependencies**: PDF library

#### Why This is #9
- **Medical Integration**: Share with doctors, health coaches
- **Professional**: Looks legitimate, builds trust
- **Documentation**: Keep records for legal/medical purposes
- **Versatile**: Export single protocol or full summary

#### Implementation Details
**PDF Library:**
- Use `jsPDF` or `pdfkit` (Rust: `printpdf`)
- Generate on Rust backend for better performance

**Report Types:**
1. **Protocol Summary**: Single protocol with all doses, costs, notes
2. **Full Health Report**: All protocols, body metrics, side effects
3. **Date Range Report**: Activity within specific timeframe
4. **Doctor-Friendly Format**: Medical terminology, professional layout

**Report Sections:**
- Header: Patient info (name, date of birth - optional), date generated
- Protocol Details: Peptide name, dosage, frequency, duration
- Dose History: Table of all logged doses
- Body Metrics: Charts showing weight, BP trends
- Side Effects: List of reported symptoms
- Cost Analysis: Total spent, cost per day
- Notes: User's research notes
- Footer: Disclaimer ("This is a self-tracking record, not medical advice")

**Frontend Changes:**
- "Export to PDF" button on protocol cards
- PDF export modal (select report type, date range, sections)
- Preview before export
- Download or email PDF

**Backend Changes:**
- Aggregate all data for report
- Generate PDF with tables, charts (as images)
- Return PDF as bytes → download

**Files to Create:**
- `src-tauri/src/commands/pdf_export.rs` - PDF generation
- `frontend/src/components/PdfExportModal.vue` - Export options UI
- `crates/core/src/pdf/` - PDF templating module
  - `crates/core/src/pdf/protocol_report.rs`
  - `crates/core/src/pdf/health_report.rs`

**Dependencies to Add:**
```toml
# Cargo.toml
printpdf = "0.7"
image = "0.25" # for embedding charts
```

**Files to Modify:**
- `frontend/src/components/ProtocolCard.vue` - Add export button
- `frontend/src/components/Dashboard.vue` - Add "Export All" button

**Testing:**
- Export single protocol → verify all doses included
- Export with date range: Jan 1-31 → only January data
- Open PDF → verify formatting, charts readable
- Test with empty data → graceful handling

---

### 🏅 #10: Predictive Inventory Alerts (Idea #26)
**Category**: Strategic - Automation
**Impact**: ⭐⭐⭐⭐ (Prevent running out, planning)
**Complexity**: Medium (4-5 days)
**Dependencies**: Historical usage data

#### Why This is #10
- **Proactive**: Never run out unexpectedly
- **Planning**: Know when to reorder in advance
- **Cost Savings**: Bulk order before running out (vs. emergency shipping)
- **Smart**: Uses actual usage patterns, not guesses

#### Implementation Details
**Algorithm:**
1. Calculate daily usage rate (last 30 days): `avg_mg_per_day`
2. Get current inventory: `total_mg_remaining`
3. Predict runout: `days_remaining = total_mg / avg_mg_per_day`
4. Alert thresholds:
   - Red (urgent): < 7 days remaining
   - Yellow (warning): 7-14 days
   - Green (good): > 14 days

**Advanced Features:**
- Account for unopened vials (don't just look at current vial)
- Seasonal patterns (e.g., higher usage in January)
- Configurable alert thresholds per user
- "Reorder now" button → link to supplier

**Frontend Changes:**
- Inventory status badges on dashboard
- Alert banner when low inventory detected
- Inventory forecast chart (projected runout date)
- "Days of supply remaining" metric

**Backend Changes:**
- `calculate_usage_rate(protocol_id, days: 30)` → avg mg/day
- `predict_inventory_depletion(protocol_id)` → days remaining
- `get_low_inventory_alerts()` → list of protocols needing reorder

**Files to Create:**
- `src-tauri/src/commands/inventory_analytics.rs` - Prediction logic
- `frontend/src/components/InventoryAlerts.vue` - Alert UI
- `frontend/src/components/InventoryForecast.vue` - Forecast chart

**Files to Modify:**
- `frontend/src/components/Dashboard.vue` - Show alerts
- `frontend/src/components/SupplierManagement.vue` - Show days remaining
- `frontend/src/stores/suppliers.ts` - Add alert state
- `crates/core/src/db.rs` - Usage rate queries

**Testing:**
- 10mg inventory, 1mg/day usage → "10 days remaining" (yellow alert)
- 5mg inventory, 1mg/day → "5 days remaining" (red alert)
- No usage last 30 days → "N/A" or "Cannot predict"
- Multiple vials: 1 in-use (2mg) + 1 unopened (10mg) = 12mg total

---

## Implementation Timeline

### Phase 1: Quick Wins (Weeks 1-2)
- **Week 1:**
  - #4: Duplicate Dose Detection (1-2 days)
  - #3: Smart Defaults (2-3 days)
  - #8: Favorites/Pinning (1-2 days)

- **Week 2:**
  - #7: Reconstitution Calculator (2-3 days)
  - #1: Bulk Operations (start)

### Phase 2: Core Features (Weeks 3-5)
- **Week 3:**
  - #1: Bulk Operations (complete, 3-5 days)
  - #2: Tags & Labels (start)

- **Week 4:**
  - #2: Tags & Labels (complete, 4-6 days)
  - #5: Body Metrics (start)

- **Week 5:**
  - #5: Body Metrics (complete, 4-6 days)

### Phase 3: Advanced Features (Weeks 6-8)
- **Week 6:**
  - #6: Side Effect Tracker (5-7 days)

- **Week 7:**
  - #10: Predictive Inventory (4-5 days)
  - #9: PDF Reports (start)

- **Week 8:**
  - #9: PDF Reports (complete, 5-7 days)
  - Polish, bug fixes, testing

---

## Success Metrics

### Quantitative
- **Bulk Operations**: 90% reduction in time to delete 10+ items
- **Tags**: 80% of active users create at least one tag
- **Smart Defaults**: 50% reduction in form completion time
- **Duplicate Detection**: Zero accidental double-dose reports
- **Body Metrics**: 60% of users log at least one metric
- **Side Effects**: 40% of users log at least one side effect
- **Calculator**: 70% of new users use calculator at least once
- **Favorites**: 85% of users favorite at least one protocol
- **PDF Export**: 30% of users export a PDF within first month
- **Inventory Alerts**: 90% accuracy in predicting runout dates

### Qualitative
- User feedback: "This feels like a professional app now"
- Reduced support questions about dosing math
- Increased confidence in protocol tracking
- Better data-driven decisions

---

## Risk Mitigation

### Technical Risks
- **Database Migrations**: Test migrations on backup data first
- **Performance**: Bulk operations with 1000+ items may be slow → pagination
- **PDF Size**: Large reports may exceed memory → stream generation
- **Data Loss**: Always backup before destructive operations

### UX Risks
- **Feature Overload**: Don't show all features at once → progressive disclosure
- **Learning Curve**: Provide tooltips, tutorials for complex features
- **Mobile**: Ensure responsive design for all new components

---

## Post-Implementation

After TOP 10 completion:
1. **User Testing**: Beta test with 5-10 power users
2. **Performance Profiling**: Ensure no regressions
3. **Documentation**: Update user guides, changelog
4. **Video Demos**: Screen recordings of each new feature
5. **Roadmap Review**: Prioritize next 10 from remaining 134 ideas

---

## Conclusion

These TOP 10 features represent **high-impact, realistic improvements** that will transform PepTrack from a solid tracking app into a **comprehensive health optimization platform**. Each feature builds on the existing architecture, requires no external dependencies, and delivers immediate user value.

**Total Estimated Time**: 8-12 weeks
**Total New Code**: ~15,000-20,000 lines
**Database Tables Added**: 4 (tags, protocol_tags, body_metrics, side_effects)
**New Components**: ~20 Vue components
**New Rust Commands**: ~40 IPC commands

Let's build this! 🚀

---

**Next Step**: Begin implementation of #4 (Duplicate Dose Detection) as the quickest win to build momentum.
