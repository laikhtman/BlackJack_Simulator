use freebj::cod::{DealerMode, SimulationConfig, simulate, simulate_with_progress};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

fn interact(args: &[&str], input: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cod-research"))
        .args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains('\x1b'), "Piped output must not include terminal escape codes");
    text
}

fn setup() -> String {
    "1\n1000\n6\n1,3,6\n50\n50\n5\n5\n1\nyes\n42\nyes\n".into()
}

fn temporary(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("cod-ui-{}-{}-{name}.json", std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))
}

#[test]
fn opens_by_default_and_handles_exit_and_eof() {
    let text = interact(&[], "0\n");
    assert!(text.contains("COD LIMASSOL"));
    assert!(text.contains("Configure simulation"));
    assert!(text.contains("Goodbye"));
    let text = interact(&["interactive"], "1\n1000\n");
    assert!(text.contains("Goodbye"), "EOF mid-wizard should exit cleanly");
}

#[test]
fn shows_exact_odds_and_recovers_from_invalid_hand() {
    let text = interact(&[], "3\n\n4\nwrong\n7H 7H 7H\n5\n\n0\n");
    assert!(text.contains("5.3729%"));
    assert!(text.contains("5.3054%"));
    assert!(text.contains("exactly three cards"));
    assert!(text.contains("net EUR +1000.00 | cash returned EUR 1005.00"));
    assert!(text.contains("net EUR +250.00 | cash returned EUR 255.00"));
}

#[test]
fn validates_wizard_inputs_and_requires_explicit_unknown_rules() {
    let text = interact(&[], concat!(
        "2\n1\n1000\n6\n1,1\n1,3,6\nNaN\n50\n2501\n50\n5\n5\n",
        "\n1\n\nyes\n42\nyes\n0\n"
    ));
    assert!(text.contains("configure a simulation first"));
    assert!(text.contains("distinct positions"));
    assert!(text.contains("finite amount"));
    assert!(text.contains("between 50.00 and 2500.00"));
    assert!(text.contains("no assumed default"));
    assert!(text.contains("Enter yes or no"));
    assert!(text.contains("Opening stake per round: EUR 180.00"));
}

#[test]
fn runs_exports_and_loads_a_setup_without_changing_math() {
    let profile = temporary("setup");
    let report = temporary("simulation");
    let input = format!("{}5\n1\n{}\n1\nq\n2\ns\n{}\n0\n", setup(), profile.display(), report.display());
    let text = interact(&[], &input);
    assert!(text.contains("previous setup is unchanged"));
    assert!(text.contains("Completed 1,000 rounds"));
    assert!(text.contains("95% interval for the mean"));
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    let expected = simulate(SimulationConfig {
        rounds: 1000, seed: 42, occupied_boxes: 6, owned_boxes: vec![1,3,6],
        table_minimum_eur: 50.0, main_bet_eur_per_box: 50.0,
        pair_fect_eur_per_box: 5.0, cod_21_eur_per_box: 5.0,
        double_after_split: true, dealer_mode: DealerMode::EuropeanAllBetsLost,
    }).unwrap();
    // Compare both results through the same JSON float encoding/decoding path.
    let expected: serde_json::Value = serde_json::from_str(&serde_json::to_string(&expected).unwrap()).unwrap();
    assert_eq!(saved, expected);
    let input = format!("5\n2\n{}\nyes\n0\n", profile.display());
    assert!(interact(&[], &input).contains("Setup loaded."));
    std::fs::remove_file(profile).unwrap();
    std::fs::remove_file(report).unwrap();
}

#[test]
fn saving_reports_does_not_overwrite_existing_experiments() {
    let path = temporary("existing");
    std::fs::write(&path, "keep original").unwrap();
    let input = format!("3\ns\n{}\n0\n", path.display());
    let text = interact(&[], &input);
    assert!(text.contains("Could not complete that action"));
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "keep original");
    std::fs::remove_file(path).unwrap();
}

#[test]
fn progress_reports_completed_rounds_without_affecting_results() {
    let c = SimulationConfig {
        rounds: 203, seed: 123, occupied_boxes: 1, owned_boxes: vec![1],
        table_minimum_eur: 50.0, main_bet_eur_per_box: 50.0,
        pair_fect_eur_per_box: 0.0, cod_21_eur_per_box: 0.0,
        double_after_split: false, dealer_mode: DealerMode::AmericanPeek,
    };
    let mut progress = Vec::new();
    let report = simulate_with_progress(c.clone(), |done, total| progress.push((done,total))).unwrap();
    assert_eq!(progress[0], (0,203));
    assert_eq!(*progress.last().unwrap(), (203,203));
    assert!(progress.windows(2).all(|p| p[0].0 < p[1].0));
    assert_eq!(serde_json::to_value(report).unwrap(), serde_json::to_value(simulate(c).unwrap()).unwrap());
}
