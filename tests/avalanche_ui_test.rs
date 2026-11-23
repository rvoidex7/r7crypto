//! Headless interaction tests for the Avalanche tab.
//!
//! These drive the real widgets — tab button, message field, input-bit slider
//! and Run button — and assert that the displayed result comes from the SHA-256
//! analyzer rather than from a canned value.

#![cfg(feature = "gui")]

use egui::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use r7crypto::crypto::AvalancheAnalyzer;
use r7crypto::viz::{R7CryptoApp, Tab};

const DEFAULT_MESSAGE: &str = "r7 Crypto Lab Avalanche Benchmark";

fn avalanche_harness() -> Harness<'static, R7CryptoApp> {
    Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Avalanche))
}

#[test]
fn avalanche_tab_is_reachable_from_the_tab_bar() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Use));

    harness.run_steps(2);
    assert_eq!(harness.state().current_tab(), Tab::Use);

    harness.get_by_label("🔀 Avalanche").click();
    harness.run();

    assert_eq!(harness.state().current_tab(), Tab::Avalanche);
    assert!(harness
        .query_by_label_contains("Avalanche Effect")
        .is_some());
}

#[test]
fn run_button_computes_single_bit_result_and_statistics() {
    let mut harness = avalanche_harness();
    harness.run_steps(2);
    assert!(harness.state().avalanche_result().is_none());

    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();

    let result = harness
        .state()
        .avalanche_result()
        .expect("Run should produce a result")
        .clone();
    assert_eq!(result.flipped_bit, 0);
    assert_eq!(
        result.original,
        AvalancheAnalyzer::sha256(DEFAULT_MESSAGE.as_bytes())
    );
    assert_ne!(result.original, result.modified);
    assert!((80..=176).contains(&result.changed_bits));
    assert!((result.flip_percentage - 50.0).abs() < 20.0);

    let stats = harness
        .state()
        .avalanche_stats()
        .expect("Run should produce statistics");
    assert_eq!(stats.samples, 200);
    assert!((stats.mean_flip_percentage - 50.0).abs() < 3.0);

    assert!(harness
        .query_by_label_contains("output bits changed")
        .is_some());
    assert!(harness
        .query_by_label_contains("Output bits (16 × 16)")
        .is_some());
}

#[test]
fn input_bit_slider_drives_the_flipped_bit() {
    let mut harness = avalanche_harness();
    harness.run_steps(2);
    assert_eq!(harness.state().avalanche_bit(), 0);

    harness.get_by_role(Role::Slider).set_value_accesskit(40.0);
    harness.run();
    assert_eq!(harness.state().avalanche_bit(), 40);

    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();

    assert_eq!(
        harness.state().avalanche_result().unwrap().flipped_bit,
        40,
        "the displayed experiment must use the selected input bit"
    );
    assert!(
        harness.query_by_label_contains("Bit 40 flipped").is_some(),
        "the result label must name the selected bit"
    );
}

#[test]
fn changing_the_message_clears_stale_results() {
    let mut harness = avalanche_harness();
    harness.run_steps(2);

    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();
    assert!(harness.state().avalanche_result().is_some());
    assert!(harness.state().avalanche_stats().is_some());

    harness
        .get_by_role(Role::TextInput)
        .set_text_value_accesskit("A different message");
    harness.run();

    assert_eq!(harness.state().avalanche_input(), "A different message");
    assert!(harness.state().avalanche_result().is_none());
    assert!(harness.state().avalanche_stats().is_none());
}

#[test]
fn empty_message_shows_a_hint_instead_of_crashing() {
    let mut harness = avalanche_harness();
    harness.run_steps(2);

    harness
        .get_by_role(Role::TextInput)
        .set_text_value_accesskit("");
    harness.run();

    assert_eq!(harness.state().avalanche_input(), "");
    assert!(harness
        .query_by_label_contains("Type a message to analyze")
        .is_some());
}

#[test]
fn running_twice_produces_the_same_result() {
    let mut harness = avalanche_harness();
    harness.run_steps(2);

    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();
    let first = harness
        .state()
        .avalanche_result()
        .expect("first run")
        .clone();

    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();
    let second = harness
        .state()
        .avalanche_result()
        .expect("second run")
        .clone();

    assert_eq!(first, second);
    assert_eq!(
        harness.state().avalanche_stats(),
        Some(&AvalancheAnalyzer::analyze_sha256(
            DEFAULT_MESSAGE.as_bytes()
        ))
    );
}

#[test]
fn shortening_the_message_clamps_the_selected_bit_and_drops_stale_results() {
    let mut harness = avalanche_harness();
    harness.run_steps(2);

    harness.get_by_role(Role::Slider).set_value_accesskit(40.0);
    harness.run();
    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();
    assert!(harness.state().avalanche_result().is_some());

    harness
        .get_by_role(Role::TextInput)
        .set_text_value_accesskit("A");
    harness.run();

    assert_eq!(
        harness.state().avalanche_bit(),
        7,
        "a one-byte message has eight bits, so bit 40 must clamp to the last bit"
    );
    assert!(
        harness.state().avalanche_result().is_none(),
        "a result for bit 40 must not survive a one-byte message"
    );
    assert!(harness.state().avalanche_stats().is_none());
}
