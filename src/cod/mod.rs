//! Research scenarios for COD Limassol. CSM internals remain an explicit assumption.
pub mod side_bets;
pub mod interactive;

use crate::basic_strategy::BasicStrategy;
use crate::betting::FixedBet;
use crate::card::Card;
use crate::game_rules::{DoublePolicy, GameRules, GameType, Soft17, SurrenderPolicy};
use crate::round::Round;
use crate::running_stats::RunningStats;
use crate::shoe::CardShoe;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use side_bets::{PhysicalCard, card_types, cod_21, pair_fect};
use std::collections::{BTreeSet, VecDeque};
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DealerMode {
    AmericanPeek,
    EuropeanAllBetsLost,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationConfig {
    pub rounds: u64,
    pub seed: u64,
    pub occupied_boxes: u8,
    /// One-based occupied positions in dealing order, maximum three owned boxes.
    pub owned_boxes: Vec<u8>,
    pub table_minimum_eur: f64,
    pub main_bet_eur_per_box: f64,
    pub pair_fect_eur_per_box: f64,
    pub cod_21_eur_per_box: f64,
    pub double_after_split: bool,
    pub dealer_mode: DealerMode,
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.rounds < 2 { return Err("At least two rounds are required for uncertainty estimates"); }
        if !(1..=6).contains(&self.occupied_boxes) { return Err("There are 1 through 6 occupied boxes"); }
        if self.owned_boxes.is_empty() || self.owned_boxes.len() > 3 {
            return Err("A patron must play 1 through 3 boxes");
        }
        let unique: BTreeSet<_> = self.owned_boxes.iter().copied().collect();
        if unique.len() != self.owned_boxes.len()
            || unique.iter().any(|&b| b == 0 || b > self.occupied_boxes) {
            return Err("Owned boxes must be distinct occupied positions");
        }
        if !self.table_minimum_eur.is_finite() || self.table_minimum_eur <= 0.0
            || !(self.table_minimum_eur * 50.0).is_finite() {
            return Err("Table minimum must be positive and finite");
        }
        if !self.main_bet_eur_per_box.is_finite()
            || self.main_bet_eur_per_box < self.table_minimum_eur
            || self.main_bet_eur_per_box > self.table_minimum_eur * 50.0 {
            return Err("Opening main wager must be within the table minimum and 50 times minimum");
        }
        for bet in [self.pair_fect_eur_per_box, self.cod_21_eur_per_box] {
            if !bet.is_finite() || bet < 0.0 || !(bet * 200.0).is_finite() {
                return Err("Side wagers must be finite and nonnegative");
            }
        }
        Ok(())
    }
}

/// Six complete decks restored and uniformly shuffled before each round.
/// Physical cards are recorded before collapsing ranks for the upstream engine.
struct RoundPack {
    cards: Vec<PhysicalCard>,
    dealt: Vec<PhysicalCard>,
    rng: SmallRng,
}

impl RoundPack {
    fn new(seed: u64) -> Self {
        Self { cards: Vec::new(), dealt: Vec::new(), rng: SmallRng::seed_from_u64(seed) }
    }

    fn begin_round(&mut self) {
        self.cards.clear();
        self.dealt.clear();
        for _ in 0..6 { self.cards.extend(card_types()); }
        self.cards.shuffle(&mut self.rng);
    }

    fn record(&mut self, card: PhysicalCard) -> Card {
        self.dealt.push(card);
        Card(card.value())
    }
}

impl fmt::Display for RoundPack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Six-deck round pack with {} cards remaining", self.cards.len())
    }
}

impl CardShoe for RoundPack {
    fn try_pick(&mut self) -> Option<Card> {
        let card = self.cards.pop()?;
        Some(self.record(card))
    }
    fn try_pick_first(&mut self, card: Card) -> Option<Card> {
        let index = self.cards.iter().position(|c| c.value() == card.0)?;
        let card = self.cards.remove(index);
        Some(self.record(card))
    }
    fn pick(&mut self) -> Card {
        self.try_pick().expect("Six-deck pack exhausted within one round")
    }
    fn reshuffle(&mut self) {
        panic!("A research round pack can only reset between rounds");
    }
    fn force_true_count(&mut self, _: f32) { panic!("No forced count in the CSM reference model"); }
    fn adjust_running_count(&mut self, _: i32) { panic!("No count adjustment in the CSM reference model"); }
}

#[derive(Debug, Serialize)]
pub struct ProfitStats {
    pub mean_net_eur_per_round: f64,
    pub sample_standard_deviation_eur: f64,
    pub standard_error_eur: f64,
    pub approximate_95_percent_ci_mean_eur: [f64; 2],
    pub total_net_eur: f64,
}

