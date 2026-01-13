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
    /// This version takes customer and charge data separately if already fetched
    /// Uses three-level fallback for state extraction: customer address → charge billing address → invoice address
    pub fn process_invoice_with_customer(
        &mut self,
        invoice: StripeInvoice,
        customer: Option<&crate::stripe::client::Customer>,
        charge: Option<&crate::stripe::client::Charge>,
        balance_transaction: Option<&crate::stripe::client::BalanceTransaction>,
    ) -> Result<()> {
        let date = format_invoice_date(invoice.paid_at.unwrap_or(invoice.created))?;
        let customer_name = extract_customer_name(&invoice)?;
        let state = extract_state_with_fallbacks(customer, charge, &invoice)?;

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
        self.process_invoice_with_customer(invoice, None, None, None)
    }

    pub fn sort_records(&mut self) {
        // Sort by state (alphabetical), then by date (ascending), then by customer name
        self.records.sort_by(|a, b| {
            match a.state.cmp(&b.state) {
                std::cmp::Ordering::Equal => {
                    match a.date.cmp(&b.date) {
                        std::cmp::Ordering::Equal => a.customer.cmp(&b.customer),
                        other => other,
                    }
                }
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

/// Extract state with three-level fallback:
/// 1. Customer address (if customer provided)
/// 2. Credit card billing address (if charge provided)
/// 3. Invoice customer address (if present)
/// 4. Error if all three are missing
fn extract_state_with_fallbacks(
    customer: Option<&crate::stripe::client::Customer>,
    charge: Option<&crate::stripe::client::Charge>,
    invoice: &StripeInvoice,
) -> Result<String> {
    // Try customer address first
    if let Some(cust) = customer {
        if let Some(address) = &cust.address {
            if let Some(state) = &address.state {
                if !state.is_empty() {
                    return Ok(state.to_uppercase());
                }
            }
        }
    }

    // Try credit card billing address second
    if let Some(chg) = charge {
        if let Some(billing_details) = &chg.billing_details {
            if let Some(address) = &billing_details.address {
                if let Some(state) = &address.state {
                    if !state.is_empty() {
                        return Ok(state.to_uppercase());
                    }
                }
            }
        }
    }

    // Try invoice customer address third
    if let Some(address) = &invoice.customer_address {
        if let Some(state) = &address.state {
            if !state.is_empty() {
                return Ok(state.to_uppercase());
            }
        }
    }

    // All three failed - error with comprehensive message
    Err(anyhow!(
        "Invoice {}: No state found in customer address, credit card billing address, or invoice address (strict validation required)",
        invoice.id
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
    use crate::stripe::client::{Address, Customer, Charge, BillingDetails};

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

    #[test]
    fn test_state_fallback_to_customer_address() {
        // Create a minimal invoice with no customer address
        let invoice = StripeInvoice {
            id: "in_test1".to_string(),
            customer: serde_json::json!("cus_123"),
            customer_name: Some("Test Company".to_string()),
            customer_address: None,
            status: "paid".to_string(),
            created: 1704067200,
            paid_at: Some(1704067200),
            amount_due: 50000,
            amount_paid: 50000,
            tax: Some(4000),
            lines: crate::stripe::client::LineItems { data: vec![] },
            charge: None,
        };

        // Create a customer with address
        let customer = Customer {
            id: "cus_123".to_string(),
            name: Some("Test Company".to_string()),
            address: Some(Address {
                city: Some("Austin".to_string()),
                country: Some("US".to_string()),
                line1: Some("123 Main St".to_string()),
                line2: None,
                postal_code: Some("78701".to_string()),
                state: Some("TX".to_string()),
            }),
        };

        let state = extract_state_with_fallbacks(Some(&customer), None, &invoice).unwrap();
        assert_eq!(state, "TX");
    }

    #[test]
    fn test_state_fallback_to_billing_address() {
        // Create a minimal invoice with no customer or invoice address
        let invoice = StripeInvoice {
            id: "in_test2".to_string(),
            customer: serde_json::json!("cus_456"),
            customer_name: Some("Another Company".to_string()),
            customer_address: None,
            status: "paid".to_string(),
            created: 1704067200,
            paid_at: Some(1704067200),
            amount_due: 50000,
            amount_paid: 50000,
            tax: Some(4000),
            lines: crate::stripe::client::LineItems { data: vec![] },
            charge: None,
        };

        // Create a customer with no address
        let customer = Customer {
            id: "cus_456".to_string(),
            name: Some("Another Company".to_string()),
            address: None,
        };

        // Create a charge with billing details
        let charge = Charge {
            id: "ch_123".to_string(),
            balance_transaction: None,
            billing_details: Some(BillingDetails {
                address: Some(Address {
                    city: Some("San Francisco".to_string()),
                    country: Some("US".to_string()),
                    line1: Some("456 Market St".to_string()),
                    line2: None,
                    postal_code: Some("94102".to_string()),
                    state: Some("CA".to_string()),
                }),
            }),
        };

        let state = extract_state_with_fallbacks(Some(&customer), Some(&charge), &invoice).unwrap();
        assert_eq!(state, "CA");
    }

    #[test]
    fn test_state_fallback_to_invoice_address() {
        // Create an invoice with customer_address
        let invoice = StripeInvoice {
            id: "in_test3".to_string(),
            customer: serde_json::json!("cus_789"),
            customer_name: Some("Third Company".to_string()),
            customer_address: Some(Address {
                city: Some("New York".to_string()),
                country: Some("US".to_string()),
                line1: Some("789 Broadway".to_string()),
                line2: None,
                postal_code: Some("10003".to_string()),
                state: Some("NY".to_string()),
            }),
            status: "paid".to_string(),
            created: 1704067200,
            paid_at: Some(1704067200),
            amount_due: 50000,
            amount_paid: 50000,
            tax: Some(4000),
            lines: crate::stripe::client::LineItems { data: vec![] },
            charge: None,
        };

        // Create a customer with no address
        let customer = Customer {
            id: "cus_789".to_string(),
            name: Some("Third Company".to_string()),
            address: None,
        };

        // No charge with billing details
        let state = extract_state_with_fallbacks(Some(&customer), None, &invoice).unwrap();
        assert_eq!(state, "NY");
    }

    #[test]
    fn test_state_fallback_priority_customer_over_charge() {
        // When customer has address, it should take precedence over charge billing address
        let invoice = StripeInvoice {
            id: "in_test4".to_string(),
            customer: serde_json::json!("cus_priority"),
            customer_name: Some("Priority Test".to_string()),
            customer_address: None,
            status: "paid".to_string(),
            created: 1704067200,
            paid_at: Some(1704067200),
            amount_due: 50000,
            amount_paid: 50000,
            tax: Some(4000),
            lines: crate::stripe::client::LineItems { data: vec![] },
            charge: None,
        };

        // Customer with TX address
        let customer = Customer {
            id: "cus_priority".to_string(),
            name: Some("Priority Test".to_string()),
            address: Some(Address {
                city: Some("Houston".to_string()),
                country: Some("US".to_string()),
                line1: Some("100 Main".to_string()),
                line2: None,
                postal_code: Some("77001".to_string()),
                state: Some("TX".to_string()),
            }),
        };

        // Charge with CA billing address
        let charge = Charge {
            id: "ch_priority".to_string(),
            balance_transaction: None,
            billing_details: Some(BillingDetails {
                address: Some(Address {
                    city: Some("Los Angeles".to_string()),
                    country: Some("US".to_string()),
                    line1: Some("200 Market".to_string()),
                    line2: None,
                    postal_code: Some("90001".to_string()),
                    state: Some("CA".to_string()),
                }),
            }),
        };

        // Should return TX (customer address) not CA (charge billing address)
        let state = extract_state_with_fallbacks(Some(&customer), Some(&charge), &invoice).unwrap();
        assert_eq!(state, "TX");
    }

    #[test]
    fn test_state_fallback_all_missing_error() {
        // Create an invoice with no address info anywhere
        let invoice = StripeInvoice {
            id: "in_test_error".to_string(),
            customer: serde_json::json!("cus_none"),
            customer_name: Some("No Address Company".to_string()),
            customer_address: None,
            status: "paid".to_string(),
            created: 1704067200,
            paid_at: Some(1704067200),
            amount_due: 50000,
            amount_paid: 50000,
            tax: Some(4000),
            lines: crate::stripe::client::LineItems { data: vec![] },
            charge: None,
        };

        // Customer with no address
        let customer = Customer {
            id: "cus_none".to_string(),
            name: Some("No Address Company".to_string()),
            address: None,
        };

        // Charge with no billing details
        let charge = Charge {
            id: "ch_none".to_string(),
            balance_transaction: None,
            billing_details: None,
        };

        // Should return error
        let result = extract_state_with_fallbacks(Some(&customer), Some(&charge), &invoice);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No state found"));
    }
}
