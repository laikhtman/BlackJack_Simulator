# Blackjack software evaluation and fork decision

Reviewed 18 October 2026. FreeBJ is the selected simulation foundation. Its Rust
library, existing early surrender, finite shoes, and MIT license make it a useful
base for local changes. It needs a separate COD model rather than a preset claiming
that every detail of the real table is already known.

## Candidates

| Project | Language and declared license | Research role | Limitation for COD |
| --- | --- | --- | --- |
| [FreeBJ](https://github.com/kevin-lesenechal/freebj) | Rust, MIT | Selected simulation engine; configurable main-game rules and JSON output | Standard shoes use penetration; original early surrender also permits an Ace; original card type loses rank and suit |
| [possibly-wrong blackjack](https://github.com/possibly-wrong/blackjack) | C++, GPL 3 or later | Independent reference for exact dealer probabilities and action EVs | Standard action API conditions on no dealer blackjack and exposes late surrender; early surrender and dealer settlement need adaptation |
| [Blackjack Strategy Simulator](https://github.com/AttackingOrDefending/Blackjack-Strategy-Simulator) | Python, AGPL 3 or later | Strategy generation, deviations and Monte Carlo experiments | Author documents split approximations and limits to strategy generation; validate independently |
| [mhluska blackjack simulator](https://github.com/mhluska/blackjack-simulator) | TypeScript, MIT | Library, CLI and browser practice app; useful if a web interface becomes important | Simulation rather than an exact probability oracle; COD-specific rules still need checking |

[Gymnasium Blackjack](https://gymnasium.farama.org/environments/toy_text/blackjack/)
is excluded as the main engine: its infinite deck and hit/stand-only actions model
a simplified reinforcement-learning problem.

An exact engine computes probabilities within its rule and strategy assumptions.
Monte Carlo estimates results from random play and needs uncertainty estimates.
Neither a repository's popularity nor agreement with one overall edge proves correctness.
Licenses above identify the upstream terms; the selected fork preserves its MIT notice.

## Fork provenance

- Fork: https://github.com/laikhtman/BlackJack_Simulator
- Parent: https://github.com/kevin-lesenechal/freebj
- Initial upstream commit: `91e9294d51e87ed36d124cfe058571dd9c05a342`
- Upstream commit date: 8 September 2025; package version: 1.2.2.
- Original executable and library retain the `freebj` name.

## Implemented adaptations

The new `cod-research` executable calls the upstream round engine through a physical
six-deck pack adapter. Physical ranks and suits remain available for side bet
settlement while main play receives blackjack values. A new surrender policy
enforces the Ace exclusion inside the engine, including forced surrender requests.

The runner explicitly requests dealer mode and doubling after splitting, validates
one through six occupied boxes and at most three owned boxes, and applies the
observed 50 times opening-bet limit. It fixes the split maximum at three hands and
declines insurance under its current non-counting strategy. It reports all assumptions.

## Further audit points

The inherited basic strategy is not proven optimal for this exact finite-pack COD
scenario. The original `freebj` CLI is separate and retains upstream defaults.
Its `--bet-max-tc` parser appears to assign `bet_neg_tc` rather than `bet_max_tc`
in `src/options.rs`; the COD runner does not use it. Original `hand_result` uses a
fixed 1.5 natural award despite the library rules field; COD also uses 3:2.
These observations are audit leads, not a completed verification of upstream correctness.
