//! Bankroll target experiments using the existing FreeBJ round engine.
//! These compare named policies; they do not prove a globally optimal gambling strategy.
use clap::{App, Arg};
use freebj::basic_strategy::BasicStrategy;
use freebj::betting::FixedBet;
use freebj::card::Card;
use freebj::cod::side_bets::{PhysicalCard, card_types, pair_fect, cod_21};
use freebj::game_rules::{GameRules, GameType, Soft17, DoublePolicy, SurrenderPolicy};
use freebj::hand::Hand;
use freebj::round::Round;
use freebj::shoe::CardShoe;
use freebj::strategy::{Decision, GameContext, Strategy};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

struct Pack {
    cards: Vec<PhysicalCard>,
    remaining: usize,
    history: Rc<RefCell<Vec<PhysicalCard>>>,
    rng: SmallRng,
}
impl Pack {
    fn new(seed: u64) -> Self {
        let mut cards = Vec::with_capacity(312);
        for _ in 0..6 { cards.extend(card_types()); }
        Self { cards, remaining: 312, history: Rc::new(RefCell::new(Vec::new())),
            rng: SmallRng::seed_from_u64(seed) }
    }
    fn reset(&mut self) { self.remaining = 312; self.history.borrow_mut().clear(); }
    fn remove(&mut self, index: usize) -> Card {
        self.remaining -= 1;
        self.cards.swap(index, self.remaining);
        let card = self.cards[self.remaining];
        self.history.borrow_mut().push(card);
        Card(card.value())
    }
}
impl fmt::Display for Pack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{} cards", self.remaining) }
}
impl CardShoe for Pack {
    fn try_pick(&mut self) -> Option<Card> {
        if self.remaining == 0 { return None; }
        let index = self.rng.random_range(0..self.remaining);
        Some(self.remove(index))
    }
    fn try_pick_first(&mut self, card: Card) -> Option<Card> {
        let index = self.cards[..self.remaining].iter().position(|c| c.value() == card.0)?;
        Some(self.remove(index))
    }
    fn pick(&mut self) -> Card { self.try_pick().expect("Pack exhausted inside a round") }
    fn reshuffle(&mut self) { panic!("Only reset between rounds"); }
    fn force_true_count(&mut self, _: f32) { panic!("No counting in reference CSM"); }
    fn adjust_running_count(&mut self, _: i32) { panic!("No counting in reference CSM"); }
}

struct Funds { cash: f64, sides_settled: bool, side_net: f64, unfunded: u64 }
struct FundedStrategy {
    base: BasicStrategy,
    funds: RefCell<Funds>,
    history: Rc<RefCell<Vec<PhysicalCard>>>,
    occupied: usize,
    owned: usize,
    pair: f64,
    total: f64,
}
impl FundedStrategy {
    fn settle_sides(&self) {
        let mut f = self.funds.borrow_mut();
        if f.sides_settled { return; }
        let h = self.history.borrow();
        let dealer = h[self.occupied];
        for i in 0..self.owned {
            let player = [h[i], h[self.occupied + 1 + i]];
            let p = pair_fect(player, dealer).profit_units as f64 * self.pair;
            let t = cod_21(player, dealer).profit_units as f64 * self.total;
            f.side_net += p + t;
            // Side stakes were reserved before the deal; winnings settle before decisions.
            f.cash += p + self.pair + t + self.total;
        }
        f.sides_settled = true;
    }
}
impl Strategy for FundedStrategy {
    fn player_turn(&self, game: &GameContext, dealer: Card, hand: &Hand) -> Decision {
        self.settle_sides();
        if hand.id as usize >= self.owned { return self.base.player_turn(game, dealer, hand); }
        let cash = self.funds.borrow().cash;
        let context = GameContext {
            rules: game.rules, may_double: game.may_double && cash + 1e-8 >= hand.bet,
            may_split: game.may_split && cash + 1e-8 >= hand.bet,
            true_count: game.true_count, holecard: game.holecard,
        };
        let preferred = self.base.player_turn(game, dealer, hand);
        let chosen = self.base.player_turn(&context, dealer, hand);
        let mut f = self.funds.borrow_mut();
        if preferred != chosen { f.unfunded += 1; }
        if matches!(chosen, Decision::Double | Decision::Split) {
            f.cash -= hand.bet;
            assert!(f.cash >= -1e-8);
        }
        chosen
    }
    fn surrender(&self, game: &GameContext, dealer: Card, hand: &Hand, early: bool) -> bool {
        self.settle_sides();
        let surrender = self.base.surrender(game, dealer, hand, early);
        if surrender && (hand.id as usize) < self.owned {
            self.funds.borrow_mut().cash += hand.bet * 0.5;
        }
        surrender
    }
}

