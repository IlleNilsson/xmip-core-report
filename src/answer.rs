//! The list of every Report, and one opened, as `xmip_operate.h` section 25
//! answers them (ADR-0072, amendment 2026-11-10): JSON in memory only
//! (ADR-0031 clause 2), shaped here once for every surface, as the audit's
//! page is shaped in the audit capability. A surface shows the words it is
//! given — a kind's name and meaning, what to expect of an entry, a state,
//! what a row is called — and keeps no list of its own. What a Quest found
//! is its own to shape (the Regulation Report's is in `regulation.rs`); this
//! adds what every opened Report says.

use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::catalog::{Data, Kind, LastProbes, Probe, Report, Reports};
use crate::quest::Installation;

/// What an entry not yet available says it is.
pub const NOT_YET: &str = "not yet available";

/// What a Quest's entry never probed says.
pub const NOT_PROBED: &str = "not probed yet";

/// What a Quest's entry says where its last probe could not be read.
pub const UNREAD: &str = "last probe could not be read";

/// Every Report in `reports`, each with its kind, and each Quest's last
/// probe as `last` says it; or, where `report` names one, that one alone.
/// Where `last` could not be read, every Report is listed all the same and
/// the answer says why, as `unread`.
///
/// # Errors
/// One sentence opening REFUSED where no Report is named `report`, naming
/// those there are.
pub fn listed(
    reports: &Reports,
    last: &dyn LastProbes,
    report: Option<&str>,
) -> Result<String, String> {
    let kinds: Vec<Value> = Kind::ALL
        .iter()
        .map(|kind| {
            json!({
                "kind": kind.number(), "word": kind.word(), "name": kind.name(),
                "meaning": kind.meaning(),
            })
        })
        .collect();
    let entries: Vec<Value> = match report {
        Some(report) => vec![entry(held(reports, report)?, last)],
        None => reports.iter().map(|held| entry(held, last)).collect(),
    };
    let mut answer = json!({ "kinds": kinds, "reports": entries });
    if let Some(unread) = last.unread() {
        answer["unread"] = json!(unread);
    }
    Ok(answer.to_string())
}

/// The Report named `report`: the one refusal of a name, which listing one
/// and opening one both give.
fn held<'r>(reports: &'r Reports, report: &str) -> Result<&'r dyn Report, String> {
    reports.named(report).ok_or_else(|| {
        let names: Vec<&str> = reports.iter().map(Report::name).collect();
        refused(&format!(
            "no Report is named '{report}'; there are: {}",
            names.join(", ")
        ))
    })
}

fn refused(said: &str) -> String {
    format!("REFUSED: {said}.")
}

/// One entry of the list.
fn entry(report: &dyn Report, last: &dyn LastProbes) -> Value {
    let data = report.data();
    let mut said = json!({
        "name": report.name(), "covers": report.covers(), "kind": data.kind().number(),
        "available": data.available(),
    });
    match data {
        Data::Existing(Some(reader)) => said["read_by"] = json!(reader),
        Data::Existing(None) | Data::Quest(None) => said["expect"] = json!(NOT_YET),
        Data::Quest(Some(probe)) => {
            if let Some((name, choices)) = probe.parameter() {
                said["parameter"] = json!({ "name": name, "choices": choices });
            }
            said["row"] = json!(probe.row());
            said["levels"] = json!(probe.levels());
            said["expect"] = json!(if last.unread().is_some() {
                UNREAD
            } else {
                NOT_PROBED
            });
            if let Some((took, at)) = last.last(report.name()) {
                said["took_ms"] = json!(took);
                said["probed_at"] = json!(at);
                said["expect"] = json!(expected(took));
            }
        }
    }
    said
}

/// What to expect of a Quest whose last probe took `took` milliseconds.
fn expected(took: u64) -> String {
    if took == 0 {
        "last probe took under 1 ms".to_string()
    } else {
        format!("last probe took {took} ms")
    }
}

/// The Quest that probes for the Report named `report`, given `parameter`:
/// everything opening it asks that needs no installation, so a caller asks
/// it before it reads one.
///
/// # Errors
/// One sentence opening REFUSED: no Report is named so, naming those there
/// are; it is not yet available; it is quick, naming the reader that
/// presents it; `parameter` is missing, or none of its choices, or given to
/// a Report that takes none.
pub fn probe_of<'r>(
    reports: &'r Reports,
    report: &str,
    parameter: Option<&str>,
) -> Result<&'r dyn Probe, String> {
    let probe = match held(reports, report)?.data() {
        Data::Quest(Some(probe)) => probe,
        Data::Existing(Some(reader)) => {
            return Err(refused(&format!(
                "the {report} Report presents data that already exists, which {reader} reads; \
                 no Quest probes for it"
            )));
        }
        Data::Existing(None) | Data::Quest(None) => {
            return Err(refused(&format!("the {report} Report is {NOT_YET}")));
        }
    };
    match (probe.parameter(), parameter) {
        (Some((name, choices)), given) if !given.is_some_and(|given| choices.contains(&given)) => {
            let given = given.map_or_else(String::new, |given| format!("'{given}' is no {name}; "));
            Err(refused(&format!(
                "{given}the {report} Report takes its {name}, one of: {}",
                choices.join(", ")
            )))
        }
        (None, Some(given)) => Err(refused(&format!(
            "the {report} Report takes no parameter, and was given '{given}'"
        ))),
        _ => Ok(probe),
    }
}

/// The Report named `report` opened: its Quest ([`probe_of`]) probes
/// `installation` for the choice `parameter`, and answers its first two
/// levels and, for the row named `row`, its third; with what a row is
/// called and how long the probe took.
///
/// # Errors
/// As [`probe_of`] refuses; and where `row` is none of its rows, in one
/// sentence opening REFUSED.
pub fn opened(
    reports: &Reports,
    report: &str,
    parameter: Option<&str>,
    row: Option<&str>,
    installation: &dyn Installation,
) -> Result<(String, Duration), String> {
    let probe = probe_of(reports, report, parameter)?;
    let started = Instant::now();
    let found = probe
        .open(installation, parameter, row)
        .map_err(|said| refused(&said))?;
    let took = started.elapsed();
    let mut answer = json!({
        "report": report,
        "node": installation.configuration().service.node_name,
        "row": probe.row(),
        "took_ms": milliseconds(took),
    });
    if let Some(parameter) = parameter {
        answer["parameter"] = json!(parameter);
    }
    if let (Value::Object(answer), Value::Object(found)) = (&mut answer, found) {
        answer.extend(found);
    }
    Ok((answer.to_string(), took))
}

/// `took` in whole milliseconds, as the answer and the audit say it.
#[must_use]
pub fn milliseconds(took: Duration) -> u64 {
    u64::try_from(took.as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests;
