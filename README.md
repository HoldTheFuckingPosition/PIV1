# PIV1 Handoff Pack v0.2

This pack is the authoritative starting point for the dedicated PIV1 development chat and Codex work on the VPS.

Files:

- `PIV1_MASTER_SPEC_v0.2.md`: consolidated product, economic, technical, security, and deployment specification.
- `PIV1_DECISIONS_v0.2.md`: compact status register using CONFIRMED / PROVISIONAL / OPEN / HISTORICAL / REJECTED.
- `PIV1_CODEX_EXECUTION_PLAN_v0.2.md`: phased development plan with acceptance gates.
- `PIV1_NEW_CHAT_PROMPT.md`: exact prompt to paste into the new PIV1 development chat.

Important:

- No mainnet deployment, authority transfer, real-fund movement, or secret handling is authorized by this pack.
- Current external addresses, program versions, fees, and protocol constraints must be reverified against official sources before use.
- The new development chat must not reopen confirmed product decisions unless a verified technical incompatibility is found.

## v0.2 changes

- Confirmed JitoSOL direct deposit and delayed direct withdrawal; no Jupiter/DEX core path.
- Confirmed six guardians with 4/6 authority and current KIF rules.
- Added explicit insufficient-distribution behavior: no snapshot, no lock, yield keeps accumulating.
- Added a 24-hour anti-spam retry cooldown after a valid insufficient attempt.
- Confirmed one active distribution at a time for V1.
- Confirmed a recyclable operational rent reserve excluded from principal and yield.
