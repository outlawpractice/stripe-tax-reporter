# Stripe Tax Reporter

A CLI tool that generates Texas sales tax reports from Stripe invoices for quarterly tax filing.

## Features

- **Automatic Quarter Detection**: Automatically calculates the previous fiscal quarter
- **Stripe Integration**: Fetches paid invoices directly from Stripe API
- **Tax-Ready Output**: Tab-delimited format ready for copy/paste to Excel
- **Strict Validation**: Ensures all required data (customer state) is present
- **Summary Totals**: Includes quarterly totals for all financial columns

## Installation

### Prerequisites

- Rust 1.70 or later
- Stripe production API key (not test)

### Build from Source

```bash
cd ~/projects/stripe-tax-reporter
cargo build --release
```

The binary will be at `target/release/stripe-tax-reporter`.

## Usage

### Setup

1. Add your Stripe production API key to your shell profile:

```bash
# Add to ~/.zshrc or ~/.bashrc
export STRIPE_PROD_API_KEY="sk_live_..."
```

2. Reload your shell:
```bash
source ~/.zshrc  # or ~/.bashrc
```

### Generate Report

Run the tool to generate the report for the previous fiscal quarter:

```bash
stripe-tax-reporter
```

The tool will output tab-delimited data to stdout:

```
Date	Customer	Users	State	Licenses	Tax	Total	Fees
10/15/2025	Margaglione Law PLLC	7	TX	280.00	22.40	302.40	9.42
11/03/2025	Smith & Associates	3	TX	150.00	12.00	162.00	5.01
12/01/2025	Johnson Legal PC	5	TX	200.00	16.00	216.00	6.88
TOTAL				630.00	50.40	680.40	21.31
```

### Copy to Excel

1. Run the tool: `stripe-tax-reporter`
2. Copy all output from your terminal
3. Open Excel and click cell A1
4. Paste (Cmd+V on Mac, Ctrl+V on Windows)
5. Verify columns are properly separated into individual cells

## Column Definitions

| Column | Description |
|--------|-------------|
| **Date** | Invoice payment date (MM/DD/YYYY format) |
| **Customer** | Customer business name from Stripe |
| **Users** | Total subscription quantity/licensed users |
| **State** | Two-letter state code (TX) |
| **Licenses** | Subscription revenue (excluding tax) |
| **Tax** | Sales tax amount |
| **Total** | Licenses + Tax |
| **Fees** | Stripe processing fees |

The last row shows quarterly totals for Licenses, Tax, Total, and Fees.

## Quarterly Selection

The tool always reports on the **previous fiscal quarter**:

- January run → Reports Q4 of previous year (Oct-Dec)
- April run → Reports Q1 (Jan-Mar)
- July run → Reports Q2 (Apr-Jun)
- October run → Reports Q3 (Jul-Sep)

## Troubleshooting

### "STRIPE_PROD_API_KEY environment variable not set"

Make sure your Stripe API key is exported:
```bash
export STRIPE_PROD_API_KEY="sk_live_..."
```

### "Stripe API error 401 Unauthorized"

Your API key is invalid or expired. Verify it's correct and has the necessary permissions to list invoices.

### "Customer state/billing address not found"

The tool uses strict validation - it requires all invoices to have a customer billing address with state information. Invoices without state data are skipped with a warning. Update your customer profiles in Stripe to include billing addresses.

### No invoices retrieved

Make sure:
1. You have paid invoices in Stripe for the previous quarter
2. Your API key is for the correct Stripe account
3. The invoices have a payment date in the correct quarter

## Data Accuracy

The report includes only:
- **Paid invoices** (status = "paid") with completed payment
- **Subscription charges** (not invoicing items or fees)
- **Texas addresses** (due to state validation requirement)

The report **excludes**:
- Draft invoices
- Unpaid invoices
- Credits and refunds
- Invoices without customer address information

## Output Format

The output uses tab-delimited format (TSV) with 8 columns:

- UTF-8 encoding
- Tab character as delimiter
- Header row on first line
- Data rows sorted by date ascending, then customer name
- Summary totals on last line
- Currency values formatted to 2 decimal places

## Performance

The tool uses pagination to handle large numbers of invoices:
- Fetches up to 100 invoices per API request
- Automatically paginates through all results
- Typical runtime: 2-10 seconds depending on invoice count

## Limitations & Notes

- **Production Only**: The tool uses production Stripe API keys (sk_live_)
- **Texas Specific**: Currently configured for Texas tax reporting
- **State Validation**: Strict - tool errors if any invoice lacks state information
- **Fees**: Currently shows $0 - fee extraction from balance transactions is planned
- **Multi-User Per Invoice**: Multiple subscription lines per invoice are summed into a single row

## Future Enhancements

- Add `--quarter` flag for historical quarters
- Export to Excel .xlsx format directly
- Support for multiple states/jurisdictions
- Fee extraction from Stripe balance transactions
- Configuration file for customization
- Refund and credit tracking

## License

Internal use only.

## Support

For issues or questions:
1. Check this README's Troubleshooting section
2. Verify your Stripe account and API key
3. Ensure invoices have complete customer address information
