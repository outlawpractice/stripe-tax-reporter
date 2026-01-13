use clap::Parser;
use anyhow::Result;

mod stripe;
mod report;

use report::{get_previous_quarter, ReportGenerator, format_as_tsv};
use stripe::StripeClient;

#[derive(Parser, Debug)]
#[command(name = "Stripe Tax Reporter")]
#[command(about = "Generate Texas sales tax reports from Stripe invoices", long_about = None)]
#[command(version = "0.1.0")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Generate tax report for previous fiscal quarter
    Generate,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Generate) | None => {
            // Prefer production API key, fall back to test key
            let api_key = std::env::var("STRIPE_PROD_API_KEY")
                .or_else(|_| std::env::var("STRIPE_API_KEY"))
                .map_err(|_| anyhow::anyhow!("Neither STRIPE_PROD_API_KEY nor STRIPE_API_KEY environment variable is set"))?;

            let (start_date, end_date, quarter, year) = get_previous_quarter();
            eprintln!("Generating report for Q{} {} ({} to {})", quarter, year, start_date, end_date);

            // Convert dates to Unix timestamps
            let start_timestamp = start_date.and_hms_opt(0, 0, 0)
                .ok_or(anyhow::anyhow!("Invalid start date"))?
                .and_utc()
                .timestamp();

            let end_timestamp = end_date.and_hms_opt(23, 59, 59)
                .ok_or(anyhow::anyhow!("Invalid end date"))?
                .and_utc()
                .timestamp();

            let client = StripeClient::new(api_key);
            eprintln!("Fetching invoices from Stripe...");

            let invoices = client.fetch_paid_invoices(start_timestamp, end_timestamp).await?;
            eprintln!("Retrieved {} invoices", invoices.len());

            let mut generator = ReportGenerator::new();

            // Process each invoice
            let mut processed = 0;
            let mut skipped = 0;
            for invoice in invoices {
                // Extract customer ID
                let customer_id = match &invoice.customer {
                    serde_json::Value::String(s) if !s.is_empty() => s.clone(),
                    serde_json::Value::Object(obj) => {
                        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                            id.to_string()
                        } else {
                            eprintln!("Warning: Skipping invoice {}: No customer ID found", invoice.id);
                            skipped += 1;
                            continue;
                        }
                    }
                    _ => {
                        eprintln!("Warning: Skipping invoice {}: No customer ID found", invoice.id);
                        skipped += 1;
                        continue;
                    }
                };

                // Fetch customer details
                match client.fetch_customer(&customer_id).await {
                    Ok(customer) => {
                        let mut charge_data = None;
                        let mut balance_transaction = None;

                        if let Some(charge_value) = &invoice.charge {
                            if let serde_json::Value::String(charge_id) = charge_value {
                                // Fetch the charge to get its balance_transaction ID and billing address
                                if let Ok(charge) = client.fetch_charge(charge_id).await {
                                    // Extract balance_transaction for fees
                                    if let Some(balance_tx_id) = &charge.balance_transaction {
                                        if let Ok(bt) = client.fetch_balance_transaction(balance_tx_id).await {
                                            balance_transaction = Some(bt);
                                        }
                                    }
                                    // Store charge for state fallback
                                    charge_data = Some(charge);
                                }
                            }
                        }

                        match generator.process_invoice_with_customer(
                            invoice.clone(),
                            Some(&customer),
                            charge_data.as_ref(),
                            balance_transaction.as_ref()
                        ) {
                            Ok(_) => processed += 1,
                            Err(e) => {
                                eprintln!("Warning: Skipping invoice {}: {}", invoice.id, e);
                                skipped += 1;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Skipping invoice {}: Failed to fetch customer: {}", invoice.id, e);
                        skipped += 1;
                    }
                }
            }

            eprintln!("Processed {} invoices, skipped {}", processed, skipped);

            // Sort records (by state, then date, then customer)
            generator.sort_records();

            // Format and output as TSV (formatter calculates per-state subtotals internally)
            let tsv_output = format_as_tsv(generator.get_records());
            println!("{}", tsv_output);

            Ok(())
        }
    }
}
