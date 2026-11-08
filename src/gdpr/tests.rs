use std::assert_matches;
use std::collections::BTreeMap;

use configure::{ConfiguredLocation, XmipConfigurationDocument, parse_toml};
use xcore::settings::Applies;

use crate::finding::{Action, Behind, State};
use crate::quest::{AtRest, Installation, Quest, Wire};
use crate::regulation::{Found, Probed, Regulation, RegulationQuest, Regulations, Tally};

const HEAD: &str = "[service]\nname = \"n\"\ncluster_name = \"c\"\nnode_name = \"d\"\n";

/// An installation as a test composes it: a configuration read from TOML,
/// each Location's wire by its name, and each domain kept as said.
struct Composed {
    configuration: XmipConfigurationDocument,
    wires: BTreeMap<&'static str, Wire>,
    at_rest: BTreeMap<&'static str, AtRest>,
}

impl Composed {
    fn of(tables: &str) -> Self {
        Self {
            configuration: parse_toml(&format!("{HEAD}{tables}")).expect("parses"),
            wires: BTreeMap::new(),
            at_rest: BTreeMap::new(),
        }
    }

    fn wire(mut self, name: &'static str, wire: Wire) -> Self {
        self.wires.insert(name, wire);
        self
    }

    fn kept(mut self, domain: &'static str, at_rest: AtRest) -> Self {
        self.at_rest.insert(domain, at_rest);
        self
    }

    fn probed(&self) -> Probed {
        let regulations = Regulations::standard();
        RegulationQuest::of(&regulations, "GDPR")
            .expect("held")
            .probe(self)
    }
}

impl Installation for Composed {
    fn configuration(&self) -> &XmipConfigurationDocument {
        &self.configuration
    }

    fn wire(&self, location: &ConfiguredLocation, _: Applies) -> Wire {
        self.wires
            .get(location.name.as_str())
            .copied()
            .unwrap_or(Wire::Undeclared)
    }

    fn at_rest(&self, domain: &str) -> AtRest {
        self.at_rest.get(domain).copied().unwrap_or(AtRest::Sealed)
    }
}

fn found<'p>(probed: &'p Probed, name: &str) -> &'p Found {
    probed.requirement(name).expect("a GDPR requirement")
}

const LOCATIONS: &str = "[[receive_locations]]\nname = \"in\"\naddress = \"https://h/in\"\n\
                         [[send_locations]]\nname = \"out\"\naddress = \"sftp://h/out\"\n";

#[test]
fn every_requirement_cites_its_article_and_reads_at_three_levels() {
    let probed = Composed::of(LOCATIONS)
        .wire("in", Wire::Encrypted)
        .wire("out", Wire::Encrypted)
        .probed();
    assert_eq!(probed.regulation(), "GDPR");
    assert!(probed.title().contains("2016/679"));
    let rows: Vec<_> = probed.rows().collect();
    assert_eq!(rows.len(), 9);
    assert!(rows.iter().all(|row| row.citation.contains("Article")));
    let tally = probed.tally();
    assert_eq!(
        tally,
        Tally {
            evidence: 3,
            gap: 6,
            not_applicable: 0,
        }
    );
    let named: Vec<&str> = rows.iter().map(|row| row.name).collect();
    let mut unique = named.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), named.len(), "a name is unique: {named:?}");
}

#[test]
fn what_xmip_cannot_support_yet_is_a_gap_that_says_so() {
    let probed = Composed::of("").probed();
    for name in [
        "Data minimization in logs and traces",
        "Storage limitation",
        "Right of access",
        "Right to erasure",
        "Records of processing activities",
        "Transfers outside the European Union",
    ] {
        let found = found(&probed, name);
        assert_eq!(found.finding.state(), State::Gap, "{name}");
        let [Action::NotYetAvailable { says }] = found.finding.actions() else {
            panic!("{name}: one action, not yet available");
        };
        assert!(says.ends_with("not yet available in Xmip."), "{says}");
    }
}

#[test]
fn records_sealed_on_the_node_are_evidence_naming_the_key_store() {
    let probed = Composed::of("[store]\nkey_store = \"xmip-core-secret-file\"\n").probed();
    let found = found(&probed, "Encryption at rest");
    assert_eq!(found.finding.state(), State::Evidence);
    assert!(
        found
            .finding
            .says()
            .contains("runtime, administration, audit")
    );
    assert_eq!(
        found.finding.behind(),
        [Behind::Setting {
            key: "store.key_store".to_string(),
            value: "xmip-core-secret-file".to_string(),
        }]
    );
    assert_eq!(found.finding.actions(), []);
}

