use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeInvoice {
    pub id: String,
    #[serde(default)]
    pub customer: serde_json::Value,
    #[serde(default)]
    pub customer_name: Option<String>,
    #[serde(default)]
    pub customer_address: Option<Address>,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub created: i64,
    #[serde(default)]
    pub paid_at: Option<i64>,
    #[serde(default)]
    pub amount_due: i64,
    #[serde(default)]
    pub amount_paid: i64,
    #[serde(default)]
    pub tax: Option<i64>,
    #[serde(default)]
    pub lines: LineItems,
    #[serde(default)]
    pub charge: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineItems {
    #[serde(default)]
    pub data: Vec<LineItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "type", default)]
    pub line_type: String,
    #[serde(default)]
    pub amount: i64,
    #[serde(default)]
    pub quantity: Option<i32>,
    #[serde(default)]
    pub tax_amounts: Option<Vec<TaxAmount>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxAmount {
    #[serde(default)]
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceListResponse {
    #[serde(default)]
    pub object: String,
    #[serde(default)]
    pub data: Vec<StripeInvoice>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Address {
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub line1: Option<String>,
    #[serde(default)]
    pub line2: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingDetails {
    #[serde(default)]
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Charge {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub balance_transaction: Option<String>,
    #[serde(default)]
    pub billing_details: Option<BillingDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceTransaction {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub fee: i64,
}

pub struct StripeClient {
    api_key: String,
    client: reqwest::Client,
}

impl StripeClient {
    pub fn new(api_key: String) -> Self {
        StripeClient {
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch a customer by ID
    pub async fn fetch_customer(&self, customer_id: &str) -> anyhow::Result<Customer> {
        let url = format!("https://api.stripe.com/v1/customers/{}", customer_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.api_key, Some(""))
            .send()
            .await
            .context("Failed to reach Stripe API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch customer {}: {} {}", customer_id, status, body);
        }

        response
            .json()
            .await
            .context("Failed to parse customer response")
    }

    /// Fetch charge by ID to get balance_transaction reference
    pub async fn fetch_charge(&self, charge_id: &str) -> anyhow::Result<Charge> {
        let url = format!("https://api.stripe.com/v1/charges/{}", charge_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.api_key, Some(""))
            .send()
            .await
            .context("Failed to reach Stripe API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch charge {}: {} {}", charge_id, status, body);
        }

        response
            .json()
            .await
            .context("Failed to parse charge response")
    }

    /// Fetch balance transaction by ID to get fee information
    pub async fn fetch_balance_transaction(&self, balance_tx_id: &str) -> anyhow::Result<BalanceTransaction> {
        let url = format!("https://api.stripe.com/v1/balance_transactions/{}", balance_tx_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.api_key, Some(""))
            .send()
            .await
            .context("Failed to reach Stripe API")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to fetch balance transaction {}: {} {}", balance_tx_id, status, body);
        }

        response
            .json()
            .await
            .context("Failed to parse balance transaction response")
    }

    /// Fetch paid invoices for a date range (Unix timestamps)
    pub async fn fetch_paid_invoices(
        &self,
        start: i64,
        end: i64,
    ) -> anyhow::Result<Vec<StripeInvoice>> {
        let mut all_invoices = Vec::new();
        let mut starting_after: Option<String> = None;

        loop {
            let url = "https://api.stripe.com/v1/invoices";

            // Build URL - we'll fetch charge details separately
            let mut full_url = format!(
                "{}?status=paid&limit=100&created[gte]={}&created[lte]={}",
                url, start, end
            );

            if let Some(starting_after_id) = &starting_after {
                full_url.push_str(&format!("&starting_after={}", starting_after_id));
            }

            let response = self
                .client
                .get(&full_url)
                .basic_auth(&self.api_key, Some(""))
                .send()
                .await
                .context("Failed to reach Stripe API")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!(
                    "Stripe API error {}: {}",
                    status,
                    body
                );
            }

            let invoice_list: InvoiceListResponse = response
                .json()
                .await
                .context("Failed to parse Stripe response")?;

            all_invoices.extend(invoice_list.data);

            if !invoice_list.has_more {
                break;
            }

            // Paginate
            if let Some(last_invoice) = all_invoices.last() {
                starting_after = Some(last_invoice.id.clone());
            }
        }

        Ok(all_invoices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stripe_client_creation() {
        let client = StripeClient::new("sk_test_123".to_string());
        // Just verify it creates without panicking
    }
}
