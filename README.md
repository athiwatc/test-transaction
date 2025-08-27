Native Token Transfer (Type-2 / EIP-1559)

This project contains a minimal Rust script that sends a native token transfer (ETH on EVM chains) and forces the transaction to use EIP-1559 (type-2).

Setup

- Copy `.env.example` to `.env` and fill in your details:
  - `RPC_URL` – HTTPS RPC endpoint (e.g., Infura/Alchemy)
  - `PRIVATE_KEY` – Sender's private key (0x-prefixed)
  - `TO_ADDRESS` – Recipient address
  - Optional: `AMOUNT_ETH`, `CHAIN_ID`, `PRIORITY_GWEI`, `FEE_MULTIPLIER`

Build & Run

```bash
cargo run --release
```

Notes

- The script constructs an `Eip1559TransactionRequest` explicitly, ensuring a type-2 transaction.
- Fees default to: priority=2 gwei; max_fee = base_fee * 2 + priority. Override via env vars if needed.
- Default `CHAIN_ID` is Sepolia (11155111). Set to your target chain if different.

