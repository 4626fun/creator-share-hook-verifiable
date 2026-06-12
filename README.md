# 4626 creator_share_hook — verifiable source

Minimal source snapshot for the deployed 4626 Transfer Hook program on the Solana
spoke: buy detection, lottery entry recording, fee harvesting, and winner notification
for Token-2022 creator share tokens.

| | |
|---|---|
| Program ID | [`EjpziSWGRcEiDHLXft5etbUtcJiZxEttkwz1tqiuzzWU`](https://explorer.solana.com/address/EjpziSWGRcEiDHLXft5etbUtcJiZxEttkwz1tqiuzzWU/verified-build) |
| Network | Solana mainnet-beta |
| Verified build status | [verify.osec.io](https://verify.osec.io/status/EjpziSWGRcEiDHLXft5etbUtcJiZxEttkwz1tqiuzzWU) |
| Expected program hash | `76bebcd7d38bd765a0905d78bc02df02eeffc8113396e86109162082d4d0a54c` |
| Framework | Anchor (anchor-lang 1.0.2), platform-tools v1.52 |

## Reproduce the on-chain hash

Requires Docker and [solana-verify](https://github.com/Ellipsis-Labs/solana-verifiable-build):

```bash
solana-verify verify-from-repo -um \
  --program-id EjpziSWGRcEiDHLXft5etbUtcJiZxEttkwz1tqiuzzWU \
  https://github.com/4626fun/creator-share-hook-verifiable \
  --library-name creator_share_hook \
  --cargo-build-sbf-args="--tools-version v1.52"
```

The `--tools-version v1.52` pin is required — the deployed binary was built with
platform-tools v1.52, and other tools versions produce a different (non-matching) hash.

## Layout

- `src/` — program source (`declare_id!` is the deployed program ID)
- `src/instructions/` — transfer-hook execute, entry recording, fee settlement, winner recording
- `src/state/` — `CreatorConfig`, `PendingEntries`, `WinnerRecord` PDAs
