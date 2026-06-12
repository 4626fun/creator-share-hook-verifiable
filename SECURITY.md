# Security policy

## Reporting a vulnerability

If you discover a security issue in the 4626 `creator_share_hook` program
(`EjpziSWGRcEiDHLXft5etbUtcJiZxEttkwz1tqiuzzWU` on Solana mainnet-beta),
please report it privately:

- Email: hello@4626.fun

Please do not open public GitHub issues for security vulnerabilities and do not
exploit issues on mainnet beyond the minimum needed to demonstrate impact.

We will acknowledge reports as quickly as possible and coordinate a fix and
disclosure timeline with you.

## Scope

- This program (Token-2022 transfer hook: buy detection, lottery entry recording,
  fee harvesting, winner notification) and its on-chain PDAs
  (`CreatorConfig`, `PendingEntries`, `WinnerRecord`).

## Verified build

The deployed binary is reproducible from this repository — see the
[README](./README.md) for the `solana-verify` command and expected program hash.
