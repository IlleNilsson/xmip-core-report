# xmip-core-report

Reporting: what happened over a period. A `ReportRecord` is a typed record
tied to a Journey and a Message with its fields, and a `ReportDataSink`
receives them; the reports themselves are derived from configuration and
records, never authored by hand.

Reporting is the historical counterpart of observation, not a second audit:
observation answers what is happening, audit is the durable record, and a
report is what a security review board or an auditor reads over a period.

`doc/architecture/observability-model.md` section 8 governs it, and which
reports exist and how each is derived is `doc/reports.md` beside this file.
Each format is a technology under this repository; `architecture.toml` names
them.
