# Web Scraper Architecture for PepTrack

## Overview

This document outlines the architecture for implementing a web scraper system in PepTrack to automatically track peptide prices and availability across multiple suppliers over time.

## Goals

1. **Automated Price Tracking**: Monitor peptide prices across configured suppliers
2. **Availability Monitoring**: Track stock status (in stock/out of stock)
3. **Historical Data**: Maintain 3+ months of price history for trend analysis
4. **Multi-Supplier Support**: Extensible architecture for adding new suppliers
5. **Alert Generation**: Automatic alerts for price changes and stock events

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                     PepTrack Application                      │
├──────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐      ┌──────────────────┐             │
│  │   Scraper UI    │◄─────┤  Price History   │             │
│  │  Configuration  │      │   Visualization  │             │
│  └────────┬────────┘      └──────────────────┘             │
│           │                                                  │
│           ▼                                                  │
│  ┌─────────────────────────────────────────┐               │
│  │      Scraper Scheduler Service           │               │
│  │  - Manages scraping jobs                 │               │
│  │  - Configurable intervals                │               │
│  │  - Error handling & retries              │               │
│  └─────────────────┬───────────────────────┘               │
│                    │                                         │
│                    ▼                                         │
│  ┌─────────────────────────────────────────┐               │
│  │       Supplier Scraper Modules           │               │
│  │  ┌──────────┐  ┌──────────┐            │               │
│  │  │ Scraper  │  │ Scraper  │  ...       │               │
│  │  │   #1     │  │   #2     │            │               │
│  │  └──────────┘  └──────────┘            │               │
│  └─────────────────┬───────────────────────┘               │
│                    │                                         │
│                    ▼                                         │
│  ┌─────────────────────────────────────────┐               │
│  │      Data Processing Pipeline            │               │
│  │  - Parse scraped data                    │               │
│  │  - Validate & normalize                  │               │
│  │  - Compare with history                  │               │
│  │  - Generate alerts                       │               │
│  └─────────────────┬───────────────────────┘               │
│                    │                                         │
│                    ▼                                         │
│  ┌─────────────────────────────────────────┐               │
│  │        Storage Layer (SQLite)            │               │
│  │  - price_history table                   │               │
│  │  - scraper_configs table                 │               │
│  │  - scraper_logs table                    │               │
│  │  - alerts table                          │               │
│  └─────────────────────────────────────────┘               │
│                                                               │
└──────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Scraper Scheduler Service

**Location**: `src-tauri/src/services/scraper_scheduler.rs`

**Responsibilities**:
- Manage scraping job queue
- Execute scrapers on configurable intervals (e.g., daily, weekly)
- Handle errors and implement retry logic with exponential backoff
- Log scraping activity
- Trigger alert generation on significant changes

**Key Features**:
- Background task execution using `tokio::spawn`
- Configurable cron-like scheduling
- Circuit breaker pattern for failing scrapers
- Rate limiting to respect supplier websites

### 2. Supplier Scraper Modules

**Location**: `src-tauri/src/services/scrapers/`

Each supplier gets its own scraper module that implements the `PeptideScraper` trait:

```rust
#[async_trait]
pub trait PeptideScraper: Send + Sync {
    /// Unique identifier for this scraper
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Scrape peptide data from the supplier's website
    async fn scrape(&self, peptide_name: &str) -> Result<ScrapedData>;

    /// Test if the scraper is functioning
    async fn health_check(&self) -> Result<bool>;
}

pub struct ScrapedData {
    pub peptide_name: String,
    pub cost_per_mg: f32,
    pub in_stock: bool,
    pub url: String,
    pub scraped_at: OffsetDateTime,
}
```

**Implementation Strategies**:

1. **HTTP + HTML Parsing** (for most sites):
   - Use `reqwest` for HTTP requests
   - Use `scraper` crate for HTML parsing
   - CSS selectors to extract price and availability

2. **Headless Browser** (for JavaScript-heavy sites):
   - Use `headless_chrome` or `fantoccini`
   - Wait for dynamic content to load
   - Execute JavaScript if needed

3. **API Integration** (if available):
   - Direct API calls for suppliers with public APIs
   - Most reliable and efficient method

### 3. Database Schema Extensions

