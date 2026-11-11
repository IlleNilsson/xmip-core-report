//! The Regulation Quest (ADR-0072): one Quest, the regulation its
//! parameter.
//!
//! A regulation is a [`Regulation`], its requirements each a
//! [`Requirement`] that cites its article and probes the installation. The
//! set is open: Xmip's own are [`Regulations::standard`], and a module adds
//! one by handing its own to [`Regulations::with`]. What the Quest finds
//! reads at three levels (the owner, 2026-11-08): the regulation with its
//! [`Tally`], its requirements one [`Row`] each, and one requirement,
//! [`Found`], with what is behind it and the actions that close it. The list
//! of every Report holds it as the Regulation Report, [`RegulationReport`]
//! (ADR-0072, amendment 2026-11-10), which presents what the Quest finds as
//! the JSON `xmip_operate.h` section 25 names: the worked example of a
//! Quest's answer, beside its Quest.

use serde_json::{Value, json};

use crate::catalog::{Data, Probe, Report};
use crate::finding::{Action, Behind, Finding, State};
use crate::gdpr::Gdpr;
use crate::quest::{Installation, Quest};

/// The Regulation Quest's name, and the Regulation Report's.
pub const NAME: &str = "Regulation";

/// The Regulation Report's levels (the owner, 2026-11-08).
const LEVELS: &[&str] = &[
    "the regulation, with its tally",
    "a row per requirement",
    "one requirement in full",
];

/// A regulation: its name, the parameter an operator gives the Quest, and
/// its requirements.
pub trait Regulation: Send + Sync {
    /// The name an operator chooses it by: `GDPR`.
    fn name(&self) -> &str;

    /// Its full title, as the law names itself.
    fn title(&self) -> &str;

    /// Its requirements, in the order the Quest shows them.
    fn requirements(&self) -> &[&dyn Requirement];
}

/// One of a regulation's requirements.
pub trait Requirement: Send + Sync {
    /// What it asks, in a few words: unique within its regulation.
    fn name(&self) -> &str;

    /// Where the regulation says it: `Article 32(1)(a)`.
    fn citation(&self) -> &str;

    /// Probe `installation` for it.
    fn probe(&self, installation: &dyn Installation) -> Finding;
}

/// The regulations a Quest may be given: Xmip's own and every one a module
/// adds.
#[derive(Default)]
pub struct Regulations {
    held: Vec<Box<dyn Regulation>>,
}

impl Regulations {
    /// Xmip's own regulations: GDPR.
    #[must_use]
    pub fn standard() -> Self {
        Self {
            held: vec![Box::new(Gdpr)],
        }
    }

    /// The same set with `regulation` added.
    ///
    /// # Errors
    ///
    /// Where a regulation of its name is already held, saying so.
    pub fn with(mut self, regulation: Box<dyn Regulation>) -> Result<Self, String> {
        if self.named(regulation.name()).is_some() {
            return Err(format!(
                "a regulation named '{}' is already held",
                regulation.name()
            ));
        }
        self.held.push(regulation);
        Ok(self)
    }

    /// The regulation named `name`, where one is held.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<&dyn Regulation> {
        self.held
            .iter()
            .find(|held| held.name() == name)
            .map(AsRef::as_ref)
    }

    /// Every regulation's name, in the order they were added.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.held.iter().map(|held| held.name())
    }
}

/// The Regulation Quest, given its regulation.
pub struct RegulationQuest<'r> {
    regulation: &'r dyn Regulation,
}

impl<'r> RegulationQuest<'r> {
    /// The Quest for the regulation named `name` among `regulations`.
    ///
    /// # Errors
    ///
    /// Where none is named so, naming those there are.
    pub fn of(regulations: &'r Regulations, name: &str) -> Result<Self, String> {
        regulations
            .named(name)
            .map(|regulation| Self { regulation })
            .ok_or_else(|| {
                let known: Vec<&str> = regulations.names().collect();
                format!(
                    "no regulation is named '{name}'; there are: {}",
                    known.join(", ")
                )
            })
    }
}

impl Quest for RegulationQuest<'_> {
    type Found = Probed;

    fn name(&self) -> &'static str {
        NAME
    }

    fn probe(&self, installation: &dyn Installation) -> Probed {
        Probed {
            regulation: self.regulation.name().to_string(),
            title: self.regulation.title().to_string(),
            found: self
                .regulation
                .requirements()
                .iter()
                .map(|requirement| Found {
                    name: requirement.name().to_string(),
                    citation: requirement.citation().to_string(),
                    finding: requirement.probe(installation),
                })
                .collect(),
        }
    }
}

/// What the Regulation Quest found: the first level, a regulation with its
/// tally, and the way to the other two.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Probed {
    regulation: String,
    title: String,
    found: Vec<Found>,
}

/// How many of a regulation's requirements are in each state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub evidence: usize,
    pub gap: usize,
    pub not_applicable: usize,
}

/// The second level: one requirement and its state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Row<'p> {
    pub name: &'p str,
    pub citation: &'p str,
    pub state: State,
}

