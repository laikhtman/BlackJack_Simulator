use freebj::betting::FixedBet;
use freebj::card::Card;
use freebj::cod::side_bets::{PhysicalCard, cod_21, exact_probabilities, pair_fect};
use freebj::cod::{DealerMode, SimulationConfig, simulate};
use freebj::game_rules::{GameRules, SurrenderPolicy};
use freebj::hand::Hand;
use freebj::round::Round;
use freebj::shoe::queued_shoe::QueuedShoe;
use freebj::strategy::{Decision, GameContext, Strategy};
use std::collections::VecDeque;
use std::str::FromStr;

fn card(s: &str) -> PhysicalCard { PhysicalCard::from_str(s).unwrap() }

#[test]
fn pays_each_pair_category_once_and_keeps_face_ranks_distinct() {
    for (a, b, d, category, pay) in [
        ("JH", "JH", "JH", "suited_three_of_a_kind", 50),
        ("JH", "JH", "JD", "suited_pair", 30),
        ("JH", "JD", "JH", "prime_pair", 10),
        ("JH", "JC", "JH", "any_pair", 5),
        ("JH", "QH", "JH", "loss", -1),
        ("10H", "JH", "10H", "loss", -1),
    ] {
        let result = pair_fect([card(a), card(b)], card(d));
        assert_eq!((result.category, result.profit_units), (category, pay));
    }
}

#[test]
fn pays_all_cod21_categories_with_priority_and_soft_aces() {
    for (a, b, d, category, pay) in [
        ("7H", "7H", "7H", "suited_777", 200),
        ("8H", "6H", "7H", "suited_678", 100),
        ("7H", "7D", "7S", "unsuited_777", 50),
        ("8H", "6D", "7H", "unsuited_678", 30),
        ("AH", "AH", "9H", "suited_21", 10),
        ("AH", "2D", "8H", "unsuited_21", 3),
        ("KH", "QH", "AD", "unsuited_21", 3),
        ("AH", "AH", "8D", "total_20", 2),
        ("AH", "2H", "6H", "total_19", 2),
        ("KH", "QD", "2H", "loss", -1),
    ] {
        let result = cod_21([card(a), card(b)], card(d));
        assert_eq!((result.category, result.profit_units), (category, pay));
        assert_eq!(cod_21([card(d), card(a)], card(b)), result);
    }
}

#[test]
fn enumerates_six_decks_and_matches_independent_pair_combinatorics() {
    let report = exact_probabilities(6).unwrap();
    assert_eq!(report.ordered_outcomes, 312 * 311 * 310);
    let pairs = &report.pair_fect_pairs.categories;
    // 52 card types, 6 copies of each. Dealer counts exclude the two player cards.
    assert_eq!(pairs["suited_three_of_a_kind"].ordered_outcomes, 52 * 6 * 5 * 4);
    assert_eq!(pairs["suited_pair"].ordered_outcomes, 52 * 6 * 5 * (310 - 4));
    assert_eq!(pairs["prime_pair"].ordered_outcomes, 52 * 6 * 6 * 310);
    assert_eq!(pairs["any_pair"].ordered_outcomes, 52 * 6 * 12 * 310);
    for bet in [&report.pair_fect_pairs, &report.cod_21_plus] {
        assert_eq!(bet.categories.values().map(|c| c.ordered_outcomes).sum::<u64>(), report.ordered_outcomes);
        assert!((bet.categories.values().map(|c| c.probability).sum::<f64>() - 1.0).abs() < 1e-12);
        assert!((bet.return_to_player + bet.house_edge - 1.0).abs() < 1e-12);
    }
    // Published casino guide: 5.37% and 5.3%, respectively, rounded.
    assert_eq!((report.pair_fect_pairs.house_edge * 10000.0).round() / 100.0, 5.37);
    assert_eq!((report.cod_21_plus.house_edge * 1000.0).round() / 10.0, 5.3);
    let single = exact_probabilities(1).unwrap();
    assert!(!single.pair_fect_pairs.categories.contains_key("suited_pair"));
    assert!(!single.pair_fect_pairs.categories.contains_key("suited_three_of_a_kind"));
    assert!(exact_probabilities(0).is_err());
}

struct StandAndInsure;
impl Strategy for StandAndInsure {
    fn player_turn(&self, _: &GameContext, _: Card, _: &Hand) -> Decision { Decision::Stand }
    fn take_insurance(&self, _: &GameContext, _: &Hand) -> bool { true }
}

struct StandOnly;
impl Strategy for StandOnly {
    fn player_turn(&self, _: &GameContext, _: Card, _: &Hand) -> Decision { Decision::Stand }
}

