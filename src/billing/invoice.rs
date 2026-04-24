//! PDF invoice generation

use super::models::*;
use super::{BillingError, BillingResult};
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

/// Data required to generate an invoice
#[derive(Debug, Clone)]
pub struct InvoiceData {
    pub invoice_number: String,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_address: Option<Address>,
    pub issue_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub currency: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub prorations: Vec<ProrationItem>,
    pub subtotal_cents: i64,
    pub tax_cents: i64,
    pub total_cents: i64,
    pub notes: Option<String>,
}

/// PDF invoice generator
pub struct InvoiceGenerator;

impl InvoiceGenerator {
    /// Generate an invoice as HTML
    pub fn generate_html(&self, data: &InvoiceData) -> BillingResult<String> {
        let html = format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Invoice {}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
        }}
        .header {{
            border-bottom: 2px solid #333;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        .invoice-number {{
            font-size: 24px;
            font-weight: bold;
            color: #333;
        }}
        .dates {{
            display: flex;
            justify-content: space-between;
            margin-top: 10px;
        }}
        .company {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 30px;
        }}
        .bill-to {{
            width: 48%;
        }}
        .bill-from {{
            width: 48%;
            text-align: right;
        }}
        .section-title {{
            font-weight: bold;
            margin-bottom: 10px;
            color: #555;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 30px;
        }}
        th {{
            background-color: #f5f5f5;
            padding: 10px;
            text-align: left;
            border-bottom: 2px solid #ddd;
        }}
        td {{
            padding: 10px;
            border-bottom: 1px solid #eee;
        }}
        .amount {{
            text-align: right;
            font-weight: bold;
        }}
        .subtotal {{
            text-align: right;
        }}
        .total {{
            font-size: 18px;
            font-weight: bold;
            text-align: right;
            border-top: 2px solid #333;
            padding-top: 10px;
        }}
        .notes {{
            margin-top: 30px;
            padding: 15px;
            background-color: #f9f9f9;
            border-left: 3px solid #333;
        }}
        .proration {{
            color: #666;
            font-style: italic;
        }}
    </style>
</head>
<body>
    <div class="header">
        <div class="invoice-number">INVOICE #{}</div>
        <div class="dates">
            <div>
                <strong>Issue Date:</strong> {}
            </div>
            <div>
                <strong>Due Date:</strong> {}
            </div>
        </div>
    </div>

    <div class="company">
        <div class="bill-to">
            <div class="section-title">Bill To:</div>
            <div><strong>{}</strong></div>
            <div>{}</div>
            {}
        </div>
        <div class="bill-from">
            <div class="section-title">From:</div>
            <div><strong>Ferroclaw</strong></div>
            <div>A Security-First AI Agent</div>
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>Description</th>
                <th>Quantity</th>
                <th>Period</th>
                <th class="amount">Amount</th>
            </tr>
        </thead>
        <tbody>
{}
{}
        </tbody>
    </table>

    <div class="subtotal">
        <div>Subtotal: {}</div>
        {}
        <div class="total">Total: {}</div>
    </div>

