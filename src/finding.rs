//! What a probe of one requirement found (ADR-0072): its state, what it
//! says, the Locations, Streams and settings behind it, and the actions
//! that close a gap.
//!
//! A finding presents evidence and gaps. It never judges whether an
//! installation meets a regulation: that is the customer's legal judgment
//! (the owner, 2026-11-08).

use xcore::StreamId;
use xcore::settings::Applies;

/// A requirement's state: Xmip's own fixed answer, so a closed set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    /// The probe found evidence that the installation does what the
    /// requirement asks of it.
    Evidence,
    /// The probe found no evidence, or found the opposite.
    Gap,
    /// The requirement asks nothing of this installation.
    NotApplicable,
}

impl State {
    /// The state's words, as a surface shows them.
    #[must_use]
    pub const fn words(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Gap => "gap",
            Self::NotApplicable => "not applicable",
        }
    }
}

/// What a finding rests on: a Location, a Stream or a setting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Behind {
    /// A Receive (`side` [`Applies::Receive`]) or Send Location, by name.
    Location { side: Applies, name: String },
    /// A Stream, by its identifier.
    Stream(StreamId),
    /// A setting, by its key in the node's configuration — `store.key_store`
    /// — and the value it has there.
    Setting { key: String, value: String },
}

/// An action that closes a gap. The operator applies it; a Quest never
/// does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// A change to the node's configuration: the setting, and what to make
    /// it say.
    Configure { setting: String, says: String },
    /// Something done outside Xmip, by whoever owns it: IT's database
    /// server, for one.
    Outside { says: String },
    /// What would close the gap is not yet available in Xmip.
    NotYetAvailable { says: String },
}

/// What a probe of one requirement found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    state: State,
    says: String,
    behind: Vec<Behind>,
    actions: Vec<Action>,
}

impl Finding {
    /// Evidence, saying what was found, resting on `behind`.
    #[must_use]
    pub fn evidence(says: impl Into<String>, behind: Vec<Behind>) -> Self {
        Self::of(State::Evidence, says, behind, Vec::new())
    }

    /// A gap, saying what is missing, resting on `behind`, and the actions
    /// that close it: at least one, as a gap with none would leave the
    /// operator without a way on.
    ///
    /// # Panics
    ///
    /// Where `actions` is empty: a requirement's probe is written so.
    #[must_use]
    pub fn gap(says: impl Into<String>, behind: Vec<Behind>, actions: Vec<Action>) -> Self {
        assert!(!actions.is_empty(), "a gap names the actions that close it");
        Self::of(State::Gap, says, behind, actions)
    }

    /// Not applicable, saying why, resting on `behind`.
    #[must_use]
    pub fn not_applicable(says: impl Into<String>, behind: Vec<Behind>) -> Self {
        Self::of(State::NotApplicable, says, behind, Vec::new())
    }

    fn of(
        state: State,
        says: impl Into<String>,
        behind: Vec<Behind>,
        actions: Vec<Action>,
    ) -> Self {
        Self {
            state,
            says: says.into(),
            behind,
            actions,
        }
    }

    /// Its state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// What it found, in words.
    #[must_use]
    pub fn says(&self) -> &str {
        &self.says
    }

    /// The Locations, Streams and settings behind it.
    #[must_use]
    pub fn behind(&self) -> &[Behind] {
        &self.behind
    }

    /// The actions that close it: none unless it is a gap.
    #[must_use]
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_gap_carries_actions() {
        let evidence = Finding::evidence("sealed", Vec::new());
        assert_eq!(evidence.state(), State::Evidence);
        assert_eq!(evidence.actions(), []);
        let closing = Action::NotYetAvailable {
            says: "later".to_string(),
        };
        let gap = Finding::gap("missing", Vec::new(), vec![closing.clone()]);
        assert_eq!(gap.state(), State::Gap);
        assert_eq!(gap.actions(), [closing]);
        let none = Finding::not_applicable("no Location", Vec::new());
        assert_eq!(none.state().words(), "not applicable");
    }

    #[test]
    #[should_panic(expected = "a gap names the actions that close it")]
    fn a_gap_without_an_action_is_refused() {
        let _ = Finding::gap("missing", Vec::new(), Vec::new());
    }
}
