//! Widget-level tests for the "Use" tab.
//!
//! These drive real text fields and buttons: typed input must reach the
//! ChaCha20 cipher, the displayed hex must match the library result, bad input
//! must be visible as an error, and identical inputs must produce identical
//! output.

#![cfg(feature = "gui")]

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use r7crypto::crypto::KeyedStreamCipher;
use r7crypto::viz::{R7CryptoApp, Tab};

const NONCE: &str = "000102030405060708090a0b";
const PASSPHRASE: &str = "test-passphrase";

fn use_harness() -> Harness<'static, R7CryptoApp> {
    Harness::builder()
        .with_size(egui::vec2(800.0, 1200.0))
        .allow_missing_glyphs()
        .with_max_steps(20_000)
        .build_eframe(|_cc| R7CryptoApp::for_tab(Tab::Use))
}

fn type_into(harness: &mut Harness<'_, R7CryptoApp>, label: &str, value: &str) {
    harness.get_by_label(label).set_text_value_accesskit(value);
    harness.run_steps(2);
}

/// Use fixed key material so the expected output is computable.
fn configure(harness: &mut Harness<'_, R7CryptoApp>) {
    harness.run_steps(2);
    type_into(harness, "Passphrase", PASSPHRASE);
    type_into(harness, "Nonce (96 bits, 24 hex characters)", NONCE);
}

fn field_value(harness: &Harness<'_, R7CryptoApp>, label: &str) -> String {
    harness
        .get_by_label(label)
        .value()
        .unwrap_or_else(|| panic!("{label} field should expose its value"))
}

#[test]
fn typed_utf8_input_produces_the_library_ciphertext() {
    let mut harness = use_harness();
    configure(&mut harness);
    let text = "Merhaba dünya! 🔐";
    type_into(&mut harness, "Input", text);

    harness.get_by_label("🔒 Encrypt").click();
    harness.run_steps(2);

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&hex::decode(NONCE).unwrap());
    let expected = hex::encode(KeyedStreamCipher::process_with_passphrase(
        PASSPHRASE,
        &nonce,
        text.as_bytes(),
    ));
    assert_eq!(field_value(&harness, "Ciphertext (hex)"), expected);
    assert!(
        harness
            .query_by_label_contains(&format!("Output length: {} bytes", text.len()))
            .is_some(),
        "the byte count, not the character count, must be shown"
    );
}

#[test]
fn encrypt_is_deterministic_for_the_same_key_nonce_and_input() {
    let mut harness = use_harness();
    configure(&mut harness);
    type_into(&mut harness, "Input", "determinism");

    harness.get_by_label("🔒 Encrypt").click();
    harness.run_steps(2);
    let first = field_value(&harness, "Ciphertext (hex)");

    harness.get_by_label("🔒 Encrypt").click();
    harness.run_steps(2);
    let second = field_value(&harness, "Ciphertext (hex)");

    assert!(!first.is_empty());
    assert_eq!(first, second);
}

#[test]
fn new_nonce_button_generates_a_fresh_24_hex_character_value() {
    let mut harness = use_harness();
    configure(&mut harness);

    harness.get_by_label("🎲 New").click();
    harness.run_steps(2);

    let nonce = field_value(&harness, "Nonce (96 bits, 24 hex characters)");
    assert_ne!(nonce, NONCE);
    assert_eq!(nonce.len(), 24);
    assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(hex::decode(&nonce).unwrap().len(), 12);
}

#[test]
fn wrong_nonce_shows_a_visible_warning_instead_of_silent_garbage() {
    let mut harness = use_harness();
    configure(&mut harness);
    type_into(&mut harness, "Input", "gizli mesaj");
    harness.get_by_label("🔒 Encrypt").click();
    harness.run_steps(2);
    let ciphertext = field_value(&harness, "Ciphertext (hex)");

    harness.get_by_label("↩ Use output as input").click();
    harness.run_steps(2);
    assert_eq!(field_value(&harness, "Input"), ciphertext);

    type_into(
        &mut harness,
        "Nonce (96 bits, 24 hex characters)",
        "ffffffffffffffffffffffff",
    );
    harness.get_by_label("🔓 Decrypt").click();
    harness.run_steps(2);

    assert!(
        harness.query_by_label_contains("not valid UTF-8").is_some(),
        "a wrong nonce must surface a visible warning"
    );
}

#[test]
fn invalid_hex_decrypt_shows_a_visible_error_and_clears_output() {
    let mut harness = use_harness();
    configure(&mut harness);
    type_into(&mut harness, "Input", "zz not hex");
    harness.get_by_label("🔓 Decrypt").click();
    harness.run_steps(2);

    assert!(
        harness
            .query_by_label_contains("Invalid hex ciphertext")
            .is_some(),
        "invalid hex must be reported in the UI"
    );
    assert_eq!(field_value(&harness, "Output"), "");
}

#[test]
fn learn_button_carries_the_input_into_the_xor_lesson() {
    let mut harness = use_harness();
    configure(&mut harness);
    type_into(&mut harness, "Input", "şifre");

    harness.get_by_label("Learn how this works").click();
    harness.run_steps(2);

    assert_eq!(harness.state().current_tab(), Tab::Learn);
    assert!(harness
        .query_by_label_contains("Learn Cryptography: XOR Cipher")
        .is_some());
    // "ş" is two UTF-8 bytes, so the visualization has six steps.
    assert!(
        harness.query_by_label_contains("Step 1 of 6").is_some(),
        "the lesson must start on the typed text, byte by byte"
    );
}
