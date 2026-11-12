use abi::operate::{audit, dead_message, publication, subscription};

use crate::catalog::{Data, Kind, Report, Reports};

/// A Report a module adds, holding what the test gives it.
struct Added {
    name: &'static str,
    data: Data<'static>,
}

impl Report for Added {
    fn name(&self) -> &'static str {
        self.name
    }

    fn covers(&self) -> &'static str {
        "What a module reports."
    }

    fn data(&self) -> Data<'_> {
        self.data
    }
}

fn names(reports: &Reports) -> Vec<&str> {
    reports.iter().map(Report::name).collect()
}

#[test]
fn the_standard_list_holds_every_report_quick_first() {
    let reports = Reports::standard();
    assert_eq!(
        names(&reports),
        [
            "Audit",
            "Dead Messages",
            "Locations",
            "Subscriptions",
            "Parties",
            "Expiring certificates and secrets",
            "Regulation",
            "Firewall",
            "Identity and isolation",
            "Security",
            "Sanity",
            "Performance",
            "Redundancy",
            "Fail Over",
            "Size",
            "Compile Time",
        ]
    );
    let kinds: Vec<Kind> = reports.iter().map(|held| held.data().kind()).collect();
    assert_eq!(kinds[..6], [Kind::ExistingData; 6]);
    assert_eq!(kinds[6..], [Kind::Quest; 10]);
}

#[test]
fn the_kind_follows_from_what_a_report_holds() {
    let quest = Added {
        name: "Probed elsewhere",
        data: Data::Quest(None),
    };
    assert_eq!(quest.data().kind(), Kind::Quest);
    assert!(!quest.data().available());
    let existing = Added {
        name: "Kept elsewhere",
        data: Data::Existing(Some(audit::AUDIT_READ_ENTRYPOINT)),
    };
    assert_eq!(existing.data().kind(), Kind::ExistingData);
    assert!(existing.data().available());
    assert!(!Data::Existing(None).available());
    assert_eq!(Kind::Quest.word(), "quest");
    assert_eq!(Kind::ExistingData.name(), "Report");
    assert_eq!(Kind::Quest.name(), "Probing report");
}

#[test]
fn a_module_adds_a_report_after_xmips_and_a_second_of_one_name_is_refused() {
    let added = Reports::standard()
        .with(Box::new(Added {
            name: "A module's own",
            data: Data::Quest(None),
        }))
        .expect("added");
    assert_eq!(names(&added).last(), Some(&"A module's own"));
    let refused = added
        .with(Box::new(Added {
            name: "Audit",
            data: Data::Existing(None),
        }))
        .err()
        .expect("refused");
    assert!(refused.contains("'Audit'"), "{refused}");
    let mut held = Reports::default();
    held.hold(Box::new(Added {
        name: "One",
        data: Data::Existing(None),
    }))
    .expect("held");
    assert!(
        held.hold(Box::new(Added {
            name: "One",
            data: Data::Quest(None),
        }))
        .is_err()
    );
    assert_eq!(names(&held), ["One"], "nothing added by a refusal");
}

#[test]
fn every_quest_but_the_regulation_and_two_quick_reports_are_not_yet_available() {
    let reports = Reports::standard();
    let unavailable: Vec<&str> = reports
        .iter()
        .filter(|held| !held.data().available())
        .map(Report::name)
        .collect();
    assert_eq!(
        unavailable,
        [
            "Parties",
            "Expiring certificates and secrets",
            "Firewall",
            "Identity and isolation",
            "Security",
            "Sanity",
            "Performance",
            "Redundancy",
            "Fail Over",
            "Size",
            "Compile Time",
        ]
    );
    for (name, reader) in [
        ("Audit", audit::AUDIT_READ_ENTRYPOINT),
        ("Dead Messages", dead_message::DEAD_MESSAGES_ENTRYPOINT),
        ("Locations", publication::READ_ENTRYPOINT),
        ("Subscriptions", subscription::SUBSCRIPTIONS_ENTRYPOINT),
    ] {
        let held = reports.named(name).expect("listed");
        assert!(
            matches!(held.data(), Data::Existing(Some(read)) if read == reader),
            "{name} is read by {reader}"
        );
    }
    assert!(matches!(
        reports.named("Regulation").expect("listed").data(),
        Data::Quest(Some(_))
    ));
}

#[test]
fn no_report_says_an_installation_meets_anything() {
    for held in Reports::standard().iter() {
        for said in [held.name(), held.covers()] {
            let lower = said.to_lowercase();
            // "certified", not "certif": a Report of certificates is no
            // verdict (ADR-0072 clause 8).
            for verdict in ["complian", "conform", "certified"] {
                assert!(!lower.contains(verdict), "'{said}' judges");
            }
        }
    }
}

#[test]
fn what_a_report_covers_is_said_of_the_report_and_never_of_who_named_it() {
    for held in Reports::standard().iter() {
        let covers = held.covers();
        assert!(covers.ends_with('.'), "'{covers}' is a sentence");
        // Who decided what is the records'; a surface shows the Report.
        for history in ["the owner", "named by", "2026-"] {
            assert!(!covers.to_lowercase().contains(history), "'{covers}'");
        }
    }
}
