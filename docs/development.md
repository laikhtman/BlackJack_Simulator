# Development and research commands

## Repository

This directory is a full clone of the GitHub fork, including upstream history.
`origin` points to https://github.com/laikhtman/BlackJack_Simulator.
The initial parent revision is recorded in [software evaluation](software-evaluation.md).
No separate upstream remote is required to build. The parent can be fetched later
by URL, or configured with `git remote add upstream https://github.com/kevin-lesenechal/freebj.git`.

The project retains its MIT license and upstream attribution. COD-specific modules
are `src/cod/` and `src/bin/cod-research.rs`. The general engine remains in the
upstream locations, with an additional `EarlySurrenderExceptAce` policy.

## Toolchain and checks

Successful CI runs provide `cod-research-windows-latest` and
`cod-research-ubuntu-latest` executable artifacts. The Windows artifact can be
downloaded into `artifacts/windows`; it runs without a local Rust installation:

```powershell
.\artifacts\windows\cod-research.exe exact
```

Rust and Cargo were not found in the initial local environment. CI installs Rust
1.90.0 and runs on Windows and Linux. With that toolchain available locally:

```powershell
cargo test --locked --all-targets
cargo build --locked --release --bins
```

Use `--bin` because the repository now contains two executables. The original
`freebj` commands describe a general shoe game; use `cod-research` for the new model.

## Exact side probabilities and individual hands

```powershell
cargo run --locked --release --bin cod-research -- exact
cargo run --locked --release --bin cod-research -- hand JH JH JH
cargo run --locked --release --bin cod-research -- hand 6H 8H 7H
```

Card arguments are player first, player second, dealer upcard. Use A, 2 through 10,
J, Q or K followed by C, D, H or S. Repeated identical rank/suit cards are possible
in six decks. Each result is net profit in units of that side wager, not total cash returned.

## Combined simulation

```powershell
cargo run --locked --release --bin cod-research -- simulate --rounds 100000 --seed 42 --occupied-boxes 6 --owned-boxes 1,3,6 --table-minimum 50 --main-bet 50 --pair-fect-bet 5 --cod21-bet 5 --dealer-mode enhc-all-bets --double-after-split yes
```

This example chooses two unconfirmed rules as a scenario. `ahc-peek` models a
dealer hole card and blackjack resolution before normal player actions;
`enhc-all-bets` models a later second card and loss of all active split/double
wagers to dealer blackjack. Accepted early surrender is already settled in either
case. Original-bets-only settlement is not implemented. Choose doubling after
splitting with `yes` or `no`; both options are required rather than silently defaulted.

Occupied positions are numbered 1 through the requested occupancy, in dealing
order. `--owned-boxes` accepts distinct comma-separated positions, maximum three.
Stakes apply separately to every owned box. Other positions use the same main
stake and strategy, but their profit is excluded from the patron's report.

The example monetary defaults are conveniences, not a claim about the table's
current minimum or side bet limits. The opening main bet must be between the
configured minimum and 50 times that minimum. Side bets default to zero.
All results assume unlimited bankroll; they include added main wagers from normal
splitting and doubling but do not enforce unconfirmed limits on that added exposure.

The JSON report includes per-box components, a correlated patron total, uncertainty
estimates, seed, complete configuration and model limitations. Redirect stdout to
save it; progress or errors use stderr. Keep the full command and code revision
alongside the report. Rare events require much more than a short smoke run.

## Further work

See [research plan](research-plan.md) for unfinished rule confirmation, exact main
strategy, machine behavior and session bankroll work. Do not interpret the inherited
strategy as a proven optimal COD strategy or the round reset model as actual machine firmware.