```sql
-- Scraper configuration
CREATE TABLE IF NOT EXISTS scraper_configs (
    id TEXT PRIMARY KEY,
    supplier_id TEXT NOT NULL REFERENCES suppliers(id) ON DELETE CASCADE,
    scraper_type TEXT NOT NULL, -- 'html', 'headless', 'api'
    enabled BOOLEAN NOT NULL DEFAULT 1,
    schedule_cron TEXT NOT NULL, -- '0 0 * * *' for daily at midnight
    url_pattern TEXT NOT NULL, -- URL template with {peptide_name} placeholder
    css_selectors TEXT, -- JSON with selectors for price, stock, etc.
    last_run_at TEXT,
    last_success_at TEXT,
    consecutive_failures INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Scraper execution logs
CREATE TABLE IF NOT EXISTS scraper_logs (
    id TEXT PRIMARY KEY,
    scraper_config_id TEXT NOT NULL REFERENCES scraper_configs(id),
    status TEXT NOT NULL, -- 'success', 'failure', 'partial'
    items_scraped INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    execution_time_ms INTEGER NOT NULL,
    executed_at TEXT NOT NULL
);

-- Scraper run queue (for scheduled jobs)
CREATE TABLE IF NOT EXISTS scraper_queue (
    id TEXT PRIMARY KEY,
    scraper_config_id TEXT NOT NULL REFERENCES scraper_configs(id),
    peptide_name TEXT,
    priority INTEGER NOT NULL DEFAULT 5, -- 1-10, higher = more urgent
    scheduled_for TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed'
    created_at TEXT NOT NULL
);
```

### 4. Data Processing Pipeline

**Location**: `src-tauri/src/services/scraper_processor.rs`

**Workflow**:
1. Receive scraped data from scraper
2. Validate data (price > 0, peptide name matches, etc.)
3. Normalize data (standardize units, clean strings)
4. Compare with latest price in history
5. If significant change (> 10%), create alert
6. Save to `price_history` table
7. Update supplier last_seen timestamp

### 5. UI Components

**Scraper Configuration Panel** (`frontend/src/components/ScraperConfig.vue`):
- Enable/disable scrapers per supplier
- Configure scraping schedule
- View scraper status and health
- Manually trigger scraping
- View scraper logs

**Price Charts** (`frontend/src/components/PriceCharts.vue`):
- Line charts showing price trends over 3+ months
- Multi-supplier comparison charts
- Stock availability timeline
- Interactive tooltips with data points

## Implementation Phases

### Phase 1: Foundation (Week 1)
- [x] Database schema for price_history, alerts (COMPLETED)
- [x] Basic scraper trait definition (DOCUMENTED)
- [ ] Scraper scheduler service skeleton
- [ ] Configuration UI for enabling/disabling scrapers

### Phase 2: Core Scrapers (Week 2)
- [ ] Implement 2-3 example scrapers for popular suppliers
- [ ] HTML parsing approach for static sites
- [ ] Error handling and retry logic
- [ ] Basic scraper testing

### Phase 3: Advanced Features (Week 3)
- [ ] Headless browser support for JavaScript sites
- [ ] Rate limiting and politeness delays
- [ ] Scraper health monitoring
- [ ] Alert generation on price changes
- [ ] Scraper logs UI

### Phase 4: Visualization & Polish (Week 4)
- [ ] Price trend charts (Chart.js or Recharts)
- [ ] Multi-supplier comparison view
- [ ] Historical data export
- [ ] Performance optimization
- [ ] Documentation and user guide

## Configuration Example

Users configure scrapers through the UI, which stores settings like:

```json
{
  "scraper_id": "supplier-abc-html",
  "supplier_id": "abc-peptides",
  "scraper_type": "html",
  "enabled": true,
  "schedule_cron": "0 2 * * *",
  "url_pattern": "https://abc-peptides.com/product/{peptide_name}",
  "css_selectors": {
    "price": ".product-price span.amount",
    "stock": ".stock-status",
    "in_stock_text": "In Stock"
  },
  "peptides_to_track": ["Tirzepatide", "Semaglutide", "BPC-157"]
}
```

## Technical Considerations

### 1. Legal & Ethical
- Respect robots.txt
- Implement rate limiting (1-5 sec between requests)
- Use appropriate User-Agent headers
- Only scrape public data
- Comply with supplier Terms of Service
- Consider API access where available

### 2. Reliability
- Implement circuit breaker for failing scrapers
- Graceful degradation if scraper breaks
- Alert admins on consecutive failures (3+)
- Fallback to manual price entry

### 3. Performance
- Async/await for concurrent scraping
- Connection pooling
- Cache HTTP responses when appropriate
- Batch database writes
- Background job processing (not blocking UI)

### 4. Maintainability
- Each scraper is isolated module
- Easy to add new suppliers
- Configuration-driven where possible
- Comprehensive logging
- Unit tests for parsers

## Security Considerations

1. **No Credentials Storage**: Scrapers should only access public data
2. **Sandboxing**: Run scrapers in isolated context
3. **Input Validation**: Sanitize all scraped data before database insertion
4. **Rate Limiting**: Prevent abuse of scraping functionality
5. **Error Handling**: Don't expose internal errors to logs that might leak info

