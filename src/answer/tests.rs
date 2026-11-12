use serde_json::{Value, json};

use crate::answer::{NOT_PROBED, NOT_YET, UNREAD, listed, milliseconds, opened, probe_of};
use crate::catalog::{Data, LastProbes, Probe, Report, Reports};
use crate::fixture::{Composed, LOCATIONS};
use crate::quest::{AtRest, Installation, Wire};

/// The last probe of every Report, the one `took` says or none.
struct Kept(Option<u64>);

impl LastProbes for Kept {
    fn last(&self, _: &str) -> Option<(u64, String)> {
        self.0
            .map(|took| (took, "2026-11-10T12:00:00Z".to_string()))
    }
}

/// Last probes that could not be read.
struct Unread;

impl LastProbes for Unread {
    fn last(&self, _: &str) -> Option<(u64, String)> {
        None
    }

    fn unread(&self) -> Option<&str> {
        Some("FAILED: the audit could not be read.")
    }
}

fn list(kept: &dyn LastProbes) -> Value {
    serde_json::from_str(&listed(&Reports::standard(), kept, None).expect("listed")).expect("JSON")
}

fn named<'v>(list: &'v Value, name: &str) -> &'v Value {
    list["reports"]
        .as_array()
        .expect("reports")
        .iter()
        .find(|entry| entry["name"] == name)
        .expect("listed")
}

#[test]
fn the_list_says_each_kind_in_words_and_each_entry_its_kind() {
    let list = list(&Kept(None));
    assert_eq!(
        list["kinds"],
        json!([
            { "kind": 0, "word": "existing-data", "name": "Report",
              "meaning": "Presents data that already exists" },
            { "kind": 1, "word": "quest", "name": "Probing report",
              "meaning": "Presents what a Quest probes first" },
        ])
    );
    let audit = named(&list, "Audit");
    assert_eq!(audit["kind"], 0);
    assert_eq!(audit["available"], true);
    assert_eq!(audit["read_by"], "xmip_audit_read_v1");
    assert!(audit.get("expect").is_none() && audit.get("levels").is_none());
    let parties = named(&list, "Parties");
    assert_eq!(parties["available"], false);
    assert_eq!(parties["expect"], NOT_YET);
    assert!(parties.get("read_by").is_none());
    let regulation = named(&list, "Regulation");
    assert_eq!(regulation["kind"], 1);
    assert_eq!(
        regulation["parameter"],
        json!({ "name": "regulation", "choices": ["GDPR"] })
    );
    assert_eq!(regulation["row"], "requirement");
    assert_eq!(regulation["levels"].as_array().map(Vec::len), Some(3));
    assert_eq!(regulation["expect"], NOT_PROBED);
    let firewall = named(&list, "Firewall");
    assert_eq!(
        (&firewall["kind"], &firewall["expect"]),
        (&json!(1), &json!(NOT_YET))
    );
    assert!(firewall.get("parameter").is_none());
}

#[test]
fn the_time_to_expect_is_the_last_probes_where_one_was_made() {
    let probed = list(&Kept(Some(4)));
    let regulation = named(&probed, "Regulation");
    assert_eq!(regulation["took_ms"], 4);
    assert_eq!(regulation["probed_at"], "2026-11-10T12:00:00Z");
    assert_eq!(regulation["expect"], "last probe took 4 ms");
    for quick_or_unbuilt in ["Audit", "Firewall"] {
        assert!(named(&probed, quick_or_unbuilt).get("took_ms").is_none());
    }
    let instant = list(&Kept(Some(0)));
    assert_eq!(
        named(&instant, "Regulation")["expect"],
        "last probe took under 1 ms"
    );
    assert!(
        named(&list(&Kept(None)), "Regulation")
            .get("took_ms")
            .is_none()
    );
    assert!(list(&Kept(None)).get("unread").is_none());
}

#[test]
fn last_probes_that_cannot_be_read_leave_every_report_listed_and_say_why() {
    let list = list(&Unread);
    assert_eq!(
        list["reports"].as_array().map(Vec::len),
        Some(Reports::standard().iter().count())
    );
    assert_eq!(list["unread"], "FAILED: the audit could not be read.");
    assert_eq!(named(&list, "Regulation")["expect"], UNREAD);
    assert_eq!(named(&list, "Firewall")["expect"], NOT_YET);
}

#[test]
fn one_report_is_listed_by_its_name_or_refused_naming_those_there_are() {
    let reports = Reports::standard();
    let one: Value =
        serde_json::from_str(&listed(&reports, &Kept(None), Some("Audit")).expect("listed"))
            .expect("JSON");
    assert_eq!(one["reports"].as_array().map(Vec::len), Some(1));
    assert_eq!(one["reports"][0]["name"], "Audit");
    assert_eq!(one["kinds"].as_array().map(Vec::len), Some(2));
    let refused = listed(&reports, &Kept(None), Some("Nowhere")).expect_err("none");
    let opening = opened(&reports, "Nowhere", None, None, &installation()).expect_err("none");
    assert_eq!(refused, opening, "one refusal of a name");
    assert!(
        refused.starts_with("REFUSED: no Report is named 'Nowhere'; there are: Audit, "),
        "{refused}"
    );
}

