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

No surface shows it yet: the C ABI, the command line, PowerShell and the GUI
follow, as do the other regulations.

`doc/architecture/observability-model.md` section 8 governs it, and which
reports and Quests exist and how each is derived is `doc/reports.md` beside
this file. Each format is a technology under this repository;
`architecture.xmip` names them.
