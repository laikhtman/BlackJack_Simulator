//! Portable, line-oriented terminal UI. No raw keyboard mode or terminal dependencies.
use super::side_bets::{self, ExactBet, PhysicalCard};
use super::{DealerMode, SimulationConfig, SimulationReport, simulate_with_progress};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const RULE: &str = "----------------------------------------------------------------------------";

#[derive(Serialize, Deserialize)]
struct SavedSetup {
    schema_version: u32,
    config: SimulationConfig,
}

struct Ui<R, W> {
    input: R,
    output: W,
    color: bool,
    terminal: bool,
    config: Option<SimulationConfig>,
}

pub fn launch() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let terminal = stdin.is_terminal() && stdout.is_terminal();
    let ansi = !cfg!(windows) || std::env::var_os("WT_SESSION").is_some()
        || std::env::var_os("TERM").is_some();
    Ui {
        input: stdin.lock(), output: stdout.lock(),
        color: terminal && ansi && std::env::var_os("NO_COLOR").is_none(), terminal,
        config: None,
    }.run()
}

impl<R: BufRead, W: Write> Ui<R, W> {
    fn title(&mut self, title: &str) -> io::Result<()> {
        writeln!(self.output, "\n{RULE}")?;
        if self.color { write!(self.output, "\x1b[1;36m")?; }
        writeln!(self.output, "  {title}")?;
        if self.color { write!(self.output, "\x1b[0m")?; }
        writeln!(self.output, "{RULE}")
    }

    fn message(&mut self, text: &str) -> io::Result<()> {
        writeln!(self.output, "  {text}")
    }

