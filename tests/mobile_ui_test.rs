//! Headless narrow-screen (360x800 logical pixels) rendering test.
//!
//! This catches desktop-only fixed widths and colliding `egui::Id`s. Android
//! itself is deferred: [Roadmap → Deferred](https://rvoidex7.github.io/r7notes/Projects/r7crypto/Roadmap#deferred).

#![cfg(feature = "gui")]

use egui_kittest::kittest::{NodeT, Queryable};
use egui_kittest::{Harness, Node};
use r7crypto::entropy::SourceKind;
use r7crypto::viz::{R7CryptoApp, Tab};

const VIEWPORT_WIDTH: f32 = 360.0;

/// Recursively assert that every widget stays inside the viewport width.
///
/// A widget whose bounding box crosses the right edge would be clipped on a
/// phone-sized screen, which is exactly the regression this test guards
/// against.
fn assert_within_viewport_width(node: &Node<'_>, path: &str, pixels_per_point: f32) {
    for (index, child) in node.children().enumerate() {
        let accesskit_node = child.accesskit_node();
        let label = accesskit_node.label().unwrap_or_default();
        let role = format!("{:?}", accesskit_node.role());

        // Some synthetic/ignored nodes have no bounds; only laid-out widgets
        // can overflow.
        if let Some(bounds) = accesskit_node.bounding_box() {
            let min_x = bounds.x0 as f32 / pixels_per_point;
            let max_x = bounds.x1 as f32 / pixels_per_point;
            assert!(
                min_x >= -0.5 && max_x <= VIEWPORT_WIDTH + 0.5,
                "{path}/{index} {role} {label:?} value={:?} exceeds the {VIEWPORT_WIDTH}px viewport: \
                 x = {min_x}..{max_x}",
                accesskit_node.value()
            );
        }

        assert_within_viewport_width(&child, &format!("{path}/{index}"), pixels_per_point);
    }
}

#[test]
fn mobile_360px_viewport_renders_all_tabs_without_overflow_or_id_clash() {
    for tab in [
        Tab::Use,
        Tab::Test,
        Tab::Benchmark,
        Tab::Learn,
        Tab::Avalanche,
    ] {
        let app = R7CryptoApp::for_tab(tab);
        let mut harness = Harness::builder()
            .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
            // Icon glyphs are drawn as tofu headlessly; the desktop build loads
            // system emoji fonts instead.
            .allow_missing_glyphs()
            .build_eframe(|_cc| app);

        // Run several frames so immediate-mode layout settles and duplicate
        // IDs would be reported.
        harness.run_steps(3);
        assert_eq!(harness.state().current_tab(), tab);

        let root = harness.root();
        let pixels_per_point = harness.ctx.pixels_per_point();
        assert_within_viewport_width(&root, &format!("{tab:?}"), pixels_per_point);
    }
}

#[test]
fn mobile_360px_header_with_bias_slider_does_not_overflow() {
    let mut app = R7CryptoApp::for_tab(Tab::Test);
    app.select_source(SourceKind::Biased);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
        .allow_missing_glyphs()
        .build_eframe(|_cc| app);

    harness.run_steps(3);
    assert_eq!(harness.state().source_kind(), SourceKind::Biased);

    let root = harness.root();
    let pixels_per_point = harness.ctx.pixels_per_point();
    assert_within_viewport_width(&root, "Biased", pixels_per_point);
}

#[test]
fn header_links_are_present_and_fit_360px() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
        .allow_missing_glyphs()
        .build_eframe(|_cc| R7CryptoApp::default());

    harness.run_steps(2);

    harness.get_by_label("Crypto notes");
    harness.get_by_label("Project docs");
    harness.get_by_label("Source");

    let root = harness.root();
    let pixels_per_point = harness.ctx.pixels_per_point();
    assert_within_viewport_width(&root, "Header links", pixels_per_point);
}

#[test]
fn mobile_360px_avalanche_grid_does_not_overflow() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
        .allow_missing_glyphs()
        .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Avalanche));

    harness.run_steps(2);
    harness.get_by_label("🔀 Run Avalanche").click();
    harness.run();
    assert!(harness.state().avalanche_result().is_some());

    let root = harness.root();
    let pixels_per_point = harness.ctx.pixels_per_point();
    assert_within_viewport_width(&root, "Avalanche", pixels_per_point);
}

#[test]
fn use_tab_encrypt_then_decrypt_round_trip_through_widgets() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
        .allow_missing_glyphs()
        .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Use));

    harness.run_steps(2);

    harness.get_by_label("🔒 Encrypt").click();
    harness.run_steps(2);

    let ciphertext = harness
        .get_by_label("Ciphertext (hex)")
        .value()
        .expect("ciphertext field should expose its value");
    assert_eq!(ciphertext.len(), "Hello, World!".len() * 2);
    assert!(ciphertext.chars().all(|c| c.is_ascii_hexdigit()));

    harness.get_by_label("↩ Use output as input").click();
    harness.run_steps(2);

    harness.get_by_label("🔓 Decrypt").click();
    harness.run_steps(2);

    let plaintext = harness
        .get_by_label("Plaintext")
        .value()
        .expect("plaintext field should expose its value");
    assert_eq!(plaintext, "Hello, World!");
}

#[test]
fn mobile_360px_learn_lessons_with_active_steps_do_not_overflow() {
    for (mode, button) in [
        ("XOR Cipher", "Start Visualization"),
        ("Shannon Entropy", "Calculate"),
        ("NIST Frequency", "Analyze"),
    ] {
        let mut harness = Harness::builder()
            .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
            .allow_missing_glyphs()
            .with_max_steps(20_000)
            .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Learn));

        harness.run_steps(2);
        harness.get_by_label(mode).click();
        harness.run_steps(2);
        harness
            .get_by_label("Input:")
            .set_text_value_accesskit("AAABBC");
        harness.run_steps(2);
        harness.get_by_label(button).click();
        harness.run();

        assert!(
            harness.query_by_label_contains("Step 1 of").is_some(),
            "{mode} should render its first step at phone width"
        );

        let root = harness.root();
        let pixels_per_point = harness.ctx.pixels_per_point();
        assert_within_viewport_width(&root, mode, pixels_per_point);
    }
}

#[test]
fn mobile_360px_test_results_do_not_overflow() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(VIEWPORT_WIDTH, 800.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| {
            let mut app = R7CryptoApp::for_tab(Tab::Test);
            app.select_source(SourceKind::Counter);
            app.set_sample_size(1_000);
            app
        });

    harness.run_steps(2);
    harness.get_by_label("🔬 Run All Tests").click();
    harness.run();
    assert!(harness.state().quality_metrics().is_some());

    let root = harness.root();
    let pixels_per_point = harness.ctx.pixels_per_point();
    assert_within_viewport_width(&root, "TestResults", pixels_per_point);
}
