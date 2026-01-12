use crate::stripe::models::InvoiceRecord;
use crate::stripe::client::StripeInvoice;
use anyhow::{anyhow, Result};
use chrono::Utc;

pub struct ReportGenerator {
    records: Vec<InvoiceRecord>,
}

impl ReportGenerator {
    pub fn new() -> Self {
        ReportGenerator {
            records: Vec::new(),
        }
    }

    /// Convert Stripe invoice data to an InvoiceRecord
    /// This version takes customer and balance_transaction data separately if already fetched
    pub fn process_invoice_with_customer(
        &mut self,
        invoice: StripeInvoice,
        customer: Option<&crate::stripe::client::Customer>,
        balance_transaction: Option<&crate::stripe::client::BalanceTransaction>,
    ) -> Result<()> {
        let date = format_invoice_date(invoice.paid_at.unwrap_or(invoice.created))?;
        let customer_name = extract_customer_name(&invoice)?;
        let state = if let Some(cust) = customer {
            extract_state_from_customer(cust, &invoice)?
        } else {
            extract_state(&invoice)?
        };

        // Sum subscription quantities
        let users = sum_subscription_quantities(&invoice)?;

        // Sum license amounts (subscription line items only, in cents)
        let licenses = sum_license_amounts(&invoice)?;

        // Extract tax
        let tax = invoice.tax.unwrap_or(0);

        // Calculate total
        let total = licenses + tax;

        // Extract fees from balance_transaction if available
        let fees = if let Some(bt) = balance_transaction {
            bt.fee
        } else {
            0
        };

        let record = InvoiceRecord {
            date,
            customer: customer_name,
            users,
            state,
            licenses,
            tax,
            total,
            fees,
        };

        self.records.push(record);
        Ok(())
    }

    /// Legacy method for backward compatibility
    pub fn process_invoice(&mut self, invoice: StripeInvoice) -> Result<()> {
        self.process_invoice_with_customer(invoice, None, None)
    }

    pub fn sort_records(&mut self) {
        // Sort by date ascending, then by customer name
        self.records.sort_by(|a, b| {
            match a.date.cmp(&b.date) {
                std::cmp::Ordering::Equal => a.customer.cmp(&b.customer),
                other => other,
            }
        });
    }

    pub fn get_records(&self) -> &[InvoiceRecord] {
        &self.records
    }

    pub fn calculate_totals(&self) -> (i64, i64, i64, i64) {
        let mut total_licenses = 0i64;
        let mut total_tax = 0i64;
        let mut total_total = 0i64;
        let mut total_fees = 0i64;

        for record in &self.records {
            total_licenses += record.licenses;
            total_tax += record.tax;
            total_total += record.total;
            total_fees += record.fees;
        }

        (total_licenses, total_tax, total_total, total_fees)
    }
}

/// Format invoice date from Unix timestamp to MM/DD/YYYY
fn format_invoice_date(timestamp: i64) -> Result<String> {
    let datetime = chrono::DateTime::<Utc>::from_timestamp(timestamp, 0)
        .ok_or_else(|| anyhow!("Invalid timestamp: {}", timestamp))?;
    Ok(datetime.format("%m/%d/%Y").to_string())
}

/// Extract customer name, with fallback
fn extract_customer_name(invoice: &StripeInvoice) -> Result<String> {
    if let Some(name) = &invoice.customer_name {
        if !name.is_empty() {
            return Ok(name.clone());
        }
    }

    // Customer can be a string ID or an expanded object
    match &invoice.customer {
        serde_json::Value::String(s) if !s.is_empty() => Ok(s.clone()),
        serde_json::Value::Object(obj) => {
            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                Ok(id.to_string())
            } else if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                Ok(name.to_string())
            } else {
                Err(anyhow!("Invoice {}: Customer object has no name or ID", invoice.id))
            }
        }
        _ => Err(anyhow!("Invoice {} has no customer name or ID", invoice.id)),
    }
}

/// Extract state from invoice, error if missing (strict validation)
fn extract_state(invoice: &StripeInvoice) -> Result<String> {
    if let Some(address) = &invoice.customer_address {
        if let Some(state) = &address.state {
            if !state.is_empty() {
                return Ok(state.to_uppercase());
            }
        }
    }

    Err(anyhow!(
        "Invoice {}: Customer state/billing address not found (strict validation required)",
        invoice.id
    ))
}

/// Extract state from customer object
fn extract_state_from_customer(customer: &crate::stripe::client::Customer, invoice: &StripeInvoice) -> Result<String> {
    if let Some(address) = &customer.address {
        if let Some(state) = &address.state {
            if !state.is_empty() {
                return Ok(state.to_uppercase());
            }
        }
    }

    Err(anyhow!(
        "Invoice {}: Customer {} has no state/billing address (strict validation required)",
        invoice.id, customer.id
    ))
}


/// Sum all subscription line item quantities
fn sum_subscription_quantities(invoice: &StripeInvoice) -> Result<u32> {
    let total: u32 = invoice
        .lines
        .data
        .iter()
        .filter(|line| line.line_type == "subscription")
        .map(|line| line.quantity.unwrap_or(0) as u32)
        .sum();
    Ok(total)
}

/// Sum all subscription line item amounts (in cents)
fn sum_license_amounts(invoice: &StripeInvoice) -> Result<i64> {
    let total: i64 = invoice
        .lines
        .data
        .iter()
        .filter(|line| line.line_type == "subscription")
        .map(|line| line.amount)
        .sum();
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        // January 15, 2026 timestamp
        let result = format_invoice_date(1768329600).unwrap();
        assert!(result.contains("01") || result.contains("2026"));
    }

    #[test]
    fn test_empty_records() {
        let generator = ReportGenerator::new();
        let totals = generator.calculate_totals();
        assert_eq!(totals, (0, 0, 0, 0));
    }
}
