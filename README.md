# xmip-core-report

Reporting: what happened over a period. A `ReportRecord` is a typed record
tied to a Journey and a Message with its fields, and a `ReportDataSink`
receives them; the reports themselves are derived from configuration and
records, never authored by hand.

Reporting is the historical counterpart of observation, not a second audit:
observation answers what is happening, audit is the durable record, and a
report is what a security review board or an auditor reads over a period.

A Report presents existing data; a Quest probes for data to present in a
Report. The Quest seam is here (ADR-0072): a `Quest` probes an
`Installation` handed to it — a node's configuration as
`xmip-core-configure` reads it, and what a Location's technology declares of
its wire and how each data domain is kept at rest — and changes nothing.
The first Quest is the Regulation Quest, `RegulationQuest`: one Quest, the
regulation a parameter from an open set, `Regulations`, to which a module
adds its own `Regulation`. Each requirement cites its article and is found
as evidence, a gap or not applicable, with the Locations, Streams and
settings behind it and the actions that close a gap, which the operator
applies. It never judges whether an installation meets a regulation. GDPR,
`gdpr.rs`, is the first regulation.

```rust
let regulations = Regulations::standard();
let probed = RegulationQuest::of(&regulations, "GDPR")?.probe(&installation);
let tally = probed.tally();                      // the regulation
for row in probed.rows() { /* name, citation, state */ }  // its requirements
let one = probed.requirement("Encryption at rest");       // one requirement
```

**Every Report in one list** (ADR-0072, amendment 2026-11-10). `Reports`
(`catalog.rs`) is the list, Xmip's own in `Reports::standard()` and a
module's added with `Reports::with`; a `Report` has a name, what it covers,
and its `Data`. Its kind follows from that data and nothing else:
`Data::Existing` presents data that already exists — `Kind::ExistingData`,
shown *Report* — named by the `xmip_operate.h` entrypoint of the reader that
presents it, and `Data::Quest` presents what a Quest probes first —
`Kind::Quest`, shown *Probing report* — through a `Probe`, a Quest as a list
holds it: its parameter and choices, what a row of its second level is
called, its levels, and `open`. Either holds `None` where it is not built
yet, and the entry is listed and says *not yet available*. The Regulation
Quest is listed as the Regulation Report, `RegulationReport`, its parameter
the regulation and its row a requirement; the JSON it answers, its tally
each state's words and count, is shaped in `regulation.rs` beside its
Quest, the example a module's Quest follows. `answer.rs` answers the list,
or one entry of it by name, and one Report opened as JSON, as
`xmip_operate.h` section 25 says: each kind's words, and for each entry
what to expect — a Quest's last probe, as `LastProbes` says it, handed in by
whoever keeps the record, or that it could not be read, every Report listed
all the same; an opened Report says what a row is called.
`answer::probe_of` is every refusal of an open that needs no installation —
a name that is none, in the one sentence the list refuses it in too, a
Report not yet available or quick, a parameter it does not take — so a
caller refuses before it reads anything. Six Reports are quick (Audit,
Dead Messages, Locations, Subscriptions; Parties and Expiring certificates
and secrets not yet available) and ten take longer (Regulation; Firewall,
Identity and isolation — ADR-0022 clause 5's, mandatory with the
Firewall's — Security, Sanity, Performance, Redundancy, Fail Over, Size and
Compile Time not yet available).

The runtime's library (`xmip-core-runtime`, `src/report.rs`) serves the
list through `xmip_report_v1`, composing what a Quest probes from a node's
configuration and recording each probe in the audit, whose last record of
each is the time to expect. Every .NET surface reads it through
`ReportCatalog` in `Xmip.Surface` (`xmip-core-abi`): the command line as
`xmip report`, PowerShell as `Get-XmipReport`, and the GUI, the web's and
the desktop's alike, as the Reports view (`/reports`), each kind shown by a
plain icon beside its name.

`doc/architecture/observability-model.md` section 8 governs it, and which
Reports and Quests exist and how each is derived is `doc/reports.md` beside
this file. Each format is a technology under this repository;
`architecture.toml` names them.
