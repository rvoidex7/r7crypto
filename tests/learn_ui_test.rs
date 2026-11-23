//! Widget-level tests for the "Learn" tab.
//!
//! All three lessons are exercised through their visible controls: mode
//! switches, text input, start/calculate/analyze buttons, step navigation and
//! play/pause. The assertions check the displayed step numbers and computed
//! values, not just that a heading exists.

#![cfg(feature = "gui")]

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use r7crypto::viz::{R7CryptoApp, Tab};

fn learn_harness() -> Harness<'static, R7CryptoApp> {
    Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Learn))
}

fn type_input(harness: &mut Harness<'_, R7CryptoApp>, value: &str) {
    harness
        .get_by_label("Input:")
        .set_text_value_accesskit(value);
    harness.run_steps(2);
}

fn click(harness: &mut Harness<'_, R7CryptoApp>, label: &str) {
    harness.get_by_label(label).click();
    harness.run_steps(2);
}

#[test]
fn learn_tab_switches_between_all_three_lessons() {
    let mut harness = learn_harness();
    harness.run_steps(2);

    assert!(harness
        .query_by_label_contains("Learn Cryptography: XOR Cipher")
        .is_some());

    click(&mut harness, "Shannon Entropy");
    assert!(harness
        .query_by_label_contains("Learn Shannon Entropy")
        .is_some());

    click(&mut harness, "NIST Frequency");
    assert!(harness
        .query_by_label_contains("Learn NIST Frequency Test")
        .is_some());

    click(&mut harness, "XOR Cipher");
    assert!(harness
        .query_by_label_contains("Learn Cryptography: XOR Cipher")
        .is_some());
}

#[test]
fn xor_lesson_start_step_navigation_and_play_toggle() {
    let mut harness = learn_harness();
    type_input(&mut harness, "AB");

    click(&mut harness, "Start Visualization");
    assert!(
        harness.query_by_label_contains("Step 1 of 2").is_some(),
        "two ASCII bytes must produce two steps"
    );

    click(&mut harness, "Next ➡");
    assert!(harness.query_by_label_contains("Step 2 of 2").is_some());

    click(&mut harness, "⬅ Previous");
    assert!(harness.query_by_label_contains("Step 1 of 2").is_some());

    click(&mut harness, "▶ Play");
    assert!(harness.query_by_label_contains("⏸ Pause").is_some());
    click(&mut harness, "⏸ Pause");
    assert!(harness.query_by_label_contains("▶ Play").is_some());
}

#[test]
fn entropy_lesson_calculates_the_shannon_value_and_navigates_steps() {
    let mut harness = learn_harness();
    click(&mut harness, "Shannon Entropy");
    type_input(&mut harness, "AAABBC");

    click(&mut harness, "Calculate");
    assert!(harness.query_by_label_contains("Step 1 of 5").is_some());

    for step in 2..=4 {
        click(&mut harness, "Next ➡");
        assert!(harness
            .query_by_label_contains(&step_label(step, 5))
            .is_some());
    }

    // A=3, B=2, C=1 over six bytes. The total appears from the Sum step on.
    let expected: f64 = [3.0f64 / 6.0, 2.0 / 6.0, 1.0 / 6.0]
        .into_iter()
        .map(|p| -p * p.log2())
        .sum();
    let label = format!("Total Entropy: {expected:.4} bits/byte");
    assert!(
        harness.query_by_label_contains(&label).is_some(),
        "the displayed entropy must match the byte frequency calculation: {label}"
    );

    click(&mut harness, "Next ➡");
    assert!(harness.query_by_label_contains(&step_label(5, 5)).is_some());
    assert!(
        harness
            .query_by_label_contains("Low entropy: the input is very predictable.")
            .is_some(),
        "the final interpretation step must classify the result"
    );
}

fn step_label(current: usize, total: usize) -> String {
    format!("Step {current} of {total}")
}

#[test]
fn nist_lesson_analyzes_bits_and_reaches_a_verdict() {
    let mut harness = learn_harness();
    click(&mut harness, "NIST Frequency");
    type_input(&mut harness, "AAABBC");

    click(&mut harness, "Analyze");
    assert!(harness.query_by_label_contains("Step 1 of 5").is_some());
    assert!(harness.query_by_label_contains("Bit Sequence:").is_some());

    click(&mut harness, "Next ➡");
    assert!(harness
        .query_by_label_contains("Count Ones and Zeros")
        .is_some());
    assert!(harness.query_by_label_contains("Ones:").is_some());

    for _ in 0..3 {
        click(&mut harness, "Next ➡");
    }
    assert!(
        harness
            .query_by_label_contains("✅ PASS: the sequence looks random.")
            .is_some()
            || harness
                .query_by_label_contains("❌ FAIL: the sequence has too many 1s or 0s.")
                .is_some(),
        "the final step must show a threshold verdict"
    );
    assert!(harness
        .query_by_label_contains("(Threshold: P-value ≥ 0.01)")
        .is_some());
}

#[test]
fn nist_generate_random_button_produces_a_new_walkthrough() {
    let mut harness = learn_harness();
    click(&mut harness, "NIST Frequency");

    click(&mut harness, "Generate Random");
    harness.run();

    assert!(
        harness.query_by_label_contains("Step 1 of 5").is_some(),
        "generated bytes must immediately start a fresh five-step walkthrough"
    );
}

#[test]
fn empty_inputs_show_prompts_without_starting_a_process() {
    let mut harness = learn_harness();
    harness.run_steps(2);

    for (lesson, button) in [
        ("XOR Cipher", "Start Visualization"),
        ("Shannon Entropy", "Calculate"),
        ("NIST Frequency", "Analyze"),
    ] {
        click(&mut harness, lesson);
        type_input(&mut harness, "");
        click(&mut harness, button);
        harness.run();
        assert!(
            harness.query_by_label_contains("Step 1 of").is_none(),
            "{lesson} must not advance on empty input"
        );
    }
}
