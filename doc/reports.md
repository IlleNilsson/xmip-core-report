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

## Reports and Quests

A Report presents existing data; a Quest probes for data to present in a
Report (the owner, 2026-11-07; `doc/terminology.md`, *Report and Quest*).
The Quests the owner named 2026-11-05 — Security, Sanity, Performance,
Redundancy, Fail Over, Size, Compile Time and Firewall — are not designed
yet: each one's data and drill-down are asked of the owner before it is
built.

### The Regulation Quest

The first Quest, and the seam every later one uses (ADR-0072). One Quest,
the regulation its parameter — GDPR, the patient data law, the Cyber
Resilience Act, NIS2, the AI Act — and a module may add one. For each of the
regulation's requirements it probes, shows the evidence or the gap or that
it does not apply, and offers the actions that close a gap; the operator
applies an action, the Quest never does. It presents evidence and gaps and
never a verdict: whether an installation meets a regulation is the
customer's legal judgment (the owner, 2026-11-08).

It reads at three levels (the owner, 2026-11-08):

1. **The regulation**, with a tally of evidence, gaps and not applicable.
2. **Its requirements**, one row each: the requirement, the article it
   cites and its state.
3. **One requirement**, with the Locations, Streams and settings behind it
   and the actions that close it.

**Derived from configuration.** GDPR, the first regulation, probes what a
node's configuration says today — encryption at rest from the data domains
and `[store]`, encryption on each Receive and Send Location's wire as its
technology declares it, and the audit — and shows each requirement Xmip
cannot support yet as a gap whose action is *not yet available in Xmip*:
how long data is kept (a node reads no retention window), erasure and
finding one data subject's data, masking personal data in logs and traces,
declaring the flows that carry personal data, and transfers outside the
European Union. ADR-0072 lists the nine with their articles. No surface
shows it yet; the ABI, the command line, PowerShell and the GUI follow.
