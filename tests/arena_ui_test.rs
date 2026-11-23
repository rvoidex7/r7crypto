//! RNG Arena UI tests.
//!
//! These drive the real widgets: the header source selector, the bias slider,
//! and the Test / Benchmark buttons. They assert that the selected source
//! actually reaches the background job, not just the label.

#![cfg(feature = "gui")]

use egui::Role;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use r7crypto::entropy::SourceKind;
use r7crypto::viz::{R7CryptoApp, Tab};

/// Build a harness large enough that the result panels do not need scrolling
/// to be laid out.
fn arena_harness(tab: Tab) -> Harness<'static, R7CryptoApp> {
    Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(move |_cc| R7CryptoApp::for_tab(tab))
}

/// Open the source combo box and pick `label`.
fn choose_source(harness: &mut Harness<'_, R7CryptoApp>, label: &str) {
    harness.get_by_role(Role::ComboBox).click();
    harness.run();
    harness.get_by_label(label).click();
    harness.run();
}

#[test]
fn source_selector_switches_the_active_source() {
    let mut harness = arena_harness(Tab::Test);
    harness.run_steps(2);

    assert_eq!(harness.state().source_kind(), SourceKind::System);
    assert!(harness.query_by_label_contains("System RNG").is_some());

    choose_source(&mut harness, "Counter");

    assert_eq!(harness.state().source_kind(), SourceKind::Counter);
    assert!(
        harness
            .query_by_label_contains("Counter (incrementing byte)")
            .is_some(),
        "the Test tab should name the selected source"
    );
}

#[test]
fn every_source_can_be_selected_from_the_header() {
    let mut harness = arena_harness(Tab::Test);
    harness.run_steps(2);

    for kind in SourceKind::ALL {
        choose_source(&mut harness, kind.label());
        assert_eq!(harness.state().source_kind(), kind);
    }
}

#[test]
fn bias_slider_appears_only_for_the_biased_source() {
    let mut harness = arena_harness(Tab::Test);
    harness.run_steps(2);

    assert_eq!(
        harness.query_all_by_label_contains("P(1)").count(),
        0,
        "the system source has no bias parameter"
    );

    choose_source(&mut harness, "Biased coin");

    assert_eq!(harness.state().source_kind(), SourceKind::Biased);
    assert!(
        harness.query_all_by_label_contains("P(1)").count() > 0,
        "the biased source must expose its P(1) slider"
    );
}

#[test]
fn counter_source_test_run_shows_a_capped_score() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| {
            let mut app = R7CryptoApp::for_tab(Tab::Test);
            app.select_source(SourceKind::Counter);
            app.set_sample_size(50_000);
            app
        });

    harness.run_steps(2);
    harness.get_by_label("🔬 Run All Tests").click();
    // Keep stepping while the app requests repaints; the worker finishes in
    // parallel and the UI stops asking once the result is in.
    harness.run();

    let state = harness.state();
    assert!(!state.is_testing(), "the analysis should have finished");

    let metrics = state
        .quality_metrics()
        .expect("the Test tab should display metrics after a run");
    assert!(
        (metrics.shannon_entropy - 8.0).abs() < 0.01,
        "a counter has a perfect histogram, got {}",
        metrics.shannon_entropy
    );
    assert!(
        metrics.markov_min_entropy < 0.5,
        "a counter is fully predictable, got {}",
        metrics.markov_min_entropy
    );
    assert!(
        metrics.overall_score() <= 30.0,
        "the score must be capped, got {}",
        metrics.overall_score()
    );
}

#[test]
fn benchmark_button_runs_with_the_selected_source() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| {
            let mut app = R7CryptoApp::for_tab(Tab::Benchmark);
            app.select_source(SourceKind::Counter);
            app
        });

    harness.run_steps(2);
    harness.get_by_label("⚡ Run Benchmark").click();
    harness.run();

    let state = harness.state();
    assert!(
        !state.is_benchmarking(),
        "the benchmark should have finished"
    );

    let result = state
        .bench_result()
        .expect("the Benchmark tab should display a result");
    assert_eq!(result.bytes_generated, 1_000_000);
    assert!(result.throughput_mbps > 0.0);
}