/// A node with a Location in the clear, one undeclared, and its runtime
/// database on a server IT runs.
fn installation() -> Composed {
    Composed::of(LOCATIONS)
        .wire("in", Wire::Plain)
        .kept("runtime", AtRest::Server("postgresql"))
}

fn open(report: &str, parameter: Option<&str>, row: Option<&str>) -> Result<Value, String> {
    let reports = Reports::standard();
    opened(&reports, report, parameter, row, &installation()).map(|(text, took)| {
        let answer: Value = serde_json::from_str(&text).expect("JSON");
        assert_eq!(answer["took_ms"], milliseconds(took));
        answer
    })
}

#[test]
fn the_regulation_opens_at_its_tally_and_rows_and_one_requirement_in_full() {
    let answer = open("Regulation", Some("GDPR"), None).expect("opened");
    assert_eq!(
        (&answer["report"], &answer["parameter"], &answer["node"]),
        (&json!("Regulation"), &json!("GDPR"), &json!("d"))
    );
    assert_eq!(answer["row"], "requirement", "what a row is called");
    assert!(
        answer["title"]
            .as_str()
            .is_some_and(|title| title.contains("2016/679"))
    );
    assert_eq!(
        answer["tally"],
        json!([
            { "state": "evidence", "count": 1 },
            { "state": "gap", "count": 8 },
            { "state": "not applicable", "count": 0 },
        ])
    );
    let rows = answer["rows"].as_array().expect("rows");
    assert_eq!(rows.len(), 9);
    assert!(
        rows.iter()
            .all(|row| row["state"] == "gap" || row["state"] == "evidence")
    );
    assert!(answer.get("requirement").is_none());

    let transit = open("Regulation", Some("GDPR"), Some("Encryption in transit")).expect("one");
    let requirement = &transit["requirement"];
    assert_eq!(requirement["state"], "gap");
    assert_eq!(
        requirement["behind"][0],
        json!({ "kind": "location", "side": "receive", "name": "in" })
    );
    assert_eq!(requirement["actions"][0]["kind"], "configure");
    assert_eq!(requirement["actions"][0]["setting"], "receive_locations.in");
    assert_eq!(requirement["actions"][1]["kind"], "not-yet-available");
    assert_eq!(transit["rows"], answer["rows"], "the levels above it too");

    let rest = open("Regulation", Some("GDPR"), Some("Encryption at rest")).expect("one");
    assert_eq!(
        rest["requirement"]["behind"],
        json!([{ "kind": "setting", "key": "runtime.storage", "value": "postgresql" }])
    );
    assert_eq!(rest["requirement"]["actions"][0]["kind"], "outside");
}

/// A Quest a module adds that takes no parameter.
struct Plain;

impl Report for Plain {
    fn name(&self) -> &'static str {
        "Plain"
    }

    fn covers(&self) -> &'static str {
        "What a module probes."
    }

    fn data(&self) -> Data<'_> {
        Data::Quest(Some(self))
    }
}

impl Probe for Plain {
    fn parameter(&self) -> Option<(&str, Vec<&str>)> {
        None
    }

    fn row(&self) -> &'static str {
        "finding"
    }

    fn levels(&self) -> &[&str] {
        &["what it found"]
    }

    fn open(
        &self,
        _: &dyn Installation,
        _: Option<&str>,
        _: Option<&str>,
    ) -> Result<Value, String> {
        Ok(json!({ "found": true }))
    }
}

#[test]
fn a_quest_a_module_adds_is_opened_through_the_same_answer() {
    let reports = Reports::standard().with(Box::new(Plain)).expect("added");
    let (text, _) = opened(&reports, "Plain", None, None, &installation()).expect("opened");
    let answer: Value = serde_json::from_str(&text).expect("JSON");
    assert_eq!(
        (&answer["report"], &answer["found"], &answer["row"]),
        (&json!("Plain"), &json!(true), &json!("finding"))
    );
    assert!(answer.get("parameter").is_none());
    assert!(
        answer.get("tally").is_none() && answer.get("title").is_none(),
        "what it does not answer is not made up: no tally of zeros, no title"
    );
    let refused =
        opened(&reports, "Plain", Some("GDPR"), None, &installation()).expect_err("refused");
    assert!(refused.contains("takes no parameter"), "{refused}");
}

#[test]
fn what_cannot_be_opened_is_refused_in_one_sentence() {
    for (report, parameter, row, said) in [
        (
            "Nowhere",
            None,
            None,
            "no Report is named 'Nowhere'; there are: Audit, ",
        ),
        (
            "Firewall",
            None,
            None,
            "the Firewall Report is not yet available",
        ),
        ("Audit", None, None, "which xmip_audit_read_v1 reads"),
        (
            "Regulation",
            Some("NIS2"),
            None,
            "'NIS2' is no regulation; ",
        ),
        (
            "Regulation",
            None,
            None,
            "takes its regulation, one of: GDPR",
        ),
        (
            "Regulation",
            Some("GDPR"),
            Some("Elsewhere"),
            "no requirement of GDPR is named",
        ),
    ] {
        let refused = open(report, parameter, row).expect_err("refused");
        assert!(
            refused.starts_with("REFUSED: ") && refused.ends_with('.') && refused.contains(said),
            "{refused}"
        );
        if row.is_none() {
            let asked = probe_of(&Reports::standard(), report, parameter).err();
            assert_eq!(asked, Some(refused), "refused before any installation");
        }
    }
}
