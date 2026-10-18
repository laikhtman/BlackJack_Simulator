# Interactive blackjack research app

## Launch on Windows

Double-click **Launch Blackjack.cmd** in the project folder. The launcher sets the
working directory so saved results go into this project. It opens the downloaded
executable at `artifacts/windows/cod-research.exe`, or a local release build if the
downloaded executable is absent.

Alternatively, open PowerShell in `C:\General\BlackJack_Simulator` and run:

```powershell
.\artifacts\windows\cod-research.exe
```

The same menus can be opened explicitly with `cod-research.exe interactive`.
Numbered commands work in ordinary Windows consoles, Windows Terminal and Linux
terminals. Color is used where the terminal supports it; setting `NO_COLOR`
disables color. No extra packages are needed for the downloaded executable.

## Menus

| Choice | What it does |
| --- | --- |
| 1 Configure simulation | Guides you through rounds, occupied/owned positions, wagers, rule scenarios and seed; shows a review before using the setup |
| 2 Run current simulation | Runs the selected configuration with a progress bar in an interactive terminal, then shows per-box and patron results |
| 3 Exact side bet odds | Shows category probabilities, house edges and return to player for both side bets under the full six-deck reference |
| 4 Check a three-card hand | Evaluates your two cards and the dealer upcard, with an example stake, showing net profit and cash returned |
| 5 Save or load setup | Saves a reusable configuration or loads and validates one for review |
| 6 Rules and model notes | Summarizes known rules, open questions and modeling assumptions |
| 0 Exit | Closes the app |

Press Enter to accept a displayed default. Type `q` or `back` to cancel the current
screen; at the main menu this exits. Cancelling setup leaves the previous setup
unchanged. An invalid entry shows an explanation and lets you try again.

Round counts accept `10k`, `100k` and `1m`. Interactive runs support 2 through
100 million rounds. Positions are occupied boxes in dealing order, from 1 through
the selected occupancy. A patron may select up to three distinct positions, such
as `1,3,6`. Stakes apply separately to every owned position; 0 disables a side bet.

## First simulation

Choose 1, start with a short round count such as `10k`, and choose the occupied
positions and your boxes. Enter the table minimum and per-box wagers. The EUR 50
default is the previously supplied example, not a universal table minimum.

Select dealer settlement and doubling after splitting explicitly. Neither has a
default on the first setup because the actual table rules are still unconfirmed.
Review the opening stake across all your boxes, then enter `yes` to use the setup.
Choose 2 from the main menu to run it. You can change the setup or rerun it from
the menu without writing another command.

The same seed and configuration reproduce the same results with the same engine
revision and toolchain. Ctrl+C stops a long run by closing the process; unfinished
results are not exported.

## Results and exports

Simulation tables show separate main-game, Pair-Fect, COD 21+ and combined totals
for each owned box. Negative numbers are losses. The patron summary combines the
actual correlated results across your boxes and shows the mean per round, standard
deviation and approximate 95 percent interval for the mean. That interval is not
a prediction of the range of possible session results.

After viewing a result, press Enter to return to the menu or `s` to export JSON.
Choose a filename or accept the timestamped default under `research/interactive/`.
Saving never overwrites an existing file. An error returns you to the main menu
with your setup still available. These generated files are excluded from Git.

Saved setups also use JSON in the same default directory. To reuse one, choose
5, then 2, paste its path, review the values and accept it. Loading a setup does
not turn its selected rule scenarios into confirmed casino facts.

## Cards and payouts

Enter three space-separated cards in this order: your first card, your second
card, dealer upcard. For example, `6H 8H 7H` is a suited 6 7 8 combination.
Use A, 2 through 10, J, Q or K with C clubs, D diamonds, H hearts or S spades.
Identical rank/suit cards are possible because there are six decks.

The hand calculator evaluates side bets rather than recommending the main-game
action. Profit odds exclude the returned winning stake: a EUR 5 wager paying
200 to 1 produces EUR 1,000 profit and EUR 1,005 cash returned.

## Model and advanced commands

The interface uses the existing research engine and adds no changes to the payout
tables or probability model. It retains the documented full-pack reset between
rounds, finite-card draws within a round, unlimited bankroll, declined insurance
and inherited strategy. Actual CSM return delays and mixing remain unknown.
See [casino rules](casino-rules.md) and [research methods](research-plan.md).

Existing `exact`, `hand` and `simulate` command-line subcommands continue to emit
JSON for automation. For source builds, use:

```powershell
cargo run --locked --release --bin cod-research -- interactive
```

If the launcher reports that the executable is missing, download the Windows
artifact from a successful [GitHub Actions run](https://github.com/laikhtman/BlackJack_Simulator/actions)
into `artifacts/windows`, or build the source. The Linux artifact runs the same menus.
