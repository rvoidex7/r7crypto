//! Deep links from r7notes land on the exact screen, and every screen points
//! back at its exact note and implementation range.

#![cfg(feature = "gui")]

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use r7crypto::viz::{LearnMode, R7CryptoApp, Tab};

#[test]
fn fragments_select_tabs_without_a_harness() {
    for (fragment, tab) in [
        ("#/use", Tab::Use),
        ("#/test", Tab::Test),
        ("#/benchmark", Tab::Benchmark),
        ("#/avalanche", Tab::Avalanche),
        ("#/learn/nist-frequency", Tab::Learn),
    ] {
        let app = R7CryptoApp::from_fragment(fragment);
        assert_eq!(app.current_tab(), tab, "fragment: {fragment}");
    }
}

#[test]
fn nist_fragment_reaches_the_named_lesson_and_its_links() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .allow_missing_glyphs()
        .build_eframe(|_cc| R7CryptoApp::from_fragment("#/learn/nist-frequency"));

    harness.run_steps(2);
    assert_eq!(harness.state().current_tab(), Tab::Learn);
    assert_eq!(harness.state().learn_mode(), LearnMode::NistFrequency);

    harness.get_by_label("Learn NIST Frequency Test");
    harness.get_by_label("Notes: NIST Tests");
    harness.get_by_label("Code: src/learn/nist_visual.rs#L52-L137");
    harness.get_by_label("Code: src/quality/nist.rs#L24-L48");
}

#[test]
fn each_screen_links_to_its_own_note_and_code() {
    let cases = [
        (
            "#/use",
            "Notes: ChaCha20",
            "Code: src/crypto/cipher.rs#L107-L143",
        ),
        (
            "#/test",
            "Notes: Entropy",
            "Code: src/quality/metrics.rs#L78-L199",
        ),
        (
            "#/benchmark",
            "Notes: RNG sources",
            "Code: src/bench/performance.rs#L48-L117",
        ),
        (
            "#/avalanche",
            "Notes: Avalanche Effect",
            "Code: src/crypto/avalanche.rs#L58-L143",
        ),
        (
            "#/learn/shannon-entropy",
            "Notes: Entropy",
            "Code: src/learn/entropy_visual.rs#L66-L161",
        ),
        (
            "#/learn/xor-cipher",
            "Notes: ChaCha20",
            "Code: src/learn/steps.rs#L20-L85",
        ),
    ];

    for (fragment, note_label, code_label) in cases {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(900.0, 700.0))
            .allow_missing_glyphs()
            .build_eframe(|_cc| R7CryptoApp::from_fragment(fragment));
        harness.run_steps(2);
        harness.get_by_label(note_label);
        harness.get_by_label(code_label);
    }
}
