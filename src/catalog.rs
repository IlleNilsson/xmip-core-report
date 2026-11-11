//! Every Report in one list, each saying whether it is quick or takes
//! longer (ADR-0072, amendment 2026-11-10).
//!
//! A Report presents existing data; a Quest probes for data to present in
//! a Report (`doc/terminology.md`, *Report and Quest*). A Report of what
//! Xmip already keeps — the audit, the Dead Message Queue, the
//! configuration — is quick; one of what a Quest probes first takes longer.
//! Which a Report is follows from what it holds, its [`Data`]: nothing
//! declares a kind of its own and nothing reads one from a name. Xmip's own
//! are [`Reports::standard`], and a module adds a Report or a Quest with
//! [`Reports::with`]. A Report not built yet is listed and says so.

use abi::operate::{audit, dead_message, publication, report, subscription};
use serde_json::Value;

use crate::quest::Installation;
use crate::regulation::RegulationReport;

/// What a Report presents: Xmip's own fixed answer, so a closed set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Data that already exists: quick.
    ExistingData,
    /// What a Quest probes first: it takes longer.
    Quest,
}

impl Kind {
    /// Both, in the order a list shows them.
    pub const ALL: [Self; 2] = [Self::ExistingData, Self::Quest];

    /// Its number, `XmipReportKind` in `xmip_operate.h` section 25.
    #[must_use]
    pub const fn number(self) -> i32 {
        match self {
            Self::ExistingData => report::kind::EXISTING_DATA,
            Self::Quest => report::kind::QUEST,
        }
    }

    /// Its word, as an answer names it.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::ExistingData => "existing-data",
            Self::Quest => "quest",
        }
    }

    /// Its name, as a surface shows it beside its icon.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ExistingData => "Quick",
            Self::Quest => "Takes longer",
        }
    }

    /// What it means, as a surface's legend says it.
    #[must_use]
    pub const fn meaning(self) -> &'static str {
        match self {
            Self::ExistingData => "Presents data that already exists",
            Self::Quest => "Presents what a Quest probes first",
        }
    }
}

/// A Report: its name, what it covers, and what it holds.
pub trait Report: Send + Sync {
    /// The name an operator chooses it by: `Audit`, `Regulation`.
    fn name(&self) -> &str;

    /// What it covers, in a sentence.
    fn covers(&self) -> &str;

    /// What it holds, which its kind and whether it is available follow
    /// from.
    fn data(&self) -> Data<'_>;
}

/// What a Report holds.
#[derive(Clone, Copy)]
pub enum Data<'r> {
    /// Data that already exists, presented by the reader whose
    /// `xmip_operate.h` entrypoint this names: `xmip_audit_read_v1`. `None`
    /// where no reader presents it yet.
    Existing(Option<&'r str>),
    /// What a Quest probes. `None` where no Quest probes it yet.
    Quest(Option<&'r dyn Probe>),
}

impl Data<'_> {
    /// The kind it makes its Report.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Existing(_) => Kind::ExistingData,
            Self::Quest(_) => Kind::Quest,
        }
    }

    /// Whether its Report can be opened: a reader presents it, or a Quest
    /// probes it.
    #[must_use]
    pub const fn available(&self) -> bool {
        matches!(self, Self::Existing(Some(_)) | Self::Quest(Some(_)))
    }
}

/// A Quest as the list holds it and a surface opens it: whatever it finds,
/// answered at the level a surface asks for. A Quest's own `Found` differs
/// from one Quest to the next, so the list holds this in its place.
pub trait Probe: Send + Sync {
    /// What its parameter is called and the choices it takes —
    /// `("regulation", ["GDPR"])` — or none where it takes none.
    fn parameter(&self) -> Option<(&str, Vec<&str>)>;

    /// What one row of its second level is called: `requirement`.
    fn row(&self) -> &str;

    /// Its levels, the first first.
    fn levels(&self) -> &[&str];

    /// Probe `installation` for the choice `parameter`, answering the first
    /// two levels, and the third for the row named `row` where one is: a
    /// JSON object, which a surface shows member by member where it holds
    /// the members `xmip_operate.h` section 25 names — a `title`, a `tally`
    /// of `{"state","count"}`, `rows` and one `requirement` — and passes on
    /// whole. [`crate::answer::probe_of`] has held `parameter` to its
    /// choices.
    ///
    /// # Errors
    /// A choice or a row that is none, in a sentence naming those there
    /// are.
    fn open(
        &self,
        installation: &dyn Installation,
        parameter: Option<&str>,
        row: Option<&str>,
    ) -> Result<Value, String>;
}

/// When each Report's Quest last probed, as whoever keeps that record says:
/// handed in (ADR-0068), so the list reads no store of its own.
pub trait LastProbes {
    /// The last probe of the Report named `report`: how long it took, in
    /// milliseconds, and when, in RFC 3339; none where it was never probed.
    fn last(&self, report: &str) -> Option<(u64, String)>;

