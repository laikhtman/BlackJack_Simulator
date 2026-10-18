# Blackjack research methods and next steps

The goal is to measure probabilities, action values and combined wager outcomes
for the observed COD Limassol game. Rule changes must stay separate from strategy
changes and from machine assumptions so comparisons remain interpretable.

## Research questions

1. What are dealer final-outcome probabilities under the chosen finite-card model?
2. For each initial player hand and dealer upcard, which available action maximizes
   expected net return, including early surrender against non-Aces?
3. What are the mean, variance and outcome distribution of main and side wagers,
   for one through three owned boxes at different occupancies?
4. How sensitive are results to unresolved dealer settlement, doubling after splits,
   and machine card-return rules?
5. How do lawful opening wagers within the 50 times range scale monetary outcomes
   and session risk for a stated bankroll and horizon?

## Current reference model

The pack has six decks, resets between rounds, and loses cards without replacement
within a round. Each run records its seed, rounds, occupancy, owned positions,
wagers and explicit rule scenarios. One sequential RNG stream is used. Reproducing
an experiment also requires its code revision, Rust version and Cargo.lock.

This is an idealized full-pack reference for a CSM game. Ordinary shoe penetration
and true-count-based betting are not the baseline. Machine return delays and mixing
may require a different model once the hardware and procedure are known.

## Expected return and uncertainty

Returns are net profit: a normal loss is -1 initial unit, a push 0, a normal win +1,
a natural +1.5, and surrender -0.5. Wins return the original stake separately.
Doubling and splitting add exposure. Insurance is a distinct wager, not a surrender
alternative. The current research strategy declines insurance.

An early surrender decision compares -0.5 with the unconditional EV of continuing,
including possible dealer blackjack. Do not substitute an action EV already
conditioned on no dealer blackjack. Win frequency alone does not determine EV.

The runner reports EUR net per round, sample standard deviation, standard error,
and a normal-approximation 95 percent interval for the mean. The interval is not a
prediction interval or a promise about a session. Rare side awards need large
samples; exact enumeration is the preferred side bet expectation reference.
Current intervals assume the implemented independent round resets, not a measured CSM.

Owned-box totals are aggregated before their variance is calculated. Do not add
box variances as if their common dealer and cards were independent. A combined
EUR return must not be labeled a house-edge percentage without stating its wager denominator.

## Validation already encoded

- Payout examples cover every side category, overlap priority, face ranks and Aces.
- Exact category counts exhaust the six-deck draw space; pair counts are checked
  against independent combinatorics and edges against the guide's rounded figures.
- Dealer blackjack cannot cancel an already accepted early surrender; an Ace
  prohibits surrender even through the forced-action API. Insurance still works.
- Fixed-seed runs reproduce results; owned totals reconcile; enabling side wagers
  leaves the sampled main play unchanged.
- Fixed-card rounds check six independent box results and independent split budgets,
  guarding against an inherited owner-ID bug fixed in this fork.
- Opening limits, invalid money values, occupied positions and owned-box limits
  are checked. CI builds and tests on Windows and Linux.

Passing these checks establishes implemented behavior, not globally optimal strategy
or fidelity to an unidentified shuffler.

## Next implementation steps

1. Confirm doubling after splitting and dealer card/blackjack settlement. Until then,
   compare clearly labeled scenarios rather than announce one casino house edge.
2. Add an independent exact main-game reference, starting with dealer distributions
   and non-split hand actions; validate split models and surrender timing separately.
3. Derive and verify a COD-specific strategy against that reference.
4. Add machine return models only with evidence about the actual procedure.
5. Extend the separate funded `cod-bankroll` research runner into the interactive
   app, with configurable starting bankrolls, targets, confirmed chip increments,
   exposure limits and session curves. The interactive fixed-bet runner still uses
   unlimited bankroll. See [the initial target study](bankroll-target-study.md).
6. Extend to heterogeneous other-player strategies and wagers if needed.

The completed first stage is documented rules, a reproducible fork, side bet
evaluation, exact side probabilities and combined simulation scenarios.
