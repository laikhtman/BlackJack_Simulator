use clap::{App, Arg, SubCommand};
use freebj::cod::{DealerMode, SimulationConfig, side_bets, simulate};
use std::str::FromStr;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let matches = App::new("cod-research")
        .about("COD Limassol research scenarios and exact side bet probabilities")
        .subcommand(SubCommand::with_name("exact")
            .about("Exact side bet probabilities for a uniformly mixed six-deck pack"))
        .subcommand(SubCommand::with_name("hand")
            .about("Evaluate player card 1, player card 2, dealer upcard")
            .arg(Arg::with_name("cards").required(true).number_of_values(3)))
        .subcommand(SubCommand::with_name("simulate")
            .about("Combined main and side wagers under an explicit CSM reference scenario")
            .arg(Arg::with_name("rounds").long("rounds").takes_value(true).default_value("100000"))
            .arg(Arg::with_name("seed").long("seed").takes_value(true).default_value("42"))
            .arg(Arg::with_name("occupied").long("occupied-boxes").takes_value(true).default_value("6"))
            .arg(Arg::with_name("owned").long("owned-boxes").takes_value(true).default_value("1"))
            .arg(Arg::with_name("minimum").long("table-minimum").takes_value(true).default_value("50"))
            .arg(Arg::with_name("main").long("main-bet").takes_value(true).default_value("50"))
            .arg(Arg::with_name("pair").long("pair-fect-bet").takes_value(true).default_value("0"))
            .arg(Arg::with_name("total").long("cod21-bet").takes_value(true).default_value("0"))
            .arg(Arg::with_name("dealer").long("dealer-mode").takes_value(true).required(true)
                .possible_values(&["ahc-peek", "enhc-all-bets"]))
            .arg(Arg::with_name("das").long("double-after-split").takes_value(true).required(true)
                .possible_values(&["yes", "no"])))
        .get_matches();
    let result = match matches.subcommand() {
        ("exact", _) => serde_json::to_value(side_bets::exact_probabilities(6)?),
        ("hand", Some(args)) => {
            let cards: Result<Vec<_>, _> = args.values_of("cards").unwrap()
                .map(side_bets::PhysicalCard::from_str).collect();
            let cards = cards?;
            Ok(serde_json::json!({
                "pair_fect_pairs": side_bets::pair_fect([cards[0], cards[1]], cards[2]),
                "cod_21_plus": side_bets::cod_21([cards[0], cards[1]], cards[2])
            }))
        }
        ("simulate", Some(args)) => {
            fn parse<T: FromStr>(args: &clap::ArgMatches, name: &str) -> Result<T, String> {
                args.value_of(name).unwrap().parse().map_err(|_| format!("Invalid {name}"))
            }
            let owned: Result<Vec<u8>, _> = args.value_of("owned").unwrap().split(',')
                .map(|s| s.parse::<u8>()).collect();
            let config = SimulationConfig {
                rounds: parse(args, "rounds")?, seed: parse(args, "seed")?,
                occupied_boxes: parse(args, "occupied")?,
                owned_boxes: owned.map_err(|_| "Use comma-separated owned positions, e.g. 1,3,6")?,
                table_minimum_eur: parse(args, "minimum")?, main_bet_eur_per_box: parse(args, "main")?,
                pair_fect_eur_per_box: parse(args, "pair")?, cod_21_eur_per_box: parse(args, "total")?,
                double_after_split: args.value_of("das") == Some("yes"),
                dealer_mode: if args.value_of("dealer") == Some("ahc-peek") {
                    DealerMode::AmericanPeek
                } else { DealerMode::EuropeanAllBetsLost },
            };
            serde_json::to_value(simulate(config)?)
        }
        _ => return Err("Choose exact, hand, or simulate; see --help".into()),
    }.map_err(|e| e.to_string())?;
    println!("{}", serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?);
    Ok(())
}
