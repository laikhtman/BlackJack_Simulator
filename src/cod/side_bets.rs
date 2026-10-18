//! COD side bets use physical ranks and suits; a J and Q are not a pair.
use serde::Serialize;
use std::collections::BTreeMap;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalCard {
    rank: u8,
    suit: u8,
}

impl PhysicalCard {
    /// Rank: A=1, J=11, Q=12, K=13. Suit: clubs=0, diamonds=1, hearts=2, spades=3.
    pub fn new(rank: u8, suit: u8) -> Result<Self, &'static str> {
        if !(1..=13).contains(&rank) || suit > 3 {
            return Err("Invalid rank or suit");
        }
        Ok(Self { rank, suit })
    }

    pub fn value(self) -> u8 {
        self.rank.min(10)
    }

    fn red(self) -> bool {
        self.suit == 1 || self.suit == 2
    }
}

impl FromStr for PhysicalCard {
    type Err = &'static str;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim().to_ascii_uppercase();
        let (rank, suit) = text.split_at_checked(text.len().saturating_sub(1))
            .ok_or("Use card notation such as AS, 10H, QC")?;
        let suit = match suit {
            "C" => 0, "D" => 1, "H" => 2, "S" => 3,
            _ => return Err("Suit must be C, D, H, or S"),
        };
        let rank = match rank {
            "A" => 1, "J" => 11, "Q" => 12, "K" => 13,
            _ => rank.parse::<u8>().map_err(|_| "Invalid rank")?,
        };
        Self::new(rank, suit)
    }
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub struct Award {
    pub category: &'static str,
    /// Net profit per unit staked; a loss is -1. Winning stakes are also returned.
    pub profit_units: i32,
}

fn award(category: &'static str, profit_units: i32) -> Award {
    Award { category, profit_units }
}

pub fn pair_fect(player: [PhysicalCard; 2], dealer: PhysicalCard) -> Award {
    let [a, b] = player;
    if a.rank != b.rank {
        award("loss", -1)
    } else if a == b && b == dealer {
        award("suited_three_of_a_kind", 50)
    } else if a.suit == b.suit {
        award("suited_pair", 30)
    } else if a.red() == b.red() {
        award("prime_pair", 10)
    } else {
        award("any_pair", 5)
    }
}

pub fn cod_21(player: [PhysicalCard; 2], dealer: PhysicalCard) -> Award {
    let cards = [player[0], player[1], dealer];
    let suited = cards.iter().all(|c| c.suit == cards[0].suit);
    let mut values = cards.map(|c| c.value());
    values.sort_unstable();
    if values == [7, 7, 7] {
        return if suited { award("suited_777", 200) } else { award("unsuited_777", 50) };
    }
    if values == [6, 7, 8] {
        return if suited { award("suited_678", 100) } else { award("unsuited_678", 30) };
    }
    let mut total: u8 = values.iter().sum();
    if values.contains(&1) && total + 10 <= 21 {
        total += 10;
    }
    match total {
        21 if suited => award("suited_21", 10),
        21 => award("unsuited_21", 3),
        20 => award("total_20", 2),
        19 => award("total_19", 2),
        _ => award("loss", -1),
    }
}

pub fn card_types() -> Vec<PhysicalCard> {
    (1..=13).flat_map(|rank| (0..4).map(move |suit| PhysicalCard { rank, suit })).collect()
}

#[derive(Debug, Serialize)]
pub struct CategoryProbability {
    pub ordered_outcomes: u64,
    pub probability: f64,
    pub profit_units: i32,
}

#[derive(Debug, Serialize)]
pub struct ExactBet {
    pub categories: BTreeMap<&'static str, CategoryProbability>,
    pub expected_profit_per_unit: f64,
    pub house_edge: f64,
    pub return_to_player: f64,
    pub standard_deviation_units: f64,
}

#[derive(Debug, Serialize)]
pub struct ExactReport {
    pub decks: u32,
    pub model: &'static str,
    pub ordered_outcomes: u64,
    pub pair_fect_pairs: ExactBet,
    pub cod_21_plus: ExactBet,
    pub covariance_per_unit_stake: f64,
}

fn summarize(counts: BTreeMap<&'static str, (u64, i32)>, total: u64) -> ExactBet {
    let mut mean = 0.0;
    let mut second_moment = 0.0;
    let categories = counts.into_iter().map(|(category, (count, profit))| {
        let probability = count as f64 / total as f64;
        mean += probability * profit as f64;
        second_moment += probability * (profit as f64).powi(2);
        (category, CategoryProbability { ordered_outcomes: count, probability, profit_units: profit })
    }).collect();
    ExactBet {
        categories, expected_profit_per_unit: mean, house_edge: -mean,
        return_to_player: 1.0 + mean,
        standard_deviation_units: (second_moment - mean * mean).max(0.0).sqrt(),
    }
}

/// Enumerate ordered draws without replacement, weighted by physical card copies.
/// This is a uniformly mixed full pack reference, not a particular CSM mechanism.
pub fn exact_probabilities(decks: u32) -> Result<ExactReport, &'static str> {
    if !(1..=8).contains(&decks) {
        return Err("Exact enumeration supports 1 through 8 decks");
    }
    let types = card_types();
    let mut pairs = BTreeMap::new();
    let mut totals = BTreeMap::new();
    let mut cross_moment = 0.0;
    let mut counted = 0u64;
    for (i, &a) in types.iter().enumerate() {
        for (j, &b) in types.iter().enumerate() {
            let remaining_b = decks as u64 - u64::from(i == j);
            if remaining_b == 0 { continue; }
            for (k, &d) in types.iter().enumerate() {
                let removed = u64::from(k == i) + u64::from(k == j);
                let Some(remaining_d) = (decks as u64).checked_sub(removed) else { continue; };
                let weight = decks as u64 * remaining_b * remaining_d;
                if weight == 0 { continue; }
                let p = pair_fect([a, b], d);
                let t = cod_21([a, b], d);
                let entry = pairs.entry(p.category).or_insert((0, p.profit_units));
                entry.0 += weight;
                let entry = totals.entry(t.category).or_insert((0, t.profit_units));
                entry.0 += weight;
                cross_moment += weight as f64 * p.profit_units as f64 * t.profit_units as f64;
                counted += weight;
            }
        }
    }
    let cards = decks as u64 * 52;
    let total = cards * (cards - 1) * (cards - 2);
    assert_eq!(counted, total);
    let pairs = summarize(pairs, total);
    let totals = summarize(totals, total);
    let covariance = cross_moment / total as f64
        - pairs.expected_profit_per_unit * totals.expected_profit_per_unit;
    Ok(ExactReport {
        decks, model: "uniform_full_pack_without_replacement", ordered_outcomes: total,
        pair_fect_pairs: pairs, cod_21_plus: totals, covariance_per_unit_stake: covariance,
    })
}
