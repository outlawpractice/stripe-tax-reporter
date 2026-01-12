use crate::stripe::models::InvoiceRecord;
use std::collections::BTreeMap;

pub fn format_as_tsv(records: &[InvoiceRecord]) -> String {
    let mut output = String::new();

    // Group records by state (BTreeMap keeps states alphabetically sorted)
    let mut grouped: BTreeMap<String, Vec<&InvoiceRecord>> = BTreeMap::new();
    for record in records {
        grouped.entry(record.state.clone())
            .or_insert_with(Vec::new)
            .push(record);
    }

    // Track grand totals across all states
    let mut grand_licenses = 0i64;
    let mut grand_tax = 0i64;
    let mut grand_total = 0i64;
    let mut grand_fees = 0i64;

    // Output each state section
    for (state, state_records) in &grouped {
        // State section header
        output.push_str(&format!("===== {} =====\n", state));

        // Column headers (NO State column)
        output.push_str("Date\tCustomer\tUsers\tLicenses\tTax\tTotal\tFees\n");

        // Data rows for this state
        for record in state_records {
            output.push_str(&format!(
                "{}\t{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\n",
                record.date,
                record.customer,
                record.users,
                record.licenses_dollars(),
                record.tax_dollars(),
                record.total_dollars(),
                record.fees_dollars(),
            ));
        }

        // Calculate state subtotals
        let mut state_licenses = 0i64;
        let mut state_tax = 0i64;
        let mut state_total = 0i64;
        let mut state_fees = 0i64;

        for record in state_records {
            state_licenses += record.licenses;
            state_tax += record.tax;
            state_total += record.total;
            state_fees += record.fees;
        }

        // State subtotal row
        output.push_str(&format!(
            "Subtotal\t\t\t{:.2}\t{:.2}\t{:.2}\t{:.2}\n\n",
            state_licenses as f64 / 100.0,
            state_tax as f64 / 100.0,
            state_total as f64 / 100.0,
            state_fees as f64 / 100.0,
        ));

        // Add to grand totals
        grand_licenses += state_licenses;
        grand_tax += state_tax;
        grand_total += state_total;
        grand_fees += state_fees;
    }

    // Grand total section
    output.push_str(&format!(
        "GRAND TOTAL\t\t\t{:.2}\t{:.2}\t{:.2}\t{:.2}\n",
        grand_licenses as f64 / 100.0,
        grand_tax as f64 / 100.0,
        grand_total as f64 / 100.0,
        grand_fees as f64 / 100.0,
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty_records() {
        let records = vec![];
        let output = format_as_tsv(&records);

        // With no records, should only contain grand total
        assert!(output.contains("GRAND TOTAL"));
    }

    #[test]
    fn test_format_single_state() {
        let records = vec![InvoiceRecord {
            date: "10/15/2025".to_string(),
            customer: "Test Company".to_string(),
            users: 5,
            state: "TX".to_string(),
            licenses: 50000,  // $500.00
            tax: 4000,        // $40.00
            total: 54000,     // $540.00
            fees: 1600,       // $16.00
        }];

        let output = format_as_tsv(&records);

        // Should have state header
        assert!(output.contains("===== TX ====="));
        // Should have correct data row (NO state column)
        assert!(output.contains("10/15/2025\tTest Company\t5\t500.00\t40.00\t540.00\t16.00"));
        // Should have state subtotal
        assert!(output.contains("Subtotal\t\t\t500.00\t40.00\t540.00\t16.00"));
        // Should have grand total
        assert!(output.contains("GRAND TOTAL\t\t\t500.00\t40.00\t540.00\t16.00"));
    }

    #[test]
    fn test_format_multiple_states() {
        let records = vec![
            InvoiceRecord {
                date: "10/15/2025".to_string(),
                customer: "TX Company".to_string(),
                users: 5,
                state: "TX".to_string(),
                licenses: 50000,  // $500.00
                tax: 4000,        // $40.00
                total: 54000,     // $540.00
                fees: 1600,       // $16.00
            },
            InvoiceRecord {
                date: "10/20/2025".to_string(),
                customer: "CA Company".to_string(),
                users: 3,
                state: "CA".to_string(),
                licenses: 30000,  // $300.00
                tax: 2000,        // $20.00
                total: 32000,     // $320.00
                fees: 900,        // $9.00
            },
        ];

        let output = format_as_tsv(&records);

        // Should have both state headers (CA should come first alphabetically)
        assert!(output.contains("===== CA ====="));
        assert!(output.contains("===== TX ====="));

        // Check CA section
        let ca_index = output.find("===== CA =====").unwrap();
        let tx_index = output.find("===== TX =====").unwrap();
        assert!(ca_index < tx_index, "CA should appear before TX");

        // Should have both companies in data
        assert!(output.contains("TX Company"));
        assert!(output.contains("CA Company"));

        // Should have per-state subtotals
        assert!(output.contains("Subtotal\t\t\t300.00\t20.00\t320.00\t9.00"));
        assert!(output.contains("Subtotal\t\t\t500.00\t40.00\t540.00\t16.00"));

        // Grand total should sum both states
        assert!(output.contains("GRAND TOTAL\t\t\t800.00\t60.00\t860.00\t25.00"));
    }
}