#[derive(Clone, Serialize)]
struct Policy { name: String, boxes: usize, fixed: f64, fraction: f64, gap_cap: bool,
    martingale: bool, pair: f64, total: f64 }
fn policies() -> Vec<Policy> {
    let mut result = Vec::new();
    for fixed in [50.0, 250.0, 1000.0, 2500.0] {
        result.push(Policy { name: format!("flat_{fixed:.0}_one_box"), boxes: 1, fixed,
            fraction: 0.0, gap_cap: false, martingale: false, pair: 0.0, total: 0.0 });
    }
    result.push(Policy { name: "flat_2500_three_boxes".into(), boxes: 3, fixed: 2500.0,
        fraction: 0.0, gap_cap: false, martingale: false, pair: 0.0, total: 0.0 });
    for fraction in [0.05, 0.10, 0.25] {
        result.push(Policy { name: format!("fraction_{:.0}_percent_one_box", fraction * 100.0),
            boxes: 1, fixed: 0.0, fraction, gap_cap: true, martingale: false, pair: 0.0, total: 0.0 });
    }
    for boxes in [1, 3] {
        result.push(Policy { name: format!("max_with_target_gap_{boxes}_boxes"), boxes,
            fixed: 2500.0, fraction: 0.0, gap_cap: true, martingale: false, pair: 0.0, total: 0.0 });
    }
    result.push(Policy { name: "martingale_50_to_2500_one_box".into(), boxes: 1, fixed: 50.0,
        fraction: 0.0, gap_cap: false, martingale: true, pair: 0.0, total: 0.0 });
    for (name, pair, total) in [("pair_fect_50",50.0,0.0), ("cod21_50",0.0,50.0)] {
        result.push(Policy { name: format!("exploratory_{name}_main50"), boxes: 1, fixed: 50.0,
            fraction: 0.0, gap_cap: false, martingale: false, pair, total });
    }
    result
}

