# Bankroll target study for EUR 10000 to EUR 100000

The strongest tested policies reached EUR 100000 in roughly 8.8 to 9.4 percent
of sessions. Roughly 90.6 to 91.2 percent of those leading-policy sessions ended
below the EUR 50 table minimum. This objective favors a volatile attempt with a
high failure rate; it does not identify a reliable way to multiply the bankroll.

The study compares named wager policies under inherited basic strategy. It does
not prove a globally optimal policy over all wagers, player decisions, side bets,
machine behavior or stopping times. Differences between several leading policies
are within sampling uncertainty. Actual dealer settlement and doubling after
splitting still need confirmation.

## Experiment

- Starting bankroll EUR 10000; target total bankroll EUR 100000, meaning EUR 90000 profit.
- Opening main wagers EUR 50 through EUR 2500 per owned box, using a research
  grid of EUR 50 increments. Actual permitted increments remain unconfirmed.
- Six occupied boxes with one or three owned boxes, depending on the policy.
- A uniformly mixed six-deck pack resets each round. Physical cards are drawn
  without replacement within the round. This is the existing CSM reference,
  not measured machine firmware or a model of return delays.
- S17, 3:2 naturals, early surrender excluding Ace, maximum three split hands,
  one card to each split Ace, and declined insurance.
- Both hole-card/peek and later-second-card/all-active-bets-lost scenarios,
  crossed with doubling after splitting allowed or forbidden.
- Stop after a settled round at or above the target, below EUR 50, or after
  10000 rounds. The time limit was not supplied by the user; this explicit cap
  makes the initial study finite. Unfinished sessions are not labeled ruin.
- The initial screen used 500 sessions for each of 13 policies in each of four
  rule scenarios, then 10000 fresh sessions for three selected policies.
- The common validation uses 50000 fresh sessions for each of eight policies
  in each of four rule scenarios: **1600000 validation sessions**.

