# City of Dreams Limassol blackjack rules

Working specification as of 18 October 2026. Observations describe the table
reported by the project owner. Published rules provide additional details;
unresolved items remain scenario inputs. Evidence identifiers are in [sources](sources.md).

## Observed table rules

| Item | Specification | Evidence |
| --- | --- | --- |
| Venue | City of Dreams Mediterranean, Limassol, Cyprus | U1 |
| Decks | Six standard decks | U3 |
| Shuffling | Continuous shuffling machine | U3 |
| Dealer | Draw through 16; stand on 17 | U1 |
| Early surrender | Available against dealer 2 through ten-valued cards; loss is half the initial wager even if dealer subsequently has blackjack | U1 |
| Dealer Ace | Insurance available; surrender unavailable | U1 |
| Opening bet spread | Maximum equals 50 times table minimum | U2 |
| Example limits | EUR 50 minimum and EUR 2,500 maximum; an example, not a universal minimum | U2 |
| Table capacity | Six player boxes | U4 |
| Patron capacity | At most three boxes per patron | U4, S2 |

Insurance is an optional additional wager. Ordinary playing actions against an
Ace remain available, subject to the normal rules. Early surrender is an initial
decision, before hits, doubles, or splits; the implementation does not offer it again.

## Additional published rules

The [official guide](https://www.cityofdreamsmed.com.cy/assets/public/2025-12/HOWTOPLAY_V13_Digital_0.pdf),
printed pages 6 and 7, supplies these details (S1):

| Item | Published rule |
| --- | --- |
| Soft 17 | Dealer stands, including Ace and six |
| Natural blackjack | Pays 3:2; two naturals push |
| Other wins | Pay 1:1; equal totals push |
| Double | Any initial two cards except a natural; one extra card |
| Splits | At most three hands |
| Split aces | One additional card per hand; an ensuing 21 is not a natural |
| Insurance | Up to half the original stake; pays 2:1 |

S2 explicitly limits aces to one split. Its table maximum applies per box.
These published rules supplement rather than resolve every detail of U1.

## Questions still open

1. Is doubling after splitting permitted?
2. Does the dealer take a second card only after player actions, or use a hole card
   and check for blackjack? In the former case, do split/double additions all lose
   to dealer blackjack, or are some returned?
3. What machine model and card-return procedure are used? How long are discards
   withheld and how are they mixed back in?
4. What are the side bet minima, maxima, permitted increments, and relation to the
   main bet? Does the table maximum constrain extra split/double exposure?
5. Do these rules vary between tables or limits?

## Model boundaries

Six decks means 312 physical cards, not an infinite deck. In the current reference,
cards leave the pack as they are dealt during a round, and all return before the
next round. This deliberately chosen idealization preserves finite-card effects
within and across the occupied boxes of that round. It does not establish the
real machine's independence between rounds or its return delays.

The research runner supports one through six occupied positions in dealing order.
The patron selects one through three distinct occupied positions. Other occupied
boxes use the same fixed main wager and playing strategy as a modeling assumption.
All boxes share a dealer and the same physical cards. Chips added for splitting
and doubling are tracked; they may create more exposure than the opening stake.

The 50 times limit constrains each opening main wager. It does not establish that
a patron should use that spread, or that increasing a wager changes card probabilities.