/// The third level: one requirement, what was found, what is behind it and
/// the actions that close it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub name: String,
    pub citation: String,
    pub finding: Finding,
}

impl Probed {
    /// The regulation's name.
    #[must_use]
    pub fn regulation(&self) -> &str {
        &self.regulation
    }

    /// Its full title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// The first level's tally of evidence, gaps and not applicable.
    #[must_use]
    pub fn tally(&self) -> Tally {
        self.found
            .iter()
            .fold(Tally::default(), |mut tally, found| {
                match found.finding.state() {
                    State::Evidence => tally.evidence += 1,
                    State::Gap => tally.gap += 1,
                    State::NotApplicable => tally.not_applicable += 1,
                }
                tally
            })
    }

    /// The second level: a row per requirement, in the regulation's order.
    pub fn rows(&self) -> impl Iterator<Item = Row<'_>> {
        self.found.iter().map(|found| Row {
            name: &found.name,
            citation: &found.citation,
            state: found.finding.state(),
        })
    }

    /// The third level: the requirement named `name`, where there is one.
    #[must_use]
    pub fn requirement(&self, name: &str) -> Option<&Found> {
        self.found.iter().find(|found| found.name == name)
    }
}

/// The Regulation Report: what the Regulation Quest probes, its parameter
/// the regulation and a row of its second level a requirement.
pub struct RegulationReport {
    regulations: Regulations,
}

impl RegulationReport {
    /// The Report over `regulations`.
    #[must_use]
    pub const fn new(regulations: Regulations) -> Self {
        Self { regulations }
    }
}

impl Default for RegulationReport {
    /// Over Xmip's own regulations.
    fn default() -> Self {
        Self::new(Regulations::standard())
    }
}

impl Report for RegulationReport {
    fn name(&self) -> &'static str {
        NAME
    }

    fn covers(&self) -> &'static str {
        "For each requirement of a regulation, the evidence or the gap a node's configuration \
         shows, and the actions that close a gap."
    }

    fn data(&self) -> Data<'_> {
        Data::Quest(Some(self))
    }
}

impl Probe for RegulationReport {
    fn parameter(&self) -> Option<(&str, Vec<&str>)> {
        Some(("regulation", self.regulations.names().collect()))
    }

    fn row(&self) -> &'static str {
        "requirement"
    }

    fn levels(&self) -> &[&str] {
        LEVELS
    }

    fn open(
        &self,
        installation: &dyn Installation,
        parameter: Option<&str>,
        row: Option<&str>,
    ) -> Result<Value, String> {
        let quest = RegulationQuest::of(&self.regulations, parameter.unwrap_or_default())?;
        answered(&quest.probe(installation), row)
    }
}

/// What the Regulation Quest found: the regulation with its tally, each
/// state's words and count, and a row per requirement; and the requirement
/// named `row` in full where one is.
///
/// # Errors
/// Where no requirement is named `row`, naming those there are.
fn answered(probed: &Probed, row: Option<&str>) -> Result<Value, String> {
    let tally = probed.tally();
    let counted = [
        (State::Evidence, tally.evidence),
        (State::Gap, tally.gap),
        (State::NotApplicable, tally.not_applicable),
    ];
    let rows: Vec<Value> = probed
        .rows()
        .map(
            |row| json!({ "name": row.name, "citation": row.citation, "state": row.state.words() }),
        )
        .collect();
    let mut found = json!({
        "title": probed.title(),
        "tally": counted
            .iter()
            .map(|(state, count)| json!({ "state": state.words(), "count": count }))
            .collect::<Vec<_>>(),
        "rows": rows,
    });
    if let Some(name) = row {
        let one = probed.requirement(name).ok_or_else(|| {
            let names: Vec<&str> = probed.rows().map(|row| row.name).collect();
            format!(
                "no requirement of {} is named '{name}'; there are: {}",
                probed.regulation(),
                names.join(", ")
            )
        })?;
        found["requirement"] = requirement(one);
    }
    Ok(found)
}

/// One requirement in full: what was found, what is behind it and the
/// actions that close it.
fn requirement(found: &Found) -> Value {
    let finding = &found.finding;
    json!({
        "name": found.name, "citation": found.citation, "state": finding.state().words(),
        "says": finding.says(),
        "behind": finding.behind().iter().map(behind).collect::<Vec<_>>(),
        "actions": finding.actions().iter().map(action).collect::<Vec<_>>(),
    })
}

fn behind(behind: &Behind) -> Value {
    match behind {
        Behind::Location { side, name } => {
            json!({ "kind": "location", "side": side.word(), "name": name })
        }
        Behind::Stream(stream) => json!({ "kind": "stream", "stream": stream.to_string() }),
        Behind::Setting { key, value } => json!({ "kind": "setting", "key": key, "value": value }),
    }
}

fn action(action: &Action) -> Value {
    match action {
        Action::Configure { setting, says } => {
            json!({ "kind": "configure", "setting": setting, "says": says })
        }
        Action::Outside { says } => json!({ "kind": "outside", "says": says }),
        Action::NotYetAvailable { says } => json!({ "kind": "not-yet-available", "says": says }),
    }
}

#[cfg(test)]
mod tests;
