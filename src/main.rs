use std::{env, sync::Arc};

use dotenvy::dotenv;
use eyre::{eyre, Result};
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::types::transaction::eip2930::AccessList;
use ethers::types::{
    Address, BlockNumber, Eip1559TransactionRequest, NameOrAddress, TransactionRequest, U256,
};
use ethers::utils::parse_units;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Required env vars
    let rpc_url = env::var("RPC_URL").map_err(|_| eyre!("RPC_URL not set"))?;
    let priv_key = env::var("PRIVATE_KEY").map_err(|_| eyre!("PRIVATE_KEY not set"))?;
    let to_addr = env::var("TO_ADDRESS").map_err(|_| eyre!("TO_ADDRESS not set"))?;

    // Optional env vars with sensible defaults
    let amount_eth = env::var("AMOUNT_ETH").unwrap_or_else(|_| "0.001".to_string());
    let chain_id: u64 = env::var("CHAIN_ID").unwrap_or_else(|_| "11155111".to_string()).parse()?; // default: Sepolia
    let priority_gwei = env::var("PRIORITY_GWEI").unwrap_or_else(|_| "2".to_string());
    let fee_multiplier: u64 = env::var("FEE_MULTIPLIER").unwrap_or_else(|_| "2".to_string()).parse().unwrap_or(2);

    // Provider and wallet
    let provider = Provider::<Http>::try_from(rpc_url.clone())?;
    let wallet: LocalWallet = priv_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    let from = wallet.address();
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    // Parse inputs
    let to: Address = to_addr.parse()?;
    let value = parse_units(&amount_eth, "ether").map_err(|e| eyre!("invalid AMOUNT_ETH: {e}"))?;

    // Gasless chain: set all fee-related fields to zero
    let max_priority_fee_per_gas: U256 = U256::zero();
    let max_fee_per_gas: U256 = U256::zero();
    let suggested_gas_price = Some(U256::zero());

    println!("From={} To={} Amount={} ETH", format_address(from), format_address(to), amount_eth);

    // Series A: fees set to 0
    println!("\nSeries: fees=0");
    let mut results: Vec<(u8, String)> = Vec::new();
    for tx_type in 0u8..=5u8 {
        match build_tx(
            tx_type,
            from,
            to,
            value.into(),
            suggested_gas_price,
            max_priority_fee_per_gas,
            max_fee_per_gas,
        ) {
            Ok(tx) => {
                println!("Attempting type-{} (fees=0)…", tx_type);
                match client.send_transaction(tx, None).await {
                    Ok(pending) => {
                        println!("  submitted: 0x{:x}", pending.tx_hash());
                        match pending.await {
                            Ok(Some(r)) => {
                                let status = r
                                    .status
                                    .map(|s| if s.as_u64() == 1 { "success" } else { "failed" })
                                    .unwrap_or("unknown");
                                println!(
                                    "  mined in block {} (status: {})",
                                    r.block_number
                                        .map(|n| n.to_string())
                                        .unwrap_or_else(|| "?".into()),
                                    status
                                );
                                results.push((tx_type, status.to_string()));
                            }
                            Ok(None) => {
                                println!("  pending (no receipt yet)");
                                results.push((tx_type, "pending".into()));
                            }
                            Err(e) => {
                                println!("  error awaiting receipt: {}", e);
                                results.push((tx_type, format!("await error: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        println!("  submission failed: {}", e);
                        results.push((tx_type, format!("submit error: {}", e)));
                    }
                }
            }
            Err(e) => {
                println!("Skipping type-{}: {}", tx_type, e);
                results.push((tx_type, "unsupported".into()));
            }
        }
    }

    println!("\nSummary (fees=0):");
    for (t, status) in results {
        println!("  type-{}: {}", t, status);
    }

    // Series B: fees set to 1
    println!("\nSeries: fees=1");
    let suggested_gas_price_1 = Some(U256::from(1));
    let max_priority_fee_per_gas_1 = U256::from(1);
    let max_fee_per_gas_1 = U256::from(1);

    let mut results_one: Vec<(u8, String)> = Vec::new();
    for tx_type in 0u8..=5u8 {
        match build_tx(
            tx_type,
            from,
            to,
            value.into(),
            suggested_gas_price_1,
            max_priority_fee_per_gas_1,
            max_fee_per_gas_1,
        ) {
            Ok(tx) => {
                println!("Attempting type-{} (fees=1)…", tx_type);
                match client.send_transaction(tx, None).await {
                    Ok(pending) => {
                        println!("  submitted: 0x{:x}", pending.tx_hash());
                        match pending.await {
                            Ok(Some(r)) => {
                                let status = r
                                    .status
                                    .map(|s| if s.as_u64() == 1 { "success" } else { "failed" })
                                    .unwrap_or("unknown");
                                println!(
                                    "  mined in block {} (status: {})",
                                    r.block_number
                                        .map(|n| n.to_string())
                                        .unwrap_or_else(|| "?".into()),
                                    status
                                );
                                results_one.push((tx_type, status.to_string()));
                            }
                            Ok(None) => {
                                println!("  pending (no receipt yet)");
                                results_one.push((tx_type, "pending".into()));
                            }
                            Err(e) => {
                                println!("  error awaiting receipt: {}", e);
                                results_one.push((tx_type, format!("await error: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        println!("  submission failed: {}", e);
                        results_one.push((tx_type, format!("submit error: {}", e)));
                    }
                }
            }
            Err(e) => {
                println!("Skipping type-{}: {}", tx_type, e);
                results_one.push((tx_type, "unsupported".into()));
            }
        }
    }

    println!("\nSummary (fees=1):");
    for (t, status) in results_one {
        println!("  type-{}: {}", t, status);
    }

    Ok(())
}

fn format_address(addr: Address) -> String {
    let s = format!("0x{:x}", addr);
    if s.len() > 12 {
        format!("{}…{}", &s[..8], &s[s.len() - 4..])
    } else {
        s
    }
}

fn format_gwei(v: U256) -> String {
    // best-effort pretty formatting for logs only
    let gwei = v / U256::exp10(9);
    gwei.to_string()
}

fn build_tx(
    tx_type: u8,
    from: Address,
    to: Address,
    value: U256,
    gas_price: Option<U256>,
    max_priority_fee_per_gas: U256,
    max_fee_per_gas: U256,
) -> Result<TypedTransaction> {
    match tx_type {
        // 0: Legacy
        0 => {
            let mut tx = TransactionRequest::default();
            tx.from = Some(from);
            tx.to = Some(NameOrAddress::Address(to));
            tx.value = Some(value);
            if let Some(gp) = gas_price { tx.gas_price = Some(gp); }
            Ok(tx.into())
        }
        // 1: EIP-2930 (access list)
        1 => {
            let mut legacy = TransactionRequest::default();
            legacy.from = Some(from);
            legacy.to = Some(NameOrAddress::Address(to));
            legacy.value = Some(value);
            if let Some(gp) = gas_price { legacy.gas_price = Some(gp); }
            let tx2930 = ethers::types::Eip2930TransactionRequest::new(legacy, AccessList::default());
            Ok(tx2930.into())
        }
        // 2: EIP-1559
        2 => {
            let mut tx = Eip1559TransactionRequest::default();
            tx.from = Some(from);
            tx.to = Some(NameOrAddress::Address(to));
            tx.value = Some(value);
            tx.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
            tx.max_fee_per_gas = Some(max_fee_per_gas);
            Ok(tx.into())
        }
        // 3..=5: Not supported by current ethers typed transaction API
        3 | 4 | 5 => Err(eyre!(
            "unsupported by current ethers TypedTransaction (no variant for type {})",
            tx_type
        )),
        // Any other value: error
        _ => Err(eyre!("unknown tx type {}", tx_type)),
    }
}
