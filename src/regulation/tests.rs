use configure::{ConfiguredLocation, XmipConfigurationDocument, parse_toml};
use xcore::settings::Applies;

use crate::finding::{Finding, State};
use crate::quest::{AtRest, Installation, Quest, Wire};
use crate::regulation::{Regulation, RegulationQuest, Regulations, Requirement};

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

struct Bare(XmipConfigurationDocument);

impl Installation for Bare {
    fn configuration(&self) -> &XmipConfigurationDocument {
        &self.0
    }

    fn wire(&self, _: &ConfiguredLocation, _: Applies) -> Wire {
        Wire::Undeclared
    }

    fn at_rest(&self, _: &str) -> AtRest {
        AtRest::Sealed
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
    let document = parse_toml("[service]\nname = \"n\"\ncluster_name = \"c\"\nnode_name = \"d\"\n")
        .expect("parses");
    let probed = quest.probe(&Bare(document));
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
