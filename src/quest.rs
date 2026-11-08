//! The Quest seam (ADR-0072): what every Quest is, and what it probes.
//!
//! A Report presents existing data; a Quest probes for data to present in
//! a Report (the owner, 2026-11-07; `doc/terminology.md`, *Report and
//! Quest*). Every Quest — the Regulation Quest first, then the Security,
//! Sanity, Performance, Redundancy, Fail Over, Size, Compile Time and
//! Firewall Quests — is a [`Quest`], and probes an [`Installation`] handed to
//! it (ADR-0068). A Quest reads what it probes and changes nothing: the
//! actions it offers are the operator's to apply.

use configure::{ConfiguredLocation, XmipConfigurationDocument};
use xcore::settings::Applies;

/// A Quest: it probes an installation and returns what it found, the data
/// a Report presents.
pub trait Quest {
    /// What it found.
    type Found;

    /// Its name, as an operator chooses it: `Regulation`.
    fn name(&self) -> &str;

    /// Probe `installation`. It reads and changes nothing.
    fn probe(&self, installation: &dyn Installation) -> Self::Found;
}

/// What a Quest probes: one node's configuration, and what the capabilities
/// that own a fact say of it. Whoever composes — the runtime, a surface, a
/// test — implements it and hands it in (ADR-0068); a Quest never reads a
/// file or looks a technology up by name.
pub trait Installation {
    /// The node's configuration, as configure reads it.
    fn configuration(&self) -> &XmipConfigurationDocument;

    /// What `location`'s technology declares of its wire, on `side`.
    fn wire(&self, location: &ConfiguredLocation, side: Applies) -> Wire;

    /// How the data domain `domain` — `runtime`, `administration` or
    /// `audit`, as [`XmipConfigurationDocument::domains`] names it — is kept
    /// at rest, as Xmip Storage reads its table.
    fn at_rest(&self, domain: &str) -> AtRest;
}

/// What a Location's technology declares of its wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wire {
    /// The technology encrypts what crosses it, as the Location is entered.
    Encrypted,
    /// The technology sends it in the clear, as the Location is entered.
    Plain,
    /// The technology declares nothing of its wire.
    Undeclared,
}

/// How a data domain's records are kept at rest (ADR-0063, amendment
/// 2026-10-01).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtRest {
    /// On this node's embedded Storage node: every record sealed by Xmip
    /// before its engine sees it.
    Sealed,
    /// On a database server IT runs, named by its `storage` word: its
    /// encryption at rest is IT's.
    Server(&'static str),
    /// Not on this node: the Storage nodes it lists keep it.
    StorageNodes,
}