    /// q/back cancels a screen. EOF exits cleanly even if it arrives mid-wizard.
    fn prompt(&mut self, label: &str, default: &str) -> io::Result<Option<String>> {
        if default.is_empty() { write!(self.output, "  {label} > ")?; }
        else { write!(self.output, "  {label} [{default}] > ")?; }
        self.output.flush()?;
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Input closed"));
        }
        let line = line.trim();
        if line.eq_ignore_ascii_case("q") || line.eq_ignore_ascii_case("back") {
            return Ok(None);
        }
        Ok(Some(if line.is_empty() { default.to_string() } else { line.to_string() }))
    }

    fn field<T>(
        &mut self, label: &str, default: &str,
        parse: impl Fn(&str) -> Result<T, String>,
    ) -> io::Result<Option<T>> {
        loop {
            let Some(value) = self.prompt(label, default)? else { return Ok(None); };
            match parse(&value) {
                Ok(value) => return Ok(Some(value)),
                Err(error) => self.message(&format!("Please try again: {error}"))?,
            }
        }
    }

    fn run(&mut self) -> io::Result<()> {
        loop {
            self.title("COD LIMASSOL  |  Blackjack Research")?;
            self.message("6 decks  |  6 table boxes  |  up to 3 owned boxes  |  S17")?;
            self.message("Early surrender except Ace. CSM uses a full-pack round reset model.")?;
            if let Some(config) = self.config.clone() {
                self.show_setup(&config)?;
            } else {
                self.message("Setup: not configured. Start with [1] to choose your scenario.")?;
            }
            self.message("")?;
            for line in [
                "[1] Configure simulation       [2] Run current simulation",
                "[3] Exact side bet odds        [4] Check a three-card hand",
                "[5] Save or load setup         [6] Rules and model notes",
                "[0] Exit",
            ] { self.message(line)?; }
            let action = match self.prompt("Choose a number", "") {
                Ok(Some(value)) => value,
                Ok(None) => break,
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };
            let result = match action.as_str() {
                "0" => break,
                "1" => self.configure(),
                "2" => self.run_simulation(),
                "3" => self.exact(),
                "4" => self.hand(),
                "5" => self.profiles(),
                "6" => self.rules(),
                _ => self.message("Choose 0 through 6; q also exits."),
            };
            if let Err(error) = result {
                if error.kind() == io::ErrorKind::UnexpectedEof { break; }
                self.message(&format!("Could not complete that action: {error}"))?;
            }
        }
        self.message("Goodbye. Saved files remain in the location you selected.")
    }

    fn show_setup(&mut self, c: &SimulationConfig) -> io::Result<()> {
        self.message(&format!("Setup: {} rounds | occupied {} | yours {} | seed {}",
            grouped(c.rounds), c.occupied_boxes, boxes(&c.owned_boxes), c.seed))?;
        self.message(&format!("Per owned box: main EUR {:.2} | Pair-Fect {:.2} | COD 21+ {:.2}",
            c.main_bet_eur_per_box, c.pair_fect_eur_per_box, c.cod_21_eur_per_box))?;
        let opening = (c.main_bet_eur_per_box + c.pair_fect_eur_per_box + c.cod_21_eur_per_box)
            * c.owned_boxes.len() as f64;
        self.message(&format!("Opening stake per round: EUR {opening:.2} before splits/doubles"))?;
        self.message(&format!("Scenario: {} | double after split: {}",
            dealer_label(c.dealer_mode), if c.double_after_split { "yes" } else { "no" }))
    }

    fn configure(&mut self) -> io::Result<()> {
        self.title("SIMULATION SETUP")?;
        self.message("Enter accepts a shown default. Type q at any step to cancel.")?;
        self.message("EUR 50 is an example table minimum. All wagers apply per owned box.")?;
        let old = self.config.clone();
        let c = old.clone().unwrap_or(SimulationConfig {
            rounds: 100_000, seed: 42, occupied_boxes: 6, owned_boxes: vec![1],
            table_minimum_eur: 50.0, main_bet_eur_per_box: 50.0,
            pair_fect_eur_per_box: 0.0, cod_21_eur_per_box: 0.0,
            // These placeholders are never accepted without an explicit first selection below.
            double_after_split: false, dealer_mode: DealerMode::EuropeanAllBetsLost,
        });
        let Some(rounds) = self.field("Rounds (e.g. 10k, 100k, 1m)", &c.rounds.to_string(), |s| {
            let n = parse_rounds(s)?;
            if (2..=100_000_000).contains(&n) { Ok(n) }
            else { Err("Choose 2 through 100m rounds in the interactive app".into()) }
        })? else { return self.cancelled(); };
        let Some(occupied) = self.field("Occupied boxes (1-6)", &c.occupied_boxes.to_string(), |s| {
            let n = s.parse::<u8>().map_err(|_| "Enter a number from 1 to 6")?;
            if (1..=6).contains(&n) { Ok(n) } else { Err("There are six table boxes".into()) }
        })? else { return self.cancelled(); };
        let prior_boxes = if c.owned_boxes.iter().all(|&b| b <= occupied) { boxes(&c.owned_boxes) } else { "1".into() };
        let Some(owned) = self.field("Your occupied positions (e.g. 1,3,6; max 3)", &prior_boxes, |s| {
            let mut result = Vec::new();
            for value in s.split(',') {
                let n = value.trim().parse::<u8>().map_err(|_| "Use comma-separated positions")?;
                if n == 0 || n > occupied || result.contains(&n) {
                    return Err(format!("Choose distinct positions from 1 through {occupied}"));
                }
                result.push(n);
            }
            if result.len() > 3 { Err("A patron can own at most three boxes".into()) } else { Ok(result) }
        })? else { return self.cancelled(); };
        let Some(minimum) = self.field("Table minimum EUR", &format!("{:.2}", c.table_minimum_eur), |s| money(s, true))?
            else { return self.cancelled(); };
        let maximum = minimum * 50.0;
        let main_default = c.main_bet_eur_per_box.max(minimum).min(maximum);
        let Some(main) = self.field(&format!("Main bet EUR ({minimum:.2} to {maximum:.2})"), &format!("{main_default:.2}"), |s| {
            let value = money(s, true)?;
            if value >= minimum && value <= maximum { Ok(value) }
            else { Err(format!("Opening main bet must be between {minimum:.2} and {maximum:.2}")) }
        })? else { return self.cancelled(); };
        self.message("Side bets: 0 disables a bet. Actual side bet limits remain unconfirmed.")?;
        let Some(pair) = self.field("Pair-Fect stake EUR", &format!("{:.2}", c.pair_fect_eur_per_box), |s| money(s, false))?
            else { return self.cancelled(); };
        let Some(total) = self.field("COD 21+ stake EUR", &format!("{:.2}", c.cod_21_eur_per_box), |s| money(s, false))?
            else { return self.cancelled(); };
        self.message("Dealer settlement is an UNCONFIRMED research scenario:")?;
        self.message("[1] Later dealer second card; all active split/double bets lose to BJ")?;
        self.message("[2] Hole card and blackjack check before ordinary player actions")?;
        let default = old.as_ref().map(|x| match x.dealer_mode {
            DealerMode::EuropeanAllBetsLost => "1", DealerMode::AmericanPeek => "2",
        }).unwrap_or("");
        let Some(dealer) = self.field("Dealer scenario", default, |s| match s {
            "1" => Ok(DealerMode::EuropeanAllBetsLost), "2" => Ok(DealerMode::AmericanPeek),
            _ => Err("Select 1 or 2; this rule has no assumed default".into()),
        })? else { return self.cancelled(); };
        self.message("Doubling after a split is also UNCONFIRMED. Select a scenario.")?;
        let default = old.as_ref().map(|x| if x.double_after_split { "yes" } else { "no" }).unwrap_or("");
        let Some(das) = self.field("Double after split (yes/no)", default, |s| match s.to_ascii_lowercase().as_str() {
            "y" | "yes" => Ok(true), "n" | "no" => Ok(false),
            _ => Err("Enter yes or no".into()),
        })? else { return self.cancelled(); };
        let Some(seed) = self.field("Random seed (repeat a run with the same seed)", &c.seed.to_string(), |s| {
            s.parse::<u64>().map_err(|_| "Enter a nonnegative whole number".into())
        })? else { return self.cancelled(); };
        let config = SimulationConfig {
            rounds, seed, occupied_boxes: occupied, owned_boxes: owned,
            table_minimum_eur: minimum, main_bet_eur_per_box: main,
            pair_fect_eur_per_box: pair, cod_21_eur_per_box: total,
            double_after_split: das, dealer_mode: dealer,
        };
        if let Err(error) = config.validate() {
            return self.message(&format!("Setup was not changed: {error}"));
        }
        self.title("REVIEW SETUP")?;
        self.show_setup(&config)?;
        self.message("CSM reference: all six decks restored between rounds; no bankroll limit.")?;
        self.message("Insurance declined. Inherited basic strategy is not proven COD optimal.")?;
        if self.confirm("Use this setup (yes/no)")? {
            self.config = Some(config);
            self.message("Setup ready. Choose [2] from the main menu to run.")
        } else { self.cancelled() }
    }

    fn cancelled(&mut self) -> io::Result<()> {
        self.message("Cancelled. Your previous setup is unchanged.")
    }

    fn confirm(&mut self, label: &str) -> io::Result<bool> {
        Ok(self.field(label, "no", |s| match s.to_ascii_lowercase().as_str() {
            "y" | "yes" => Ok(true), "n" | "no" => Ok(false),
            _ => Err("Enter yes or no".into()),
        })?.unwrap_or(false))
    }

    fn run_simulation(&mut self) -> io::Result<()> {
        let Some(config) = self.config.clone() else {
            self.message("Choose [1] to configure a simulation first.")?;
            return Ok(());
        };
        self.title("RUNNING SIMULATION")?;
        self.show_setup(&config)?;
        self.message("Use Ctrl+C to stop a long run. Results are estimates, not a session forecast.")?;
        self.output.flush()?;
        let started = Instant::now();
        let terminal = self.terminal;
        let mut progress_error = None;
        let report = simulate_with_progress(config, |done, total| {
            if !terminal || progress_error.is_some() { return; }
            let fraction = done as f64 / total as f64;
            let filled = (fraction * 24.0) as usize;
            let elapsed = started.elapsed().as_secs_f64();
            let eta = if done > 0 { elapsed * (total - done) as f64 / done as f64 } else { 0.0 };
            let result = write!(self.output, "\r  [{}{}] {:>3.0}% | {:.1}s elapsed | ~{:.1}s left   ",
                "=".repeat(filled), " ".repeat(24 - filled), fraction * 100.0, elapsed, eta)
                .and_then(|_| self.output.flush());
            if let Err(error) = result { progress_error = Some(error); }
        }).map_err(io::Error::other)?;
        if let Some(error) = progress_error { return Err(error); }
        if terminal { writeln!(self.output)?; }
        self.message(&format!("Completed {} rounds in {:.2}s.", grouped(report.config.rounds), started.elapsed().as_secs_f64()))?;
        self.simulation_results(&report)?;
        self.offer_save(&serde_json::to_value(&report).map_err(io::Error::other)?, "simulation")
    }

    fn simulation_results(&mut self, report: &SimulationReport) -> io::Result<()> {
        self.title("RESULTS  |  Net profit in EUR")?;
        writeln!(self.output, "  {:<8} {:>12} {:>12} {:>12} {:>14}", "Box", "Main total", "Pair-Fect", "COD 21+", "Combined")?;
        for b in &report.boxes {
            writeln!(self.output, "  {:<8} {:>12.2} {:>12.2} {:>12.2} {:>14.2}", b.box_number,
                b.main_game.total_net_eur, b.pair_fect_pairs.total_net_eur,
                b.cod_21_plus.total_net_eur, b.combined.total_net_eur)?;
        }
        writeln!(self.output, "{RULE}")?;
        let patron = &report.patron_combined;
        self.profit("Patron combined", patron.total_net_eur)?;
        self.profit("Mean per round", patron.mean_net_eur_per_round)?;
        self.message(&format!("95% interval for the mean: EUR {:.4} to {:.4} per round",
            patron.approximate_95_percent_ci_mean_eur[0], patron.approximate_95_percent_ci_mean_eur[1]))?;
        self.message(&format!("Per-round standard deviation: EUR {:.2}", patron.sample_standard_deviation_eur))?;
        self.message("Negative means loss. Returned winning stakes are not counted as profit.")?;
        self.message("Combined uncertainty includes correlation between your boxes.")?;
        self.message("This interval describes the mean under the chosen model, not a session range.")?;
        self.message("CSM reset and scenario assumptions apply; no bankroll exhaustion is modeled.")
    }

    fn profit(&mut self, label: &str, value: f64) -> io::Result<()> {
        if self.color { write!(self.output, "{}", if value < 0.0 { "\x1b[31m" } else { "\x1b[32m" })?; }
        writeln!(self.output, "  {label}: EUR {value:+.4}")?;
        if self.color { write!(self.output, "\x1b[0m")?; }
        Ok(())
    }

    fn exact(&mut self) -> io::Result<()> {
        self.title("EXACT SIDE BET ODDS  |  Six decks")?;
        self.message("Uniform full pack, draws without replacement. No sampling error.")?;
        self.message("This reference does not reproduce a particular CSM's internal mixing.")?;
        let report = side_bets::exact_probabilities(6).map_err(io::Error::other)?;
        self.exact_table("Pair-Fect Pairs", &report.pair_fect_pairs)?;
        self.exact_table("COD 21+", &report.cod_21_plus)?;
        self.message("House edge is average loss per unit staked, not probability of losing.")?;
        self.offer_save(&serde_json::to_value(report).map_err(io::Error::other)?, "exact-odds")
    }

    fn exact_table(&mut self, name: &str, bet: &ExactBet) -> io::Result<()> {
        self.message("")?;
        self.message(&format!("{name} | House edge {:.4}% | RTP {:.4}%", bet.house_edge * 100.0, bet.return_to_player * 100.0))?;
        writeln!(self.output, "  {:<30} {:>12} {:>20}", "Outcome", "Probability", "Net / EUR 1")?;
        let mut categories: Vec<_> = bet.categories.iter().collect();
        categories.sort_by_key(|(_, c)| std::cmp::Reverse(c.profit_units));
        for (name, c) in categories {
            writeln!(self.output, "  {:<30} {:>11.6}% {:>+20}", friendly(name), c.probability * 100.0, c.profit_units)?;
        }
        self.message(&format!("Average net per EUR 100 staked: EUR {:.4}", bet.expected_profit_per_unit * 100.0))
    }

    fn hand(&mut self) -> io::Result<()> {
        self.title("CHECK A THREE-CARD HAND")?;
        self.message("Enter your two cards, then dealer upcard. Example: 6H 8H 7H")?;
        self.message("Ranks: A, 2-10, J, Q, K. Suits: C clubs, D diamonds, H hearts, S spades.")?;
        self.message("This evaluates side bet payouts, not the best main-game action.")?;
        let Some((cards, text)) = self.field("Three cards (q returns)", "", |s| {
            let parts: Vec<_> = s.split_whitespace().collect();
            if parts.len() != 3 { return Err("Enter exactly three cards separated by spaces".into()); }
            let cards: Result<Vec<_>, _> = parts.iter().map(|s| PhysicalCard::from_str(s)).collect();
            Ok((cards.map_err(str::to_string)?, parts.join(" ").to_ascii_uppercase()))
        })? else { return Ok(()); };
        let Some(stake) = self.field("Example stake on EACH side bet EUR", "1", |s| money(s, true))?
            else { return Ok(()); };
        let pair = side_bets::pair_fect([cards[0], cards[1]], cards[2]);
        let total = side_bets::cod_21([cards[0], cards[1]], cards[2]);
        self.title(&format!("SIDE BET PAYOUTS  |  {text}"))?;
        for (name, result) in [("Pair-Fect Pairs", pair), ("COD 21+", total)] {
            let net = result.profit_units as f64 * stake;
            self.message(&format!("{name}: {}", friendly(result.category)))?;
            self.message(&format!("Stake EUR {stake:.2} | net EUR {net:+.2} | cash returned EUR {:.2}",
                if result.profit_units >= 0 { net + stake } else { 0.0 }))?;
        }
        self.message("Awards settle on initial cards and survive a later main-game loss/surrender.")?;
        self.offer_save(&serde_json::json!({"cards": text, "stake_eur_each": stake,
            "pair_fect_pairs": pair, "cod_21_plus": total}), "hand")
    }

    fn offer_save(&mut self, value: &Value, name: &str) -> io::Result<()> {
        loop {
            let Some(action) = self.prompt("[s] Save JSON report  |  [Enter] Back to menu", "")?
                else { return Ok(()); };
            match action.to_ascii_lowercase().as_str() {
                "" | "b" => return Ok(()),
                "s" => { self.save_json(value, name)?; return Ok(()); }
                _ => self.message("Type s to save, or press Enter to go back.")?,
            }
        }
    }

    fn save_json(&mut self, value: &Value, name: &str) -> io::Result<()> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
        let default = format!("research/interactive/{name}-{stamp}.json");
        let Some(path) = self.prompt("Save file (q cancels)", &default)? else { return Ok(()); };
        let path = Path::new(&path);
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent)?;
        }
        // A new report must never silently overwrite an existing experiment.
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
        serde_json::to_writer_pretty(&mut file, value).map_err(io::Error::other)?;
        writeln!(file)?;
        self.message(&format!("Saved {}", path.display()))
    }

    fn profiles(&mut self) -> io::Result<()> {
        self.title("SAVED SETUPS")?;
        let Some(action) = self.prompt("[1] Save current setup  [2] Load setup  [q] Back", "")?
            else { return Ok(()); };
        match action.as_str() {
            "1" => {
                let Some(config) = self.config.clone() else {
                    return self.message("Configure a simulation before saving a setup.");
                };
                let value = serde_json::to_value(SavedSetup { schema_version: 1, config }).map_err(io::Error::other)?;
                self.save_json(&value, "setup")
            }
            "2" => {
                let Some(path) = self.prompt("Setup JSON path (q cancels)", "")? else { return Ok(()); };
                let setup: SavedSetup = serde_json::from_slice(&fs::read(&path)?).map_err(io::Error::other)?;
                if setup.schema_version != 1 {
                    return self.message("Unsupported setup version. Your current setup is unchanged.");
                }
                if let Err(error) = setup.config.validate() {
                    return self.message(&format!("Invalid setup: {error}. Your current setup is unchanged."));
                }
                if setup.config.rounds > 100_000_000 {
                    return self.message("Interactive setups support at most 100m rounds. Setup unchanged.");
                }
                self.show_setup(&setup.config)?;
                self.message("Loaded rule selections remain research assumptions.")?;
                if self.confirm("Use this loaded setup (yes/no)")? {
                    self.config = Some(setup.config);
                    self.message("Setup loaded.")?;
                }
                Ok(())
            }
            _ => self.message("Choose 1 or 2 next time to save or load a setup."),
        }
    }

    fn rules(&mut self) -> io::Result<()> {
        self.title("RULES AND MODEL NOTES")?;
        for line in [
            "OBSERVED / PUBLISHED",
            "Six decks in a continuous shuffler; six boxes; at most three owned.",
            "Dealer stands on all 17s. Early surrender excludes a dealer Ace.",
            "Natural pays 3:2. Maximum three split hands; split aces get one card.",
            "Opening main bet maximum is 50 times the table minimum, per box.",
            "",
            "CHOOSE AS SCENARIOS UNTIL CONFIRMED",
            "Dealer second-card timing / blackjack settlement; double after split.",
            "Side bet limits and limits on additional split/double exposure are unknown.",
            "",
            "CURRENT RESEARCH MODEL",
            "All six decks reset between rounds; cards removed within each round.",
            "This does not model the actual CSM's return delay or internal mixing.",
            "All occupied boxes use inherited basic strategy; not proven COD optimal.",
            "Insurance declined. Fixed wagers and unlimited bankroll; no ruin model.",
            "",
            "Sources and full specification: docs/casino-rules.md and docs/sources.md",
        ] { self.message(line)?; }
        self.prompt("Press Enter to return", "")?;
        Ok(())
    }
}

