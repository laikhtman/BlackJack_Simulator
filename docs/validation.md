# Initial simulator validation

Verified 18 October 2026 at code revision
`48729ff3a89b753ec355d84f5213eb43beaebaa0`.
The [successful CI run](https://github.com/laikhtman/BlackJack_Simulator/actions/runs/39149816865)
used Rust 1.90.0 and the committed Cargo.lock on Windows and Linux.

| Check | Windows | Linux |
| --- | --- | --- |
| Library tests | 58 passed | 58 passed |
| Original CLI tests | 3 passed | 3 passed |
| COD research tests | 8 passed | 8 passed |
| Inherited fixed-shoe integration tests | 5 passed | 5 passed |
| Release build of both binaries | Passed | Passed |
| Exact side bet calculation | Passed | Passed |
| Six occupied boxes and three owned boxes smoke simulation | Passed | Passed |

The downloaded Windows executable was also run in the local project directory.
Its SHA256 is `0d7ca95f682376c9fe27d35c30fdca15d883459bcc69bb3564e05b68a936428a`.
It is available locally at `artifacts/windows/cod-research.exe`; generated executable
artifacts are ignored by Git and remain downloadable from the linked CI run.

## Saved outputs

- [Exact six-deck side bet probabilities](../research/results/side-bets-six-decks.json)
- [Six-box smoke result](../research/results/smoke-six-boxes.json)
- [Commands and artifact provenance](../research/results/manifest.json)

The exact probabilities use a uniformly mixed complete pack without replacement.
The smoke run is only a functional check, with seed 42 and 1,000 rounds; its
sample return is not a casino edge estimate. It selects ENHC with all active
split/double wagers lost to dealer blackjack and permits doubling after splitting.
Those selections remain scenario assumptions, not confirmed table observations.

The modeled CSM resets the full pack between rounds. The actual machine's return
delays and mixing remain unknown, and inherited basic strategy is not proven
optimal for every COD situation. See [open rules](casino-rules.md) and
[research methods](research-plan.md) before using the runner for further studies.