Exact counts, seeds and provenance are in
[the summary dataset](../research/results/bankroll-target-summary.json).
Full-precision runner JSON is available in the
[validation workflow artifacts](https://github.com/laikhtman/BlackJack_Simulator/actions/runs/39152825341).
Code revision: `2e7922e6382eccaf11743103d80805ba52d8b535`.

## Leading point estimates in each rule scenario

| Dealer scenario | Double after split | Highest observed policy | Hit target | Marginal 95 percent Wilson interval | Below EUR 50 |
| --- | --- | --- | ---: | --- | ---: |
| Hole card and peek | Yes | max_with_target_gap_1_boxes | 9.444% | 9.191 to 9.703% | 90.556% |
| Hole card and peek | No | fraction_25_percent_one_box | 9.004% | 8.756 to 9.258% | 90.996% |
| Later second card, all active bets lost | Yes | fraction_25_percent_one_box | 9.274% | 9.023 to 9.531% | 90.726% |
| Later second card, all active bets lost | No | flat_2500_one_box | 8.840% | 8.594 to 9.092% | 91.160% |

Each row's leading policy has zero unfinished sessions in its 50000-session
validation sample. The intervals describe Monte Carlo sampling error within the
selected model, not casino-rule uncertainty or simultaneous confidence over all
comparisons. Selecting the largest estimate does not establish a uniquely best
true policy.

## Common policy comparison

Entries are probabilities of reaching the target within 10000 rounds.

| Wager policy | Peek with DAS | Peek without DAS | Later card with DAS | Later card without DAS |
| --- | ---: | ---: | ---: | ---: |
| Up to EUR 1000, one box | 8.630% | 7.750% | 7.814% | 6.948% |
| Up to EUR 2500, one box | 8.974% | 8.910% | 8.804% | 8.840% |
| Up to EUR 2500 per box, three boxes | 8.746% | 8.894% | 8.848% | 8.502% |
| 5 percent bankroll, one box | 8.470% | 7.580% | 7.964% | 7.244% |
| 10 percent bankroll, one box | 9.272% | 8.734% | 8.834% | 8.322% |
| 25 percent bankroll, one box | 9.272% | 9.004% | 9.274% | 8.528% |
| Maximum with target gap cap, one box | 9.444% | 8.866% | 8.846% | 8.632% |
| Maximum with target gap cap, three boxes | 9.016% | 8.886% | 8.682% | 8.680% |

DAS means doubling after splitting. Lower fixed wagers and a capped Martingale
were screened but were not included in this common validation. The initial
Martingale screen reached the target in 3.4 to 4.8 percent of sessions within
the cap, with some unfinished sessions. That small screen is not an estimate
of its unlimited-time success probability.

The initial EUR 50 Pair-Fect and EUR 50 COD 21+ exploratory cases each had zero
target hits in 500 sessions per rule scenario. Zero observed hits is not zero
probability: the marginal Wilson upper bound is about 0.762 percent. Those side
wager limits are unconfirmed, and the screen does not prove that every possible
side-wager allocation is inferior for a target-hit objective.

## What the wager policies actually do

An up-to-EUR-2500 policy requests EUR 2500 per box while funds permit. It lowers
the wager to fit the available bankroll on the EUR 50 grid. A three-box policy
reduces owned occupancy if it cannot fund the minimum wagers on all three boxes.

A fractional policy requests its named percentage of the current bankroll for
one box, rounds down to the EUR 50 grid, and applies the EUR 50 minimum and
EUR 2500 maximum. Fractional policies and the target-gap policies also cap the
requested wager at the remaining target gap rounded up to EUR 50. The maximum
target-gap policy otherwise requests EUR 2500 per box.

These are opening wagers. Available cash is reserved separately for extra
splits and doubles. If the bankroll cannot fund an action, inherited basic strategy
chooses with that action unavailable. Side payouts and early-surrender refunds
become available before later decisions, as an explicit settlement assumption.
Other occupied boxes have independent funds and the same main wager and strategy.
The target is tested only after the whole round settles.

The one-box EUR 2500 policy is a robust candidate across the four modeled rule
scenarios, but it is not a proven global optimum. Its observed hit probability
ranges from 8.804 to 8.974 percent. In the later-card/no-DAS scenario, it had
4420 target hits and 45580 unplayable outcomes in 50000 sessions, with mean ending
bankroll EUR 8917.61. The 25-percent policy and target-gap policy lead in other
scenarios. Confirming the actual rules matters before choosing between them.

Average round counts include many fast failures. They are not the average time
needed to win EUR 90000. Successful-session time is a separate metric in the full
runner output. Session mean ending values are sample means, not promised returns.

## Mathematical benchmark

If settled bankroll is nonnegative and has nonpositive conditional expected
increments, the nonnegative-supermartingale maximal inequality gives a target-hit
ceiling of starting bankroll divided by target: 10000 / 100000 = 10 percent.
See the [Berkeley statement of the Ville inequality](https://www.stat.berkeley.edu/~stark/Seminars/auditCCRL21.slides.pdf).
This is a conditional mathematical ceiling, not a verified exact casino probability
or an assertion that 10 percent is achievable under table limits.

## Reproduce and limitations

The study runner is separate from the interactive unlimited-bankroll round
simulator. From a synchronized source checkout with Rust installed:

```powershell
cargo run --locked --release --bin cod-bankroll -- --dealer enhc --das no --validation-only --holdout 50000 --max-rounds 10000
```

Use `--dealer ahc` for the modeled hole-card/peek scenario and `--das yes` to
allow doubling after splits. The initial screening mode omits
`--validation-only` and accepts `--sessions`, `--holdout`, `--max-rounds` and `--seed`.

The new funded runner passed its four tests, including explicit insufficient-cash
and surrender-refund cases. The full package passed **84 tests on Windows and
Linux** at the cited revision. All four study jobs completed successfully.

Local process launching returned Access is denied, and terminal input was
rejected by automatic approval review because approvals were disabled. The
research therefore ran in GitHub Actions through the connected repository.
The local interactive app was not launched or updated for this study.
