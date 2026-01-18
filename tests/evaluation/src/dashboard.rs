//! Interactive dashboard generation.
//!
//! This module generates HTML dashboards with embedded Chart.js visualizations
//! for stakeholder-friendly reporting and trend visualization.

use serde_json::json;
use std::collections::HashMap;

use crate::timeseries::EvaluationRecord;

/// Dashboard generator
pub struct DashboardGenerator {
    title: String,
}

impl DashboardGenerator {
    /// Create a new dashboard generator
    pub fn new() -> Self {
        Self {
            title: "Evaluation Dashboard".to_string(),
        }
    }

    /// Generate HTML dashboard from evaluation data
    pub fn generate(&self, data: &[EvaluationRecord]) -> Result<String, String> {
        let chart_builder = ChartDataBuilder::new();
        let chart_json = chart_builder.build_trend_chart(data)?;

        let summary = SummaryStats::from_records(data);

        let mut html = String::new();

        // HTML header
        html.push_str("<!DOCTYPE html>\n");
        html.push_str("<html>\n");
        html.push_str("<head>\n");
        html.push_str(&format!("  <title>{}</title>\n", self.title));
        html.push_str("  <meta charset=\"utf-8\">\n");
        html.push_str("  <script src=\"https://cdn.jsdelivr.net/npm/chart.js\"></script>\n");
        html.push_str("</head>\n");
        html.push_str("<body>\n");

        // Dashboard header
        html.push_str(&format!("  <h1>{}</h1>\n", self.title));

        // Summary section
        html.push_str("  <div class=\"summary\">\n");
        html.push_str(&format!(
            "    <p>Total Evaluations: {}</p>\n",
            summary.total_evaluations
        ));
        html.push_str(&format!(
            "    <p>Latest Pass Rate: {:.1}%</p>\n",
            summary.latest_pass_rate * 100.0
        ));
        html.push_str(&format!(
            "    <p>Average Pass Rate: {:.1}%</p>\n",
            summary.average_pass_rate * 100.0
        ));
        html.push_str(&format!("    <p>Trend: {}</p>\n", summary.trend_direction));
        html.push_str("  </div>\n");

        // Chart section
        html.push_str("  <div class=\"chart-container\">\n");
        html.push_str("    <canvas id=\"trendChart\"></canvas>\n");
        html.push_str("  </div>\n");

        // Embed chart data
        html.push_str("  <script>\n");
        html.push_str(&format!("    const chartData = {};\n", chart_json));
        html.push_str("    const ctx = document.getElementById('trendChart').getContext('2d');\n");
        html.push_str("    new Chart(ctx, {\n");
        html.push_str("      type: 'line',\n");
        html.push_str("      data: chartData,\n");
        html.push_str("      options: {\n");
        html.push_str("        responsive: true,\n");
        html.push_str("        scales: {\n");
        html.push_str("          y: {\n");
        html.push_str("            beginAtZero: true,\n");
        html.push_str("            max: 1.0,\n");
        html.push_str("            ticks: {\n");
        html.push_str("              callback: function(value) { return (value * 100) + '%'; }\n");
        html.push_str("            }\n");
        html.push_str("          }\n");
        html.push_str("        }\n");
        html.push_str("      }\n");
        html.push_str("    });\n");
        html.push_str("  </script>\n");

        // Include backend names and categories as comments
        for record in data {
            html.push_str(&format!(
                "  <!-- Backend: {}, Category: {}, Pass Rate: {:.2} -->\n",
                record.backend, record.category, record.pass_rate
            ));
        }

        html.push_str("</body>\n");
        html.push_str("</html>\n");

        Ok(html)
    }
}

impl Default for DashboardGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Chart data builder
pub struct ChartDataBuilder;

impl ChartDataBuilder {
    /// Create a new chart data builder
    pub fn new() -> Self {
        Self
    }