#[test]
fn bias_slider_value_reaches_the_quality_job() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| {
            let mut app = R7CryptoApp::for_tab(Tab::Test);
            app.select_source(SourceKind::Biased);
            app.set_sample_size(50_000);
            app
        });

    harness.run_steps(2);
    harness
        .query_all_by_label("P(1)")
        .next()
        .expect("the biased source exposes a P(1) slider")
        .set_value_accesskit(0.95);
    harness.run_steps(2);
    assert!(
        (harness.state().bias_probability() - 0.95).abs() < 0.01,
        "the slider must update the app's bias value"
    );

    harness.get_by_label("🔬 Run All Tests").click();
    harness.run();

    let metrics = harness
        .state()
        .quality_metrics()
        .expect("biased source should still produce metrics");
    assert!(
        metrics.shannon_entropy < 3.0,
        "P(1)=0.95 should be visibly biased, got {} bits/byte",
        metrics.shannon_entropy
    );
    assert!(metrics.is_capped());
}

#[test]
fn sample_size_slider_reaches_the_quality_job() {
    let mut harness = arena_harness(Tab::Test);
    harness.run_steps(2);
    let default_size = harness.state().sample_size();

    harness
        .query_all_by_label("Sample size:")
        .next()
        .expect("the Test tab exposes a sample size slider")
        .set_value_accesskit(2_000.0);
    harness.run_steps(2);
    let selected = harness.state().sample_size();
    assert!(
        selected < default_size,
        "the slider must change the sample size"
    );

    harness.get_by_label("🔬 Run All Tests").click();
    harness.run();

    let metrics = harness
        .state()
        .quality_metrics()
        .expect("the Test tab should display metrics after a run");
    assert_eq!(
        metrics.total_bytes, selected,
        "the job must use the sample size chosen in the UI"
    );
}

#[test]
fn nist_results_table_shows_all_nine_named_tests_with_verdicts() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| {
            let mut app = R7CryptoApp::for_tab(Tab::Test);
            app.select_source(SourceKind::ChaCha20);
            app.set_sample_size(50_000);
            app
        });

    harness.run_steps(2);
    harness.get_by_label("🔬 Run All Tests").click();
    harness.run();

    assert!(harness
        .query_by_label_contains("P-values ≥ 0.01 indicate randomness (Pass)")
        .is_some());
    for name in [
        "Frequency Test",
        "Block Frequency Test",
        "Runs Test",
        "Longest Run Test",
        "Chi-Square Test",
        "Serial Test (p1)",
        "Serial Test (p2)",
        "Cumulative Sums (forward)",
        "Cumulative Sums (reverse)",
    ] {
        assert!(
            harness.query_by_label(name).is_some(),
            "the NIST table must name {name}"
        );
    }
    let passes = harness.query_all_by_label_contains("✓ Pass").count();
    let failures = harness.query_all_by_label_contains("✗ Fail").count();
    assert_eq!(
        passes + failures,
        9,
        "every NIST row must carry exactly one verdict"
    );
}

#[test]
fn changing_the_source_discards_the_previous_run() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| {
            let mut app = R7CryptoApp::for_tab(Tab::Test);
            app.select_source(SourceKind::Counter);
            app.set_sample_size(50_000);
            app
        });

    harness.run_steps(2);
    harness.get_by_label("🔬 Run All Tests").click();
    harness.run();
    assert!(harness.state().quality_metrics().is_some());

    choose_source(&mut harness, "System");

    assert_eq!(harness.state().source_kind(), SourceKind::System);
    assert!(
        harness.state().quality_metrics().is_none(),
        "stale results must not be attributed to the new source"
    );
}
