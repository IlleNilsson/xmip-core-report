# The reports

Moved here from the estate root on 2026-09-12 (ADR-0020 clause 3: the document lives where its subject lives).


`xmip-core-report` answers *what happened over a period*. Four reports, and the
split between mandatory and optional is the point:

**Mandatory:**

| Report | Answers |
| --- | --- |
| Firewall and Operations | every TCP and UDP port, every UNC path a file-based Location touches, and every protocol exposed — inbound and outbound |
| Identity and Isolation Compliance | every identity context, its class, where it runs, and every isolation rule evaluated with its result |

**Optional:** Performance and Capacity, and Artifact End-to-End Drill-Down.

The two mandatory reports are mandatory because of who needs them and when. A
security review board asks for the firewall report before Xmip is permitted onto
a network, and an auditor asks for the compliance report after something has
gone wrong. Neither population knows to ask for a report by name, and a report
that must be requested before it exists is a report nobody has.

**Both are derived, never authored.** The firewall report is computable from the
Receive and Send Locations plus their transport configuration; the compliance
report is computable from identity contexts and the ADR-0022 isolation rules.
A hand-written network document describes what someone believed the estate did
when they wrote it, which is the failure mode this replaces.

Recovered from the `_origins` design export, 2026-08-26.