fn money(s: &str, positive: bool) -> Result<f64, String> {
    let value = s.parse::<f64>().map_err(|_| "Enter an amount such as 50 or 12.50")?;
    // Keep totals and payouts finite even for long multi-box research runs.
    if !value.is_finite() || value < 0.0 || (positive && value == 0.0) || value > 1_000_000_000.0 {
        return Err("Use a finite amount up to EUR 1 billion; this field must be positive unless 0 disables a side bet".into());
    }
    Ok(value)
}

fn parse_rounds(s: &str) -> Result<u64, String> {
    let s = s.replace(['_', ','], "").to_ascii_lowercase();
    let (digits, scale) = if let Some(s) = s.strip_suffix('k') { (s, 1_000) }
        else if let Some(s) = s.strip_suffix('m') { (s, 1_000_000) }
        else { (s.as_str(), 1) };
    digits.parse::<u64>().ok().and_then(|n| n.checked_mul(scale))
        .ok_or_else(|| "Enter a whole number, optionally ending in k or m".into())
}

fn boxes(positions: &[u8]) -> String {
    positions.iter().map(u8::to_string).collect::<Vec<_>>().join(",")
}

fn dealer_label(mode: DealerMode) -> &'static str {
    match mode {
        DealerMode::AmericanPeek => "hole card + peek",
        DealerMode::EuropeanAllBetsLost => "later card / all bets lost to BJ",
    }
}

fn grouped(value: u64) -> String {
    let text = value.to_string();
    text.chars().enumerate().fold(String::new(), |mut out, (i, c)| {
        if i > 0 && (text.len() - i) % 3 == 0 { out.push(','); }
        out.push(c);
        out
    })
}

fn friendly(category: &str) -> String {
    category.replace('_', " ")
}