#[derive(Debug)]
struct Outcome { ending: f64, rounds: u64, hit: bool, ruined: bool, unfunded: u64 }
fn session(pack: &mut Pack, rules: &GameRules, p: &Policy, max_rounds: u64) -> Outcome {
    let empty = VecDeque::new();
    let mut bank: f64 = 10_000.0;
    let mut martingale: f64 = 50.0;
    let mut unfunded = 0;
    for round in 0..max_rounds {
        let owned = p.boxes.min((bank / 50.0).floor() as usize);
        if owned == 0 { return Outcome { ending: bank, rounds: round, hit: false, ruined: true, unfunded }; }
        let mut pair = p.pair;
        let mut total = p.total;
        // If optional side stakes cannot be funded alongside legal minimum main stakes, omit them.
        if owned as f64 * (50.0 + pair + total) > bank { pair = 0.0; total = 0.0; }
        let desired = if p.martingale { martingale } else if p.fraction > 0.0 { bank * p.fraction } else { p.fixed };
        let affordable = bank / owned as f64 - pair - total;
        let gap = if p.gap_cap { ((100_000.0 - bank) / owned as f64 / 50.0).ceil() * 50.0 } else { 2500.0 };
        let bet = ((desired.min(2500.0).min(affordable).min(gap) / 50.0).floor() * 50.0).max(50.0);
        assert!(bet >= 50.0 && bet <= 2500.0);
        let cash = bank - owned as f64 * (bet + pair + total);
        assert!(cash >= -1e-8);
        pack.reset();
        let strategy = FundedStrategy {
            base: BasicStrategy::new(false),
            funds: RefCell::new(Funds { cash, sides_settled: false, side_net: 0.0, unfunded: 0 }),
            history: pack.history.clone(), occupied: 6, owned, pair, total,
        };
        let outcome = {
            let (_, result) = Round::new(rules, &strategy, &FixedBet(bet), pack,
                6, false, None, None, &empty, &empty).run();
            result
        };
        strategy.settle_sides(); // Also handles immediate dealer blackjack.
        let f = strategy.funds.borrow();
        let net = outcome.player_results[..owned].iter().sum::<f64>() + f.side_net;
        assert!(f.cash >= -1e-8);
        unfunded += f.unfunded;
        bank += net;
        assert!(bank >= -1e-8, "Unfunded session");
        if p.martingale {
            if net < 0.0 { martingale = (martingale * 2.0).min(2500.0); }
            else if net > 0.0 { martingale = 50.0; }
        }
        if bank >= 100_000.0 { return Outcome { ending: bank, rounds: round + 1, hit: true, ruined: false, unfunded }; }
        if bank < 50.0 { return Outcome { ending: bank, rounds: round + 1, hit: false, ruined: true, unfunded }; }
    }
    Outcome { ending: bank, rounds: max_rounds, hit: false, ruined: false, unfunded }
}
fn wilson(successes: u64, n: u64) -> [f64; 2] {
    let n = n as f64; let p = successes as f64 / n; let z = 1.959963984540054;
    let denom = 1.0 + z*z/n;
    let center = (p + z*z/(2.0*n)) / denom;
    let half = z * (p*(1.0-p)/n + z*z/(4.0*n*n)).sqrt() / denom;
    [ (center-half).max(0.0), (center+half).min(1.0) ]
}
#[derive(Serialize)]
struct ResultRow {
    phase: String, policy: Policy, sessions: u64, seed: u64, max_rounds: u64,
    hit: u64, unplayable: u64, unfinished: u64,
    probability_hit_by_cap: f64, hit_probability_95_wilson: [f64;2],
    eventual_probability_identification_interval: [f64;2],
    mean_ending_eur: f64, mean_net_eur: f64, mean_rounds: f64,
    mean_rounds_successful: Option<f64>, unfunded_decisions: u64,
}
fn experiment(p: &Policy, rules: &GameRules, sessions: u64, cap: u64, seed: u64, phase: &str) -> ResultRow {
    let mut pack = Pack::new(seed);
    let (mut hit,mut unplayable,mut unfinished,mut total_rounds,mut winning_rounds,mut ending,mut unfunded) =
        (0u64,0u64,0u64,0u64,0u64,0.0,0u64);
    for _ in 0..sessions {
        let result = session(&mut pack, rules, p, cap);
        if result.hit { hit += 1; winning_rounds += result.rounds; }
        else if result.ruined { unplayable += 1; } else { unfinished += 1; }
        ending += result.ending; total_rounds += result.rounds; unfunded += result.unfunded;
    }
    let row = ResultRow {
        phase: phase.into(), policy: p.clone(), sessions, seed, max_rounds: cap, hit, unplayable, unfinished,
        probability_hit_by_cap: hit as f64 / sessions as f64,
        hit_probability_95_wilson: wilson(hit,sessions),
        eventual_probability_identification_interval: [hit as f64/sessions as f64,(hit+unfinished) as f64/sessions as f64],
        mean_ending_eur: ending/sessions as f64, mean_net_eur: ending/sessions as f64-10_000.0,
        mean_rounds: total_rounds as f64/sessions as f64,
        mean_rounds_successful: if hit>0 {Some(winning_rounds as f64/hit as f64)} else {None},
        unfunded_decisions: unfunded,
    };
    eprintln!("RESULT {} {} sessions={} hit={} ruin={} unfinished={} hit_pct={:.4} ci_pct=[{:.4},{:.4}] mean_ending={:.2} mean_rounds={:.1}",
        phase,p.name,sessions,hit,unplayable,unfinished,row.probability_hit_by_cap*100.0,
        row.hit_probability_95_wilson[0]*100.0,row.hit_probability_95_wilson[1]*100.0,
        row.mean_ending_eur,row.mean_rounds);
    row
}
fn main() {
    let args = App::new("cod-bankroll").about("Simulated EUR 10k to EUR 100k policy study")
        .arg(Arg::with_name("dealer").long("dealer").takes_value(true).required(true).possible_values(&["ahc","enhc"]))
        .arg(Arg::with_name("das").long("das").takes_value(true).required(true).possible_values(&["yes","no"]))
        .arg(Arg::with_name("sessions").long("sessions").takes_value(true).default_value("500"))
        .arg(Arg::with_name("holdout").long("holdout").takes_value(true).default_value("10000"))
        .arg(Arg::with_name("cap").long("max-rounds").takes_value(true).default_value("10000"))
        .arg(Arg::with_name("seed").long("seed").takes_value(true).default_value("20261018"))
        .get_matches();
    let parse = |name| args.value_of(name).unwrap().parse::<u64>().expect("Expected whole number");
    let count = parse("sessions"); let holdout = parse("holdout"); let cap = parse("cap"); let seed = parse("seed");
    assert!(count>0 && holdout>0 && cap>0);
    let rules = GameRules {
        game_type: if args.value_of("dealer")==Some("ahc") {GameType::Ahc} else {GameType::Enhc},
        soft17: Soft17::S17, das: args.value_of("das")==Some("yes"), bj_pays: 1.5,
        double_down: DoublePolicy::AnyTwo, surrender: SurrenderPolicy::EarlySurrenderExceptAce,
        play_ace_pairs: false, max_splits: 3, decks: 6, penetration_cards: 312,
    };
    let mut rows = Vec::new();
    let policies = policies();
    for (i,p) in policies.iter().enumerate() {
        rows.push(experiment(p,&rules,count,cap,seed+i as u64*1_000_003,"screen"));
    }
    let mut ranked: Vec<_> = rows.iter().filter(|r| r.policy.pair==0.0 && r.policy.total==0.0).collect();
    ranked.sort_by(|a,b| b.probability_hit_by_cap.total_cmp(&a.probability_hit_by_cap));
    let selected: Vec<_> = ranked.iter().take(3).map(|r| r.policy.clone()).collect();
    for (i,p) in selected.iter().enumerate() {
        rows.push(experiment(p,&rules,holdout,cap,seed+1_000_000_007+i as u64*1_000_003,"holdout"));
    }
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
        "starting_bankroll_eur":10000, "target_eur":100000, "minimum_eur":50, "maximum_eur_per_box":2500,
        "occupied_boxes":6, "dealer":args.value_of("dealer"), "double_after_split":rules.das,
        "model":"uniform_full_pack_reset_between_rounds_lazy_uniform_draws_without_replacement",
        "objective":"reach target after round settlement before bankroll below minimum, within max_rounds",
        "notes":[
            "Screening selects candidates; only independent holdout results should be used for ranking",
            "Intervals measure Monte Carlo uncertainty, not unknown casino rules or model error",
            "Unfinished sessions are not counted as ruin; eventual identification bounds exclude sampling uncertainty",
            "Policies use EUR 50 wager increments as a research grid, not a confirmed chip-increment rule",
            "Unfunded splits/doubles use inherited strategy with those actions unavailable",
            "Side bet EUR 50 limits are unconfirmed; side bet rows are exploratory and excluded from holdout selection",
            "Side payouts and surrender refunds are available before later player decisions",
            "Other boxes use the same main stake and basic strategy, with independent funds",
            "No policy here is proven globally optimal; main-game actions follow inherited expected-value basic strategy"
        ], "results":rows
    })).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn complete_physical_pack_and_restoration() {
        let mut p=Pack::new(42);
        for _ in 0..312 {p.pick();}
        assert!(p.try_pick().is_none());
        let h=p.history.borrow(); for c in card_types() {assert_eq!(h.iter().filter(|&&x| x==c).count(),6);}
        drop(h); p.reset(); assert_eq!(p.remaining,312); assert!(p.history.borrow().is_empty());
    }
    #[test] fn funded_sessions_never_go_negative_and_are_repeatable() {
        let rules=GameRules { surrender:SurrenderPolicy::EarlySurrenderExceptAce,..Default::default() };
        for p in policies() {
            let a=session(&mut Pack::new(7),&rules,&p,250);
            let b=session(&mut Pack::new(7),&rules,&p,250);
            assert_eq!(a.ending,b.ending); assert_eq!(a.rounds,b.rounds); assert!(a.ending>=0.0);
        }
    }
    #[test] fn zero_success_wilson_upper_is_not_zero() {assert!(wilson(0,500)[1]>0.0);}
}