{}
</body>
</html>
"#,
            data.invoice_number,
            data.issue_date.format("%Y-%m-%d"),
            data.due_date.format("%Y-%m-%d"),
            data.customer_name,
            data.customer_email,
            self.format_address(&data.customer_address),
            self.format_line_items(&data.line_items),
            self.format_prorations(&data.prorations),
            self.format_currency(data.subtotal_cents, &data.currency),
            if data.tax_cents > 0 {
                format!("Tax: {}", self.format_currency(data.tax_cents, &data.currency))
            } else {
                String::new()
            },
            self.format_currency(data.total_cents, &data.currency),
            self.format_notes(&data.notes)
        );

        Ok(html)
    }

    /// Generate an invoice as Markdown
    pub fn generate_markdown(&self, data: &InvoiceData) -> BillingResult<String> {
        let mut md = String::new();

        md.push_str(&format!("# Invoice {}\n\n", data.invoice_number));

        md.push_str("**Issue Date:** ");
        md.push_str(&data.issue_date.format("%Y-%m-%d").to_string());
        md.push_str("  \n");

        md.push_str("**Due Date:** ");
        md.push_str(&data.due_date.format("%Y-%m-%d").to_string());
        md.push_str("\n\n");

        md.push_str("## Bill To\n\n");
        md.push_str(&format!("**{}**\n", data.customer_name));
        md.push_str(&format!("{}\n", data.customer_email));
        if let Some(addr) = &data.customer_address {
            md.push_str(&format!(
                "{}, {}, {} {}\n",
                addr.city,
                addr.state.as_deref().unwrap_or(""),
                addr.postal_code,
                addr.country
            ));
        }
        md.push_str("\n");

        md.push_str("## Line Items\n\n");
        md.push_str("| Description | Quantity | Period | Amount |\n");
        md.push_str("|-------------|----------|--------|--------|\n");
        for item in &data.line_items {
            md.push_str(&format!(
                "| {} | {} | {} - {} | {} |\n",
                item.description,
                item.quantity,
                item.period_start.format("%Y-%m-%d"),
                item.period_end.format("%Y-%m-%d"),
                self.format_currency(item.amount_cents, &data.currency)
            ));
        }
        md.push_str("\n");

        if !data.prorations.is_empty() {
            md.push_str("## Prorations\n\n");
            for proration in &data.prorations {
                let proration_type = match proration.proration_type {
                    ProrationType::Upgrade => "Upgrade",
                    ProrationType::Downgrade => "Downgrade",
                    ProrationType::Cancellation => "Cancellation",
                    ProrationType::Credit => "Credit",
                };
                md.push_str(&format!(
                    "* **{}**: {} ({})\n",
                    proration_type,
                    proration.description,
                    self.format_currency(proration.amount_cents, &data.currency)
                ));
            }
            md.push_str("\n");
        }

        md.push_str("## Summary\n\n");
        md.push_str(&format!("**Subtotal:** {}\n", self.format_currency(data.subtotal_cents, &data.currency)));
        if data.tax_cents > 0 {
            md.push_str(&format!("**Tax:** {}\n", self.format_currency(data.tax_cents, &data.currency)));
        }
        md.push_str(&format!("**Total:** {}\n", self.format_currency(data.total_cents, &data.currency)));
        md.push_str("\n");

        if let Some(notes) = &data.notes {
            md.push_str("## Notes\n\n");
            md.push_str(notes);
            md.push_str("\n");
        }

        Ok(md)
    }

    /// Generate an invoice as text (plain)
    pub fn generate_text(&self, data: &InvoiceData) -> BillingResult<String> {
        let mut text = String::new();

        text.push_str(&format!("INVOICE {}\n", data.invoice_number));
        text.push_str(&format!(
            "Issue Date: {}\n",
            data.issue_date.format("%Y-%m-%d")
        ));
        text.push_str(&format!(
            "Due Date: {}\n\n",
            data.due_date.format("%Y-%m-%d")
        ));

        text.push_str("BILL TO:\n");
        text.push_str(&format!("  {}\n", data.customer_name));
        text.push_str(&format!("  {}\n", data.customer_email));
        if let Some(addr) = &data.customer_address {
            text.push_str(&format!("  {}, {}, {} {}\n", addr.city, addr.state.as_deref().unwrap_or(""), addr.postal_code, addr.country));
        }
        text.push_str("\n");

        text.push_str("LINE ITEMS:\n");
        text.push_str(&format!(
            "{:<50} {:>10} {:>20} {:>15}\n",
            "Description", "Qty", "Period", "Amount"
        ));
        text.push_str(&format!(
            "{:<50} {:>10} {:>20} {:>15}\n",
            str::repeat("-", 50), str::repeat("-", 10), str::repeat("-", 20), str::repeat("-", 15)
        ));
        for item in &data.line_items {
            let period = format!(
                "{} - {}",
                item.period_start.format("%Y-%m-%d"),
                item.period_end.format("%Y-%m-%d")
            );
            text.push_str(&format!(
                "{:<50} {:>10} {:>20} {:>15}\n",
                item.description,
                item.quantity,
                period,
                self.format_currency(item.amount_cents, &data.currency)
            ));
        }
        text.push_str("\n");

        if !data.prorations.is_empty() {
            text.push_str("PRORATIONS:\n");
            for proration in &data.prorations {
                text.push_str(&format!(
                    "  - {}: {} ({})\n",
                    proration.description,
                    self.format_currency(proration.amount_cents, &data.currency),
                    format!("{:?}", proration.proration_type)
                ));
            }
            text.push_str("\n");
        }

        text.push_str("SUMMARY:\n");
        text.push_str(&format!(
            "  Subtotal: {:>15}\n",
            self.format_currency(data.subtotal_cents, &data.currency)
        ));
        if data.tax_cents > 0 {
            text.push_str(&format!(
                "  Tax:       {:>15}\n",
                self.format_currency(data.tax_cents, &data.currency)
            ));
        }
        text.push_str(&format!(
            "  TOTAL:     {:>15}\n",
            self.format_currency(data.total_cents, &data.currency)
        ));

        if let Some(notes) = &data.notes {
            text.push_str("\nNOTES:\n");
            text.push_str(notes);
            text.push_str("\n");
        }

        Ok(text)
    }

    /// Save invoice to file
    pub fn save_invoice(
        &self,
        data: &InvoiceData,
        output_path: &Path,
        format: InvoiceFormat,
    ) -> BillingResult<()> {
        let content = match format {
            InvoiceFormat::Html => self.generate_html(data)?,
            InvoiceFormat::Markdown => self.generate_markdown(data)?,
            InvoiceFormat::Text => self.generate_text(data)?,
        };

        let mut file = File::create(output_path)
            .map_err(|e| BillingError::InvoiceGeneration(format!("Failed to create file: {}", e)))?;

        file.write_all(content.as_bytes())
            .map_err(|e| BillingError::InvoiceGeneration(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    // Helper methods

    fn format_address(&self, address: &Option<Address>) -> String {
        match address {
            Some(addr) => format!(
                "{}, {}, {} {}<br>{}",
                addr.line1,
                addr.line2.as_deref().unwrap_or(""),
                addr.city,
                addr.postal_code,
                addr.country
            ),
            None => String::new(),
        }
    }

    fn format_line_items(&self, items: &[InvoiceLineItem]) -> String {
        items
            .iter()
            .map(|item| {
                format!(
                    r#"<tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{} - {}</td>
                        <td class="amount">{}</td>
                    </tr>"#,
                    item.description,
                    item.quantity,
                    item.period_start.format("%Y-%m-%d"),
                    item.period_end.format("%Y-%m-%d"),
                    self.format_currency(item.amount_cents, "USD") // TODO: Use actual currency
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_prorations(&self, prorations: &[ProrationItem]) -> String {
        if prorations.is_empty() {
            return String::new();
        }

        let rows = prorations
            .iter()
            .map(|p| {
                let proration_type = match p.proration_type {
                    ProrationType::Upgrade => "Upgrade",
                    ProrationType::Downgrade => "Downgrade",
                    ProrationType::Cancellation => "Cancellation",
                    ProrationType::Credit => "Credit",
                };
                format!(
                    r#"<tr class="proration">
                        <td colspan="3">{}: {}</td>
                        <td class="amount">{}</td>
                    </tr>"#,
                    proration_type,
                    p.description,
                    self.format_currency(p.amount_cents, "USD")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        format!(
            r#"<tr><th colspan="4">Prorations</th></tr>
            {}"#,
            rows
        )
    }

    fn format_currency(&self, cents: i64, currency: &str) -> String {
        let amount = cents as f64 / 100.0;
        format!("{:.2} {}", amount, currency)
    }

    fn format_notes(&self, notes: &Option<String>) -> String {
        match notes {
            Some(n) => format!(
                r#"<div class="notes">
                    <strong>Notes:</strong>
                    <p>{}</p>
                </div>"#,
                n
            ),
            None => String::new(),
        }
    }
}

/// Invoice output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvoiceFormat {
    Html,
    Markdown,
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_generate_invoice_html() {
        let data = InvoiceData {
            invoice_number: "INV-001".to_string(),
            customer_name: "John Doe".to_string(),
            customer_email: "john@example.com".to_string(),
            customer_address: None,
            issue_date: Utc::now(),
            due_date: Utc::now(),
            currency: "USD".to_string(),
            line_items: vec![InvoiceLineItem {
                id: Uuid::new_v4(),
                description: "Premium Plan".to_string(),
                quantity: 1,
                unit_price_cents: 4900,
                amount_cents: 4900,
                period_start: Utc::now(),
                period_end: Utc::now(),
            }],
            prorations: vec![],
            subtotal_cents: 4900,
            tax_cents: 490,
            total_cents: 5390,
            notes: None,
        };

        let generator = InvoiceGenerator;
        let html = generator.generate_html(&data).unwrap();
        assert!(html.contains("INVOICE #INV-001"));
        assert!(html.contains("John Doe"));
        assert!(html.contains("$49.00"));
    }

    #[test]
    fn test_generate_invoice_markdown() {
        let data = InvoiceData {
            invoice_number: "INV-001".to_string(),
            customer_name: "Jane Smith".to_string(),
            customer_email: "jane@example.com".to_string(),
            customer_address: None,
            issue_date: Utc::now(),
            due_date: Utc::now(),
            currency: "USD".to_string(),
            line_items: vec![],
            prorations: vec![],
            subtotal_cents: 0,
            tax_cents: 0,
            total_cents: 0,
            notes: Some("Thank you!".to_string()),
        };

        let generator = InvoiceGenerator;
        let md = generator.generate_markdown(&data).unwrap();
        assert!(md.contains("# Invoice INV-001"));
        assert!(md.contains("Jane Smith"));
        assert!(md.contains("Thank you!"));
    }
}
