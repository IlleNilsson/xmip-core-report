use serde_json::json;
use xcore::StreamId;

use crate::finding::{Behind, Finding, State};
use crate::fixture::Composed;
use crate::quest::{Installation, Quest};
use crate::regulation::{Regulation, RegulationQuest, Regulations, Requirement, answered, behind};

/// A regulation a module adds: one requirement, not applicable anywhere.
struct Added;

struct Nowhere;

impl Requirement for Nowhere {
    fn name(&self) -> &'static str {
        "Nowhere"
    }

    fn citation(&self) -> &'static str {
        "Article 1"
    }

    fn probe(&self, _: &dyn Installation) -> Finding {
        Finding::not_applicable("asks nothing of a node", Vec::new())
    }
}

impl Regulation for Added {
    fn name(&self) -> &'static str {
        "Added"
    }

    fn title(&self) -> &'static str {
        "A regulation a module adds"
    }

    fn requirements(&self) -> &[&dyn Requirement] {
        &[&Nowhere]
    }
}

#[test]
fn a_module_adds_a_regulation_and_the_quest_takes_it_by_name() {
    let regulations = Regulations::standard()
        .with(Box::new(Added))
        .expect("added");
    assert_eq!(regulations.names().collect::<Vec<_>>(), ["GDPR", "Added"]);
    let quest = RegulationQuest::of(&regulations, "Added").expect("held");
    assert_eq!(quest.name(), "Regulation");
    let probed = quest.probe(&Composed::of(""));
    assert_eq!(probed.tally().not_applicable, 1);
    let rows: Vec<_> = probed.rows().collect();
    assert_eq!(rows[0].state, State::NotApplicable);
    assert_eq!(
        probed.requirement("Nowhere").expect("found").finding.says(),
        "asks nothing of a node"
    );
    assert!(probed.requirement("Elsewhere").is_none());
}

#[test]
fn a_second_regulation_of_one_name_is_refused() {
    let refused = Regulations::default()
        .with(Box::new(Added))
        .and_then(|held| held.with(Box::new(Added)))
        .err()
        .expect("refused");
    assert!(refused.contains("'Added'"), "{refused}");
}

#[test]
fn the_report_answers_its_tally_in_the_states_words_and_each_behind_its_kind() {
    let regulations = Regulations::standard()
        .with(Box::new(Added))
        .expect("added");
    let probed = RegulationQuest::of(&regulations, "Added")
        .expect("held")
        .probe(&Composed::of(""));
    let found = answered(&probed, Some("Nowhere")).expect("answered");
    assert_eq!(
        found["tally"],
        json!([
            { "state": "evidence", "count": 0 },
            { "state": "gap", "count": 0 },
            { "state": "not applicable", "count": 1 },
        ])
    );
    assert_eq!(found["requirement"]["state"], "not applicable");
    let refused = answered(&probed, Some("Elsewhere")).expect_err("no such row");
    assert!(refused.contains("there are: Nowhere"), "{refused}");
    assert_eq!(
        behind(&Behind::Stream(StreamId::new(1))),
        json!({ "kind": "stream", "stream": StreamId::new(1).to_string() })
    );
}

#[test]
fn an_unknown_regulation_is_refused_naming_those_there_are() {
    let regulations = Regulations::standard();
    let refused = RegulationQuest::of(&regulations, "NIS2")
        .err()
        .expect("refused");
    assert!(
        refused.contains("'NIS2'") && refused.contains("GDPR"),
        "{refused}"
    );
}