    /// Why the last probes could not be read, in one sentence opening
    /// FAILED, where they could not; every Report is listed all the same.
    fn unread(&self) -> Option<&str> {
        None
    }
}

/// One of Xmip's own Reports, other than the Regulation Report.
#[derive(Clone, Copy)]
struct Own {
    name: &'static str,
    covers: &'static str,
    data: Data<'static>,
}

impl Report for Own {
    fn name(&self) -> &'static str {
        self.name
    }

    fn covers(&self) -> &'static str {
        self.covers
    }

    fn data(&self) -> Data<'_> {
        self.data
    }
}

/// What a Quest covers that is named and not designed yet.
const UNDESIGNED: &str = "What it probes is not designed yet.";

/// Xmip's own quick Reports, in the order the list shows them.
const QUICK: [Own; 6] = [
    Own {
        name: "Audit",
        covers: "What every Xmip program recorded, with each record's severity (information, \
                 warning or error), action and phase, over a time range.",
        data: Data::Existing(Some(audit::AUDIT_READ_ENTRYPOINT)),
    },
    Own {
        name: "Dead Messages",
        covers: "The Messages no Subscription matched, kept in each node's Dead Message Queue.",
        data: Data::Existing(Some(dead_message::DEAD_MESSAGES_ENTRYPOINT)),
    },
    Own {
        name: "Locations",
        covers: "Every Receive and Send Location each node runs, as configured.",
        data: Data::Existing(Some(publication::READ_ENTRYPOINT)),
    },
    Own {
        name: "Subscriptions",
        covers: "The Subscriptions each node routes by, and whether each is paused.",
        data: Data::Existing(Some(subscription::SUBSCRIPTIONS_ENTRYPOINT)),
    },
    Own {
        name: "Parties",
        covers: "The Parties each node recognizes, with the identities each is recognized by \
                 and presented with.",
        data: Data::Existing(None),
    },
    Own {
        name: "Expiring certificates and secrets",
        covers: "The certificates and secrets each node holds, and when each expires.",
        data: Data::Existing(None),
    },
];

/// Xmip's own Reports a Quest probes for after the Regulation Report, in
/// the order the list shows them: none built yet. The Identity and
/// isolation Report is computed from each identity context and the
/// isolation rules evaluated on it (ADR-0022 clause 5), so a Quest probes
/// for it.
const QUESTS: [Own; 9] = [
    Own {
        name: "Firewall",
        covers: "Every TCP and UDP port, every path a file-based Location touches and every \
                 protocol a node opens or reaches, inbound and outbound.",
        data: Data::Quest(None),
    },
    Own {
        name: "Identity and isolation",
        covers: "Every identity context, its class and where it runs, and every isolation \
                 rule evaluated on it with its result.",
        data: Data::Quest(None),
    },
    Own {
        name: "Security",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
    Own {
        name: "Sanity",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
    Own {
        name: "Performance",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
    Own {
        name: "Redundancy",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
    Own {
        name: "Fail Over",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
    Own {
        name: "Size",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
    Own {
        name: "Compile Time",
        covers: UNDESIGNED,
        data: Data::Quest(None),
    },
];

/// Every Report: Xmip's own and every one a module adds.
#[derive(Default)]
pub struct Reports {
    held: Vec<Box<dyn Report>>,
}

impl Reports {
    /// Xmip's own: the quick ones, then the Regulation Report, then the
    /// other Quests.
    #[must_use]
    pub fn standard() -> Self {
        let own = |own: &Own| Box::new(*own) as Box<dyn Report>;
        let mut held: Vec<Box<dyn Report>> = QUICK.iter().map(own).collect();
        held.push(Box::new(RegulationReport::default()));
        held.extend(QUESTS.iter().map(own));
        Self { held }
    }

    /// The same list with `report` added after every one held.
    ///
    /// # Errors
    /// Where a Report of its name is already held, saying so.
    pub fn with(mut self, report: Box<dyn Report>) -> Result<Self, String> {
        self.hold(report)?;
        Ok(self)
    }

    /// Add `report` after every one held.
    ///
    /// # Errors
    /// Where a Report of its name is already held, saying so; nothing is
    /// added.
    pub fn hold(&mut self, report: Box<dyn Report>) -> Result<(), String> {
        if self.named(report.name()).is_some() {
            return Err(format!(
                "a Report named '{}' is already held",
                report.name()
            ));
        }
        self.held.push(report);
        Ok(())
    }

    /// The Report named `name`, where one is held.
    #[must_use]
    pub fn named(&self, name: &str) -> Option<&dyn Report> {
        self.iter().find(|held| held.name() == name)
    }

    /// Every Report, in the order the list shows them.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Report> {
        self.held.iter().map(AsRef::as_ref)
    }
}

#[cfg(test)]
mod tests;