    /// Build Chart.js data for trend visualization
    pub fn build_trend_chart(&self, data: &[EvaluationRecord]) -> Result<String, String> {
        let labels: Vec<String> = data.iter().map(|r| r.timestamp.clone()).collect();

        let datasets = vec![json!({
            "label": "Pass Rate",
            "data": data.iter().map(|r| r.pass_rate).collect::<Vec<f64>>(),
            "borderColor": "rgb(75, 192, 192)",
            "backgroundColor": "rgba(75, 192, 192, 0.2)"
        })];

        let chart_data = json!({
            "labels": labels,
            "datasets": datasets
        });

        serde_json::to_string(&chart_data).map_err(|e| e.to_string())
    }
}

impl Default for ChartDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Heatmap generator
pub struct HeatmapGenerator;

impl HeatmapGenerator {
    /// Create a new heatmap generator
    pub fn new() -> Self {
        Self
    }

    /// Generate HTML heatmap for model × category comparison
    pub fn generate(&self, data: &[EvaluationRecord]) -> Result<String, String> {
        let mut html = String::new();

        // Group by backend and category
        let mut grid: HashMap<(String, String), f64> = HashMap::new();
        let mut backends = std::collections::HashSet::new();
        let mut categories = std::collections::HashSet::new();

        for record in data {
            backends.insert(record.backend.clone());
            categories.insert(record.category.clone());
            grid.insert(
                (record.backend.clone(), record.category.clone()),
                record.pass_rate,
            );
        }

        let mut backends_vec: Vec<String> = backends.into_iter().collect();
        backends_vec.sort();

        let mut categories_vec: Vec<String> = categories.into_iter().collect();
        categories_vec.sort();

        // Generate HTML table
        html.push_str("<table>\n");

        // Header row
        html.push_str("  <tr>\n");
        html.push_str("    <th>Backend</th>\n");
        for category in &categories_vec {
            html.push_str(&format!("    <th>{}</th>\n", category));
        }
        html.push_str("  </tr>\n");

        // Data rows
        for backend in &backends_vec {
            html.push_str("  <tr>\n");
            html.push_str(&format!("    <td>{}</td>\n", backend));

            for category in &categories_vec {
                let pass_rate = grid
                    .get(&(backend.clone(), category.clone()))
                    .unwrap_or(&0.0);
                html.push_str(&format!("    <td>{:.0}%</td>\n", pass_rate * 100.0));
            }

            html.push_str("  </tr>\n");
        }

        html.push_str("</table>\n");

        Ok(html)
    }
}

impl Default for HeatmapGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary statistics
#[derive(Debug, Clone)]
pub struct SummaryStats {
    /// Total number of evaluations
    pub total_evaluations: usize,
    /// Latest pass rate
    pub latest_pass_rate: f64,
    /// Average pass rate across all evaluations
    pub average_pass_rate: f64,
    /// Trend direction ("improving", "stable", or "regressing")
    pub trend_direction: String,
}

impl SummaryStats {
    /// Calculate summary statistics from evaluation records
    pub fn from_records(records: &[EvaluationRecord]) -> Self {
        if records.is_empty() {
            return Self {
                total_evaluations: 0,
                latest_pass_rate: 0.0,
                average_pass_rate: 0.0,
                trend_direction: "unknown".to_string(),
            };
        }

        let total_evaluations = records.len();
        let latest_pass_rate = records.last().map(|r| r.pass_rate).unwrap_or(0.0);

        let sum: f64 = records.iter().map(|r| r.pass_rate).sum();
        let average_pass_rate = sum / total_evaluations as f64;

        // Determine trend
        let trend_direction = if records.len() < 2 {
            "unknown".to_string()
        } else {
            let first_half = &records[0..records.len() / 2];
            let second_half = &records[records.len() / 2..];

            let first_avg: f64 =
                first_half.iter().map(|r| r.pass_rate).sum::<f64>() / first_half.len() as f64;
            let second_avg: f64 =
                second_half.iter().map(|r| r.pass_rate).sum::<f64>() / second_half.len() as f64;

            let diff = second_avg - first_avg;

            if diff > 0.05 {
                "improving".to_string()
            } else if diff < -0.05 {
                "regressing".to_string()
            } else {
                "stable".to_string()
            }
        };

        Self {
            total_evaluations,
            latest_pass_rate,
            average_pass_rate,
            trend_direction,
        }
    }
}
