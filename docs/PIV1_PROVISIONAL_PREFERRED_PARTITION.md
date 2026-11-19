# Provisional preferred-source partition

The scope is an already prepared delayed-withdrawal round with a fixed, partly
or wholly unassigned target. Preparation, routing and accounting retain the
qualified release behavior. This is an undeployed candidate for a founder
decision.

Let R be the unassigned target, C the maximum safe input from the authenticated
selected source and M the greater of the round's stored floor and the exact
current source floor. Use min(R,C) whenever it fills R or leaves a remainder of
at least M. Existing liquidity, price, funding and accounting checks apply.

Consider an exception only when all of the following hold.

1. The pool identifies the current selected active validator as preferred.
   A distinct, canonical active alternative is gated by the pinned SPL
   preferred-source limit before this leg.
2. Maximum fill itself is at least M and would leave a positive remainder
   smaller than M. The first amount is derived as R minus M, at least M
   and strictly smaller than maximum fill. A caller supplies source identities
   and never supplies either amount.
3. Use exact SPL ceil fee and floor redemption to project the pool ratio and
   the preferred list record after that amount. The remaining preferred
   balance must be at or below the SPL threshold applicable to the specified
   alternative's Stake metadata and projected pool ratio. The alternative
   must have capacity for all M units under the new ratio and must pass the
   actual Stake floor, which cannot be assumed static at a different state.
4. Require room for both legs, two standard operational rent advances, the
   existing slippage bound and protected residual and round-floor checks.
   After the first CPI verify the actual preferred record, pool, mint, source
   and destination states. Check the unchanged alternative against the
   observed ratio and preferred record before committing. The second leg
   reauthenticates the current pool, list, epoch, Stake source and budgets.
   A later external update can invalidate the witness and requires a fresh
   valid second call.

For example, when a zero-fee one-to-one pool has R = 2M + 1 and C = M + 2,
the candidate cut is M + 1. The second leg is M. The cut is admitted only if
the resulting preferred balance satisfies upstream exhaustion and the
alternative is currently feasible. The program derives the same rule for
other figures. The existing instructions and account layouts retain their
existing use. The prototype permits one optional read-only alternate Stake
account on the exceptional first initiation. The alternate's actual vote
and canonical list row are established on chain. Normal initiation still
uses the ordinary account shape and full maximum fill. Submit the two legs
as separate transactions so the alternative remains read-only in the first
message. Solana grants effective account privileges across that whole message.

The fixed target and custody roles stay fixed. The same checked transition
accounts for both legs, with exact consumed input, manager fee, mint burn,
delegated native stake, vault debits, rent, counters and distinct sequential
leg PDAs. Remaining capital, HWM and the configured round minimum remain
protected. The additional path performs bounded exact arithmetic, at most one
alternate lookup in the authenticated populated list and no additional CPI.
