# Blackjack research sources and provenance

Recorded 18 October 2026. User observations govern the target table. Published
guidance supplies other rules but may cover more than one table configuration.
Calculated results and modeling assumptions are labeled separately.

## User observations

| Identifier | Observation supplied in this project conversation |
| --- | --- |
| U1 | Limassol venue; dealer hits 16 and stands 17; early surrender against every non-Ace upcard remains valid despite eventual dealer blackjack; Ace offers insurance and no surrender |
| U2 | Maximum opening bet is 50 times minimum; example EUR 50 to EUR 2,500 |
| U3 | Continuous shuffling machine loaded with six decks |
| U4 | Six table boxes; each patron may play up to three |

The number of decks is resolved by U3. Earlier notes treating it as unknown are
superseded. The generic guide's range of deck counts does not override this observation.

## Casino publications

- **S1** [Official game guide](https://www.cityofdreamsmed.com.cy/assets/public/2025-12/HOWTOPLAY_V13_Digital_0.pdf).
  Printed pages 6 and 7 cover main blackjack; 8 and 9 cover associated side bets.
  General rules support the published-rules table; side bet edges are rounded
  comparison targets, not proof of an exact main-game ruleset.
- **S2** [Official blackjack page](https://www.cityofdreamsmed.com.cy/en/ts-how-play/blackjack).
  Current paytables, splitting terms, patron box limit, main-wager prerequisite and
  maximum per box. The page does not establish the observed 50 times ratio or CSM model.
- **S3** [Earlier Cyprus Casinos guide](https://www.cypruscasinos.com/sites/g/files/cityofdreamsmt/files/2023-07/HOWTOPLAY%20for%20website.pdf).
  Used only to clarify pair category definitions. Its historical 21+3 game is not
  the current COD 21+ total-based game.

## Software sources

- [FreeBJ source and MIT license](https://github.com/kevin-lesenechal/freebj/tree/91e9294d51e87ed36d124cfe058571dd9c05a342).
  Inspect `src/round.rs`, `game_rules.rs`, `card.rs`, `basic_strategy.rs`,
  `shoe/standard_shoe.rs`, `betting.rs`, and `options.rs` for the fork assessment.
- [Possibly wrong blackjack](https://github.com/possibly-wrong/blackjack).
  Its `src/blackjack.h` documents action EV conditioning and surrender interfaces;
  its readme explains split strategy models and GPL terms.
- [Python Blackjack Strategy Simulator](https://github.com/AttackingOrDefending/Blackjack-Strategy-Simulator).
  Readme describes rule coverage, approximation limits and AGPL terms.
- [TypeScript blackjack simulator](https://github.com/mhluska/blackjack-simulator).
  Readme describes library, CLI and browser use; repository declares MIT.
- [Gymnasium Blackjack documentation](https://gymnasium.farama.org/environments/toy_text/blackjack/).
  Describes the simplified environment used to exclude it as the primary engine.

## Recording future evidence

For a changed rule, record the table/limit, observation date and source, then update
the rules and affected experiments. Store code revision, full command, output,
seed and assumptions with each published research result. Do not silently convert
unconfirmed dealer settlement, side limits or CSM behavior into casino facts.