fn stats(values: &RunningStats) -> ProfitStats {
    let error = values.stddev() / (values.count() as f64).sqrt();
    ProfitStats {
        mean_net_eur_per_round: values.mean(), sample_standard_deviation_eur: values.stddev(),
        standard_error_eur: error,
        approximate_95_percent_ci_mean_eur: [values.mean() - 1.96 * error, values.mean() + 1.96 * error],
        total_net_eur: values.mean() * values.count() as f64,
    }
}

#[derive(Debug, Serialize)]
pub struct BoxResult {
    pub box_number: u8,
    pub main_game: ProfitStats,
    pub pair_fect_pairs: ProfitStats,
    pub cod_21_plus: ProfitStats,
    pub combined: ProfitStats,
}

#[derive(Debug, Serialize)]
pub struct SimulationReport {
    pub config: SimulationConfig,
    pub decks: u32,
    pub shuffle_model: &'static str,
    pub strategy: &'static str,
    pub insurance: &'static str,
    pub limitations: [&'static str; 4],
    pub boxes: Vec<BoxResult>,
    pub patron_combined: ProfitStats,
}

pub fn simulate(config: SimulationConfig) -> Result<SimulationReport, &'static str> {
    simulate_with_progress(config, |_, _| {})
}

/// Reports completed rounds without changing the RNG stream or round accounting.
pub fn simulate_with_progress(
    config: SimulationConfig,
    mut progress: impl FnMut(u64, u64),
) -> Result<SimulationReport, &'static str> {
    config.validate()?;
    progress(0, config.rounds);
    let rules = GameRules {
        game_type: match config.dealer_mode {
            DealerMode::AmericanPeek => GameType::Ahc,
            DealerMode::EuropeanAllBetsLost => GameType::Enhc,
        },
        soft17: Soft17::S17, das: config.double_after_split, bj_pays: 1.5,
        double_down: DoublePolicy::AnyTwo,
        surrender: SurrenderPolicy::EarlySurrenderExceptAce,
        play_ace_pairs: false, max_splits: 3, decks: 6, penetration_cards: 312,
    };
    let strategy = BasicStrategy::new(false);
    let betting = FixedBet(config.main_bet_eur_per_box);
    let empty = VecDeque::new();
    let mut shoe = RoundPack::new(config.seed);
    let mut by_box: Vec<[RunningStats; 4]> = (0..config.owned_boxes.len())
        .map(|_| std::array::from_fn(|_| RunningStats::default())).collect();
    let mut patron = RunningStats::default();
    let progress_step = (config.rounds / 100).max(1);
    for round in 0..config.rounds {
        shoe.begin_round();
        let result = {
            let (_, result) = Round::new(&rules, &strategy, &betting, &mut shoe,
                config.occupied_boxes, false, None, None, &empty, &empty).run();
            result
        };
        let occupied = config.occupied_boxes as usize;
        let dealer = shoe.dealt[occupied];
        let mut round_net = 0.0;
        for (i, &position) in config.owned_boxes.iter().enumerate() {
            let index = position as usize - 1;
            let player = [shoe.dealt[index], shoe.dealt[occupied + 1 + index]];
            let main = result.player_results[index];
            let pair = pair_fect(player, dealer).profit_units as f64 * config.pair_fect_eur_per_box;
            let total = cod_21(player, dealer).profit_units as f64 * config.cod_21_eur_per_box;
            let combined = main + pair + total;
            for (stat, value) in by_box[i].iter_mut().zip([main, pair, total, combined]) {
                stat.push(value);
            }
            round_net += combined;
        }
        patron.push(round_net);
        let done = round + 1;
        if done % progress_step == 0 || done == config.rounds {
            progress(done, config.rounds);
        }
    }
    let boxes = config.owned_boxes.iter().zip(by_box).map(|(&position, values)| BoxResult {
        box_number: position, main_game: stats(&values[0]), pair_fect_pairs: stats(&values[1]),
        cod_21_plus: stats(&values[2]), combined: stats(&values[3]),
    }).collect();
    Ok(SimulationReport {
        config, decks: 6, shuffle_model: "uniform_full_pack_reset_between_rounds",
        strategy: "upstream_basic_strategy_without_counting_not_proven_cod_optimal",
        insurance: "declined",
        limitations: [
            "CSM return delays and internal mixing are not modeled",
            "Dealer mode and double after split are user supplied scenarios, not confirmed casino facts",
            "Side bet limits and treatment of added split/double stakes at table maximum remain unconfirmed",
            "All occupied boxes use the same strategy and fixed main wager; bankroll exhaustion is not modeled",
        ],
        boxes, patron_combined: stats(&patron),
    })
}
