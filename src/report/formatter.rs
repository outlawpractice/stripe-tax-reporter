use crate::stripe::models::InvoiceRecord;

pub fn format_as_tsv(records: &[InvoiceRecord], totals: (i64, i64, i64, i64)) -> String {
    let mut output = String::new();

    // Header row
    output.push_str("Date\tCustomer\tUsers\tState\tLicenses\tTax\tTotal\tFees\n");

    // Data rows
    for record in records {
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\n",
            record.date,
            record.customer,
            record.users,
            record.state,
            record.licenses_dollars(),
            record.tax_dollars(),
            record.total_dollars(),
            record.fees_dollars(),
        ));
    }

    // Summary/totals row
    let (total_licenses, total_tax, total_total, total_fees) = totals;
    output.push_str(&format!(
        "TOTAL\t\t\t\t{:.2}\t{:.2}\t{:.2}\t{:.2}\n",
        total_licenses as f64 / 100.0,
        total_tax as f64 / 100.0,
        total_total as f64 / 100.0,
        total_fees as f64 / 100.0,
    ));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_empty_records() {
        let records = vec![];
        let totals = (0, 0, 0, 0);
        let output = format_as_tsv(&records, totals);

        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines.len(), 2); // Header + totals
        assert!(lines[0].starts_with("Date\tCustomer"));
        assert!(lines[1].starts_with("TOTAL"));
    }

    #[test]
    fn test_format_with_records() {
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

        let totals = (50000, 4000, 54000, 1600);
        let output = format_as_tsv(&records, totals);

        assert!(output.contains("10/15/2025\tTest Company\t5\tTX\t500.00\t40.00\t540.00\t16.00"));
        assert!(output.contains("TOTAL\t\t\t\t500.00\t40.00\t540.00\t16.00"));
    }
}
