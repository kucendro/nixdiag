//! Static analysis of the documented repo's own files.
//!
//! Everything here reads *source text*, as opposed to `facts`, which is the
//! evaluated configuration. That split is the point of v2: eval supplies
//! which services exist and where they are defined, the source supplies what
//! they are *for*. Comments are invisible to eval, so annotations can only be
//! read here — and the renderer owns the repo source in both modes.
//!
//! `flakelock` is the same idea one level up: the flake's supply chain, read
//! straight out of `flake.lock` rather than from eval.

pub mod annotations;
pub mod doccomment;
pub mod flakelock;
pub mod imports;
pub mod repo;
