# COD blackjack side bets

The implemented bets are Pair-Fect Pairs and COD 21+, using the
[current casino paytables](https://www.cityofdreamsmed.com.cy/en/ts-how-play/blackjack)
(S2). Both use the player's initial two cards; COD 21+ also uses the dealer upcard.
Pair-Fect's highest award uses the dealer upcard too. S1 requires a main wager
and resolves these bets before ordinary player actions.

## Payouts

Profit odds below exclude the returned winning stake. Losing wagers return nothing.
Within each bet, only the highest qualifying category pays.

| Pair-Fect category | Profit per unit |
| --- | ---: |
| Same rank and suit on all three cards | 50 |
| Player pair with the same suit | 30 |
| Player pair with the same color and different suits | 10 |
| Player pair with opposite colors | 5 |
| No pair | -1 |

A pair requires the same rank: J and Q do not qualify together. The older official
[Cyprus Casinos guide](https://www.cypruscasinos.com/sites/g/files/cityofdreamsmt/files/2023-07/HOWTOPLAY%20for%20website.pdf)
(S3) clarifies these pair definitions. Its older 21+3 paytable is not used.

| COD 21+ category | Profit per unit |
| --- | ---: |
| Same-suit 7 7 7 | 200 |
| Same-suit 6 7 8 | 100 |
| Other 7 7 7 | 50 |
| Other 6 7 8 | 30 |
| Other same-suit total 21 | 10 |
| Other total 21 | 3 |
| Total 20 | 2 |
| Total 19 | 2 |
| No qualifying combination | -1 |

Suit means all three cards share one suit. Aces use the highest blackjack total
not exceeding 21. For example, A A 9 totals 21. Category priority prevents a suited
7 7 7 from also collecting the ordinary 21 award. These interpretations reproduce
the guide's rounded overall edges; any different table instruction should be recorded.

## Exact six deck reference

The exact calculator weights every ordered rank/suit triple by the number of
remaining physical copies. There are 312 x 311 x 310 = 30,079,920 equally likely
ordered physical draws from a full uniformly mixed pack. It draws without replacement.

| Bet | Player EV per unit staked | House edge | Return to player |
| --- | ---: | ---: | ---: |
| Pair-Fect Pairs | -0.0537288663001763 | 5.3728866300% | 94.6271133700% |
| COD 21+ | -0.0530538645049588 | 5.3053864505% | 94.6946135495% |

These are calculated reference results, not measurements of the physical CSM.
They agree with S1's rounded figures. JSON output includes exact integer counts,
category probabilities, standard deviations, and covariance between the two bets.

Independent pair counts provide a check: suited triples have 52 x 6 x 5 x 4
outcomes; remaining suited pairs have 52 x 6 x 5 x 306; prime pairs have
52 x 6 x 6 x 310; mixed-color pairs have 52 x 6 x 12 x 310. All other outcomes lose.

## Combined research

The simulator evaluates side bets from the exact physical cards dealt to each
owned box in the main game. Side awards remain paid if the player subsequently
surrenders, busts, splits, or loses to dealer blackjack. They do not use hit cards
or the dealer's later cards, and splits do not create additional side wagers.

Each owned box reports separate main, Pair-Fect, COD 21+, and combined net results.
The patron's round total adds actual correlated results across owned boxes.
For expectation, side wager amounts multiply the per-unit EVs above. Variance
must include covariance; the implementation measures combined round results directly.

Unknown side bet table limits are not invented or inferred from the main 50 times
ratio. A zero side wager disables that bet. The runner requires a valid main wager.