#[test]
fn a_database_server_is_a_gap_closed_outside_xmip() {
    let probed = Composed::of(
        "[runtime]\nstorage = \"postgresql\"\nconnection = \"host=db-1 dbname=xmip_runtime\"\n",
    )
    .kept("runtime", AtRest::Server("postgresql"))
    .probed();
    let found = found(&probed, "Encryption at rest");
    assert_eq!(found.finding.state(), State::Gap);
    assert_eq!(
        found.finding.behind(),
        [Behind::Setting {
            key: "runtime.storage".to_string(),
            value: "postgresql".to_string(),
        }]
    );
    assert_matches!(found.finding.actions(), [Action::Outside { .. }]);
}

#[test]
fn a_node_keeping_nothing_has_no_encryption_at_rest_of_its_own() {
    let probed = Composed::of("[storage]\nnodes = [\"storage-1.example:7443\"]\n")
        .kept("runtime", AtRest::StorageNodes)
        .kept("administration", AtRest::StorageNodes)
        .kept("audit", AtRest::StorageNodes)
        .probed();
    let found = found(&probed, "Encryption at rest");
    assert_eq!(found.finding.state(), State::NotApplicable);
    assert_eq!(
        found.finding.behind(),
        [Behind::Setting {
            key: "storage.nodes".to_string(),
            value: "storage-1.example:7443".to_string(),
        }]
    );
}

#[test]
fn every_location_encrypted_is_evidence_naming_each() {
    let probed = Composed::of(LOCATIONS)
        .wire("in", Wire::Encrypted)
        .wire("out", Wire::Encrypted)
        .probed();
    let found = found(&probed, "Encryption in transit");
    assert_eq!(found.finding.state(), State::Evidence);
    assert_eq!(
        found.finding.behind(),
        [
            Behind::Location {
                side: Applies::Receive,
                name: "in".to_string(),
            },
            Behind::Location {
                side: Applies::Send,
                name: "out".to_string(),
            },
        ]
    );
}

#[test]
fn a_plain_or_undeclared_wire_is_a_gap_with_its_actions() {
    let probed = Composed::of(LOCATIONS).wire("in", Wire::Plain).probed();
    let found = found(&probed, "Encryption in transit");
    assert_eq!(found.finding.state(), State::Gap);
    assert_eq!(found.finding.behind().len(), 2);
    let [
        Action::Configure { setting, .. },
        Action::NotYetAvailable { says },
    ] = found.finding.actions()
    else {
        panic!("a change for the plain one, not yet for the undeclared");
    };
    assert_eq!(setting, "receive_locations.in");
    assert!(says.contains("out"), "{says}");
}

#[test]
fn a_node_without_locations_has_no_wire_to_probe() {
    let probed = Composed::of("").probed();
    let found = found(&probed, "Encryption in transit");
    assert_eq!(found.finding.state(), State::NotApplicable);
}

#[test]
fn the_audit_is_evidence_naming_where_it_is_kept() {
    let probed = Composed::of(
        "[audit]\nstorage = \"sqlite\"\nconnection = \"D:/Xmip/data/storage/audit.sqlite\"\n",
    )
    .probed();
    let found = found(&probed, "Accountability: the audit");
    assert_eq!(found.finding.state(), State::Evidence);
    assert_eq!(
        found.finding.behind(),
        [Behind::Setting {
            key: "audit.storage".to_string(),
            value: "sqlite".to_string(),
        }]
    );
}

#[test]
fn nothing_the_quest_says_judges_the_regulation() {
    let probed = Composed::of(LOCATIONS).probed();
    for row in probed.rows() {
        let found = found(&probed, row.name);
        let mut words = vec![row.name.to_string(), found.finding.says().to_string()];
        words.extend(
            found
                .finding
                .actions()
                .iter()
                .map(|action| format!("{action:?}")),
        );
        for said in words {
            let lower = said.to_lowercase();
            for verdict in ["complian", "conform", "certif"] {
                assert!(!lower.contains(verdict), "'{said}' judges");
            }
        }
    }
    assert!(
        !crate::gdpr::Gdpr
            .title()
            .to_lowercase()
            .contains("complian")
    );
}
