use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRecord {
    pub date: String,              // MM/DD/YYYY format
    pub customer: String,           // Customer name
    pub users: u32,                 // Total subscription quantity
    pub state: String,              // Two-letter state code
    pub licenses: i64,              // Amount in cents
    pub tax: i64,                   // Amount in cents
    pub total: i64,                 // licenses + tax (cents)
    pub fees: i64,                  // Amount in cents
}

impl InvoiceRecord {
    pub fn licenses_dollars(&self) -> f64 {
        self.licenses as f64 / 100.0
    }

    pub fn tax_dollars(&self) -> f64 {
        self.tax as f64 / 100.0
    }

    pub fn total_dollars(&self) -> f64 {
        self.total as f64 / 100.0
    }

    pub fn fees_dollars(&self) -> f64 {
        self.fees as f64 / 100.0
    }
}
