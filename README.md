# Stripe Tax Reporter

A CLI tool that generates sales tax reports from Stripe invoices for quarterly tax filing.

## Features

- **Automatic Quarter Detection**: Automatically calculates the previous fiscal quarter
- **Stripe Integration**: Fetches paid invoices directly from Stripe API
- **Tax-Ready Output**: Tab-delimited format ready for copy/paste to Excel
- **Strict Validation**: Ensures all required data (customer state) is present
- **Summary Totals**: Includes quarterly totals for all financial columns

## Installation

### Prerequisites

- Stripe production API key (not test)

### Option 1: Download Pre-built Binary (All Platforms)

Pre-built binaries are available for macOS, Linux, and Windows from the [GitHub releases page](https://github.com/outlawpractice/stripe-tax-reporter/releases/tag/v1.0.2).

**macOS (Intel):**
```bash
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.2/stripe-tax-reporter-macos-x86_64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
export STRIPE_PROD_API_KEY="sk_live_..."
./stripe-tax-reporter
```

**macOS (Apple Silicon):**
```bash
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.2/stripe-tax-reporter-macos-aarch64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
export STRIPE_PROD_API_KEY="sk_live_..."
./stripe-tax-reporter
```

**Linux (x86_64):**
```bash
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.2/stripe-tax-reporter-linux-x86_64 -o stripe-tax-reporter
chmod +x stripe-tax-reporter
export STRIPE_PROD_API_KEY="sk_live_..."
./stripe-tax-reporter
```

**Windows (x86_64):**
```bash
curl -L https://github.com/outlawpractice/stripe-tax-reporter/releases/download/v1.0.2/stripe-tax-reporter-windows-x86_64.exe -o stripe-tax-reporter.exe
export STRIPE_PROD_API_KEY="sk_live_..."
.\stripe-tax-reporter.exe
```

Alternatively, use Option 2 (Build from Source) or Option 3 (Cargo Install) below.

### Option 2: Build from Source

If you prefer to build from source, you'll need:
- Rust 1.70 or later

**Build:**
```bash
git clone https://github.com/outlawpractice/stripe-tax-reporter.git
cd stripe-tax-reporter
cargo build --release
```

The binary will be at `target/release/stripe-tax-reporter`.

### Option 3: Install with Cargo

If you have Rust installed:
```bash
cargo install --git https://github.com/outlawpractice/stripe-tax-reporter.git
```

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

The tool will output tab-delimited data to stdout, grouped by state:

```
===== CALIFORNIA (CA) =====
Date	Customer	Users	Licenses	Tax	Total	Fees
10/20/2025	Acme Corp	5	200.00	17.00	217.00	6.90
11/15/2025	Widget Inc	2	80.00	6.80	86.80	2.75
Subtotal			280.00	23.80	303.80	9.65

===== TEXAS (TX) =====
Date	Customer	Users	Licenses	Tax	Total	Fees
10/15/2025	Margaglione Law PLLC	7	280.00	22.40	302.40	9.42
11/03/2025	Smith & Associates	3	150.00	12.00	162.00	5.01
12/01/2025	Johnson Legal PC	5	200.00	16.00	216.00	6.88
Subtotal			630.00	50.40	680.40	21.31

GRAND TOTAL			910.00	74.20	984.20	40.96
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
| **Licenses** | Subscription revenue (excluding tax) |
| **Tax** | Sales tax amount |
| **Total** | Licenses + Tax |
| **Fees** | Stripe processing fees |

**Note:** The state is shown in the section header (e.g., "===== TEXAS (TX) =====") rather than as a column. Each state gets its own table section with a subtotal row, followed by a grand total across all states.

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

The tool requires state information from at least one of three sources:
1. **Customer profile address** - Update your customer's billing address in Stripe
2. **Credit card billing address** - The payment method's billing address
3. **Invoice address** - The address stored on the invoice itself

Invoices without state data from any of these sources are skipped with a warning. Make sure your customers have at least one source with complete state information.

### No invoices retrieved

Make sure:
1. You have paid invoices in Stripe for the previous quarter
2. Your API key is for the correct Stripe account
3. The invoices have a payment date in the correct quarter

## Data Accuracy

The report includes only:
- **Paid invoices** (status = "paid") with completed payment
- **Subscription charges** (not invoicing items or fees)
- **Invoices with state information** (from customer address, billing address, or invoice address)

The report **excludes**:
- Draft invoices
- Unpaid invoices
- Credits and refunds
- Invoices without state information from any of the three sources

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

## How It Works

### Architecture Overview

The tool is built with three main components:

#### 1. **Stripe Client** (`src/stripe/client.rs`)
Handles all communication with the Stripe API:
- `fetch_paid_invoices()` - Lists all paid invoices for the quarter with pagination (limit=100)
- `fetch_customer()` - Retrieves individual customer records with billing address details
- `fetch_charge()` - Gets charge details to locate the balance transaction
- `fetch_balance_transaction()` - Retrieves fee information from balance transactions

The client uses async/await with reqwest for efficient API calls and implements pagination automatically.

#### 2. **Report Generator** (`src/report/generator.rs`)
Processes invoice data and extracts tax-relevant fields:
- **Customer Name**: Extracted from invoice.customer_name or customer.name
- **Billing State**: Extracted using three-level fallback (see "State Extraction with Three-Level Fallback" below)
- **Users**: Sum of all subscription line item quantities
- **Licenses**: Sum of subscription line item amounts (in cents, converted to dollars)
- **Tax**: From invoice.tax field
- **Total**: Licenses + Tax
- **Fees**: From balance_transaction.fee field
- **Date**: Converted from Unix timestamp to MM/DD/YYYY format

Validation is strict - invoices without state information from any source are skipped with a warning. Uses a three-level fallback:
  1. Customer profile address (if available)
  2. Credit card billing address (if available)
  3. Invoice customer address (if available)

If all three sources lack state information, the invoice is skipped.

#### 3. **Report Formatter** (`src/report/formatter.rs`)
Generates tab-delimited output grouped by state:
- Separate section for each state (alphabetically ordered)
- State header showing the state code (e.g., "===== TEXAS (TX) =====")
- Column headers: Date, Customer, Users, Licenses, Tax, Total, Fees (NO State column)
- Data rows sorted by date (ascending) then customer name (alphabetical) within each state
- Per-state subtotal row showing that state's totals
- Grand total row showing sums across all states
- Currency formatted to 2 decimal places

### Data Flow

```
1. Detect Quarter → Calculate start/end dates for previous fiscal quarter
                  ↓
2. Convert Dates → Transform to Unix timestamps for API filtering
                  ↓
3. Fetch Invoices → GET /v1/invoices?status=paid&created[gte/lte]=...
                  ↓
4. For Each Invoice:
   ├─ Extract charge ID
   ├─ Fetch charge → GET /v1/charges/{charge_id}
   ├─ Get balance_transaction ID from charge
   ├─ Fetch balance_transaction → GET /v1/balance_transactions/{txn_id}
   ├─ Extract customer ID
   └─ Fetch customer → GET /v1/customers/{customer_id}
                  ↓
5. Process Invoice → Validate state, sum amounts, format dates
                  ↓
6. Sort Records → By state (alphabetically), then date (ascending), then customer name
                  ↓
7. Format Output → Group by state, generate per-state sections with subtotals and grand total
                  ↓
8. Output to stdout → Ready for terminal copy/paste to Excel
```

### State Extraction with Three-Level Fallback

The tool uses a three-level fallback to ensure maximum data coverage when extracting customer state:

1. **Customer Profile Address** (Primary Source)
   - Extracted from the Stripe customer object: `customer.address.state`
   - Used when customer has updated their billing address in Stripe

2. **Credit Card Billing Address** (Secondary Fallback)
   - Extracted from the payment charge object: `charge.billing_details.address.state`
   - Used when the customer profile lacks an address but the credit card has billing details

3. **Invoice Customer Address** (Tertiary Fallback)
   - Extracted from the invoice object: `invoice.customer_address.state`
   - Used as final fallback for invoices with address information

If all three sources lack state information, the invoice is skipped with a warning and counted in the "skipped" total.

This three-level approach maximizes the number of invoices that can be reported while maintaining strict validation that every reported invoice has verified state information for tax compliance.

### Fee Extraction Details

Stripe processing fees require a multi-step lookup:
1. Invoice contains a `charge` field (charge ID like `ch_3SniE6H...`)
2. Fetching the charge returns a `balance_transaction` ID (like `txn_3SniE6H...`)
3. Fetching the balance transaction returns the `fee` field (in cents)
4. Fee is converted to dollars (cents ÷ 100) for display

This approach avoids API expand parameter issues and reliably retrieves actual Stripe fees.

## Limitations & Notes

- **Production Only**: The tool uses production Stripe API keys (sk_live_)
- **Multi-State Supported**: Automatically groups invoices by state with per-state subtotals
- **State Validation**: Strict - tool requires invoices to have billing state information from one of three sources: customer address, credit card billing address, or invoice address (skips invoices without state)
- **Paid Invoices Only**: Only includes invoices with status="paid"
- **Subscription Lines Only**: Sums only subscription line items, excludes other line types
- **Multi-User Per Invoice**: Multiple subscription lines per invoice are summed into a single row
- **API Pagination**: Handles up to 100 invoices per request, automatically paginates through all results

## Future Enhancements

- Add `--quarter` flag for historical quarters
- Export to Excel .xlsx format directly
- Configuration file for customization
- Refund and credit tracking
- Command to verify Stripe configuration before running report
- Support for tax rates by state/jurisdiction

## License

Internal use only.

## Support

For issues or questions:
1. Check this README's Troubleshooting section
2. Verify your Stripe account and API key
3. Ensure invoices have complete customer address information