## Dependencies

**Rust**:
- `reqwest` - HTTP client
- `scraper` - HTML parsing
- `tokio` - Async runtime
- `tokio-cron-scheduler` - Job scheduling
- `headless_chrome` - Optional, for JavaScript sites
- `serde_json` - Configuration parsing

**Frontend**:
- `chart.js` or `recharts` - Price visualizations
- Existing Tauri commands for data fetching

## Monitoring & Alerts

The scraper system generates alerts for:
- **Price Increase** (> 10%): Warning level
- **Price Decrease** (> 10%): Info level
- **Out of Stock**: Critical level (if was in stock)
- **Back in Stock**: Info level
- **Scraper Failure** (3+ consecutive): Warning level

## User Benefits

1. **Automated Price Tracking**: No manual checking needed
2. **Historical Insights**: See price trends over months
3. **Cost Optimization**: Buy when prices drop
4. **Availability Alerts**: Know when peptides restock
5. **Multi-Supplier Comparison**: Find best deals automatically

## Future Enhancements

1. **ML Price Prediction**: Predict future price trends
2. **Automated Purchase Recommendations**: Alert when to buy
3. **Supplier Reliability Scoring**: Track uptime and stock consistency
4. **API Webhooks**: Real-time notifications via webhook
5. **Mobile Push Notifications**: Alert on mobile devices
6. **Competitor Analysis**: Compare across entire market

---

## Getting Started (For Developers)

### 1. Create Your First Scraper

Create a new file `src-tauri/src/services/scrapers/example_supplier.rs`:

```rust
use async_trait::async_trait;
use anyhow::Result;
use reqwest::Client;
use scraper::{Html, Selector};
use time::OffsetDateTime;

use super::{PeptideScraper, ScrapedData};

pub struct ExampleSupplierScraper {
    client: Client,
}

impl ExampleSupplierScraper {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 PepTrack/1.0")
                .build()
                .unwrap(),
        }
    }
}

#[async_trait]
impl PeptideScraper for ExampleSupplierScraper {
    fn id(&self) -> &str {
        "example-supplier"
    }

    fn name(&self) -> &str {
        "Example Supplier"
    }

    async fn scrape(&self, peptide_name: &str) -> Result<ScrapedData> {
        let url = format!("https://example.com/product/{}", peptide_name);
        let response = self.client.get(&url).send().await?;
        let html = response.text().await?;

        let document = Html::parse_document(&html);

        // Parse price
        let price_selector = Selector::parse(".product-price").unwrap();
        let price_text = document
            .select(&price_selector)
            .next()
            .ok_or_else(|| anyhow::anyhow!("Price not found"))?
            .text()
            .collect::<String>();
        let price = parse_price(&price_text)?;

        // Parse stock status
        let stock_selector = Selector::parse(".stock-status").unwrap();
        let stock_text = document
            .select(&stock_selector)
            .next()
            .map(|e| e.text().collect::<String>())
            .unwrap_or_default();
        let in_stock = stock_text.contains("In Stock");

        Ok(ScrapedData {
            peptide_name: peptide_name.to_string(),
            cost_per_mg: price,
            in_stock,
            url,
            scraped_at: OffsetDateTime::now_utc(),
        })
    }

    async fn health_check(&self) -> Result<bool> {
        let response = self.client.get("https://example.com").send().await?;
        Ok(response.status().is_success())
    }
}

fn parse_price(text: &str) -> Result<f32> {
    // Remove $, commas, and parse
    let cleaned = text.replace(&['$', ','][..], "");
    Ok(cleaned.trim().parse()?)
}
```

### 2. Register the Scraper

In `src-tauri/src/services/scraper_scheduler.rs`:

```rust
pub fn register_scrapers() -> Vec<Box<dyn PeptideScraper>> {
    vec![
        Box::new(ExampleSupplierScraper::new()),
        // Add more scrapers here
    ]
}
```

### 3. Test the Scraper

```rust
#[tokio::test]
async fn test_example_scraper() {
    let scraper = ExampleSupplierScraper::new();
    let result = scraper.scrape("Tirzepatide").await;
    assert!(result.is_ok());
    let data = result.unwrap();
    assert!(data.cost_per_mg > 0.0);
}
```

---

## Conclusion

This architecture provides a robust, extensible foundation for automated price tracking in PepTrack. The modular design allows for easy addition of new suppliers while maintaining reliability and performance. The system respects supplier websites through rate limiting and proper error handling, while providing users with valuable price insights and alerts.

For implementation questions or suggestions, please open an issue on the GitHub repository.
