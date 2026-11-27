//! Teaching state machines only; rendering lives in `viz`.
//! Layering rationale: https://rvoidex7.github.io/r7notes/Projects/r7crypto/Architecture#why-the-layers-exist

pub mod entropy_visual;
pub mod nist_visual;
pub mod steps;
pub mod xor_visual;

pub use entropy_visual::EntropyProcess;
pub use nist_visual::NistProcess;
pub use steps::{BitOperation, EncryptionStep};
pub use xor_visual::EncryptionProcess;