#[test]
fn keeps_six_box_results_and_split_budgets_separate() {
    let empty = VecDeque::new();
    let rules = GameRules::default();
    // First card for six boxes, dealer upcard, second card for six boxes, hole card.
    let mut shoe = QueuedShoe::from_ints(&[10,10,10,10,10,10,10,10,9,8,7,6,5,8]);
    let (_, result) = Round::new(&rules, &StandOnly, &FixedBet(50.0), &mut shoe,
        6, false, None, None, &empty, &empty).run();
    assert_eq!(result.player_results, [50.0,50.0,0.0,-50.0,-50.0,-50.0,0.0]);

    let rules = GameRules { max_splits: 2, ..Default::default() };
    let strategy = freebj::basic_strategy::BasicStrategy::new(false);
    // Two boxes each split 8/8 to 18 and 18; dealer draws to 19.
    let mut shoe = QueuedShoe::from_ints(&[8,8,6,8,8,10,10,10,10,10,3]);
    let (_, result) = Round::new(&rules, &strategy, &FixedBet(50.0), &mut shoe,
        2, false, None, None, &empty, &empty).run();
    assert_eq!(result.player_results[0], -100.0);
    assert_eq!(result.player_results[1], -100.0);
    assert_eq!(result.hand_stats.split, 4);
}

#[test]
fn early_surrender_survives_dealer_blackjack_but_is_forbidden_against_ace() {
    let rules = GameRules { surrender: SurrenderPolicy::EarlySurrenderExceptAce, ..Default::default() };
    let empty = VecDeque::new();
    for (cards, expected, surrendered) in [
        ([10, 10, 5, 1], -25.0, 1), // surrender against ten, despite dealer blackjack
        ([10, 1, 5, 10], 0.0, 0), // no surrender against ace; insurance offsets main loss
    ] {
        let mut shoe = QueuedShoe::from_ints(&cards);
        let (_, result) = Round::new(&rules, &StandAndInsure, &FixedBet(50.0),
            &mut shoe, 1, false, None, Some(true), &empty, &empty).run();
        assert_eq!(result.player_results[0], expected);
        assert_eq!(result.hand_stats.surrender, surrendered);
    }
}

fn config() -> SimulationConfig {
    SimulationConfig {
        rounds: 2000, seed: 42, occupied_boxes: 6, owned_boxes: vec![1, 3, 6],
        table_minimum_eur: 50.0, main_bet_eur_per_box: 50.0,
        pair_fect_eur_per_box: 5.0, cod_21_eur_per_box: 5.0,
        double_after_split: true, dealer_mode: DealerMode::EuropeanAllBetsLost,
    }
}

#[test]
fn seeded_multiple_box_results_reconcile_and_side_bets_do_not_change_main_play() {
    let report = simulate(config()).unwrap();
    let repeated = simulate(config()).unwrap();
    assert_eq!(serde_json::to_value(&report).unwrap(), serde_json::to_value(&repeated).unwrap());
    let sum: f64 = report.boxes.iter().map(|b| b.combined.total_net_eur).sum();
    assert!((sum - report.patron_combined.total_net_eur).abs() < 1e-7);
    let mut no_side = config();
    no_side.pair_fect_eur_per_box = 0.0;
    no_side.cod_21_eur_per_box = 0.0;
    let no_side = simulate(no_side).unwrap();
    for (with, without) in report.boxes.iter().zip(no_side.boxes) {
        assert_eq!(with.main_game.total_net_eur, without.main_game.total_net_eur);
    }
    let mut peek = config();
    peek.dealer_mode = DealerMode::AmericanPeek;
    assert!(simulate(peek).is_ok());
}

#[test]
fn validates_table_limits_boxes_and_finite_stakes() {
    let mut settings = config();
    settings.main_bet_eur_per_box = 2500.0;
    assert!(settings.validate().is_ok());
    settings.main_bet_eur_per_box = 2500.01;
    assert!(settings.validate().is_err());
    settings.main_bet_eur_per_box = 49.99;
    assert!(settings.validate().is_err());
    settings = config();
    settings.owned_boxes = vec![1, 2, 3, 4];
    assert!(settings.validate().is_err());
    settings.owned_boxes = vec![1, 1];
    assert!(settings.validate().is_err());
    settings.owned_boxes = vec![0];
    assert!(settings.validate().is_err());
    settings = config();
    settings.occupied_boxes = 7;
    assert!(settings.validate().is_err());
    settings = config();
    settings.cod_21_eur_per_box = f64::NAN;
    assert!(settings.validate().is_err());
    assert!(PhysicalCard::from_str("🂡").is_err());
    assert!(PhysicalCard::from_str("14H").is_err());
}

#[test]
fn cli_evaluates_a_hand_and_requires_unconfirmed_scenario_choices() {
    let bin = env!("CARGO_BIN_EXE_cod-research");
    let output = std::process::Command::new(bin).args(["hand", "7H", "7H", "7H"]).output().unwrap();
    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["cod_21_plus"]["profit_units"], 200);
    let output = std::process::Command::new(bin).arg("simulate").output().unwrap();
    assert!(!output.status.success());
}
