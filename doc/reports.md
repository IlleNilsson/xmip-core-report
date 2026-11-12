# The Reports

Moved here from the estate root on 2026-09-12 (ADR-0020 clause 3: the document lives where its subject lives).

`xmip-core-report` answers *what happened over a period*, and *what an
installation shows* when a Quest probes it.

A Report presents existing data; a Quest probes for data to present in a
Report (the owner, 2026-11-07; `doc/terminology.md`, *Report and Quest*).
Every Report is in one list, on every surface, and each says which of the
two it is: a small icon where a surface draws one, its name where a surface
shows text (the owner, 2026-11-10: *I just want the user to know that a
report may go quicker than a report based on quests*; 2026-11-12: *Report
icon and a Probing report icon, not the text*; ADR-0072, amendments
2026-11-10 and 2026-11-12):

- **Report** — it presents data that already exists: what Xmip keeps, read
  as it is.
- **Probing report** — it presents what a Quest probes first. Its entry shows
  how long its last probe took, as the time to expect, or that it has not
  been probed yet.

Which a Report is follows from what it holds, never from its name. An
entry not built yet is listed and says *not yet available*. The list is
`Reports` in `src/catalog.rs`; a module adds a Report or a Quest to it.

## The list

| Report | Kind | Covers | Available |
| --- | --- | --- | --- |
| Audit | Report | What every Xmip program recorded, with each record's severity (information, warning or error), action and phase, over a time range. | Yes, presented by the audit's reader (`xmip_audit_read_v1`) |
| Dead Messages | Report | The Messages no Subscription matched, kept in each node's Dead Message Queue. | Yes (`xmip_dead_messages_v1`) |
| Throughput and latency | Report | Each integration's deliveries and the time from a document's drop to its arrival, and each send's steps, as the nodes measured them. | Not yet: the nodes measure it; no reader presents it as a Report |
| Capacity | Report | Each Storage node's size, each queue's depth, and how fast each grows. | Not yet |
| Expiring certificates and secrets | Report | The certificates and secrets each node holds, and when each expires. | Not yet |
| Parties | Report | The Parties each node recognizes, with the identities each is recognized by and presented with. | Not yet: no reader presents them; the Topology draws them |
| Regulation | Probing report | For each requirement of a regulation, the evidence or the gap a node's configuration shows, and the actions that close a gap. | Yes, below |
| Firewall | Probing report | Every TCP and UDP port, every path a file-based Location touches and every protocol a node opens or reaches, inbound and outbound. | Not yet |
| Security | Probing report | Every identity context, its class and where it runs, and every isolation rule evaluated on it with its result. | Not yet |
| Resilience | Probing report | Every single point of failure, each standby's lag behind its primary, and what a failover would lose. | Not yet |
| Dependency | Probing report | How each integration is connected within Xmip, and every gap in it with the action that closes it. | Not yet: decided (ADR-0072, amendment 2026-11-11) |

What an operator or an auditor asks is in the list, never a second copy of
a view: the Locations and the Subscriptions are their own views, and were
taken out of the list on 2026-11-12 (the owner, the same day: *lets reflect
over what is actually needed. Compile Time we can drop*; ADR-0072,
amendment 2026-11-12).

Two are mandatory: the Firewall Report and the Security Report, whose
identity and isolation ADR-0022 clause 5 requires. A security review board asks for the first
before Xmip is permitted onto a network, and an auditor for the second
after something has gone wrong; neither knows to ask for a report by name,
and a report that must be requested before it exists is a report nobody
has. Being in the one list on every installation is what makes them
mandatory: nobody asks for them into being.

A quick Report is opened where its reader already presents it — the Audit
view, `xmip audit` and `Get-XmipAudit` for the Audit — never through a
second reader. A Report that takes longer is opened by its Quest probing an
installation: a node's configuration, or a cluster's `cluster.xmip` sliced
for one node. Probing reads and changes nothing, so every role may probe;
each probe is recorded in the audit with how long it took, and the newest
record of each Report is the time its entry shows.

## The Firewall Report

A security review board asks for it before Xmip is permitted onto a
network, and neither the board nor the network's owner knows to ask for a
report by name: a report that must be requested before it exists is a
report nobody has. It is derived, never authored: every TCP and UDP port,
every path a file-based Location touches and every protocol exposed,
inbound and outbound, computable from the Receive and Send Locations and
their transports' configuration. A hand-written network document describes
what someone believed the estate did when they wrote it, which is the
failure mode this replaces. Recovered from the `_origins` design export,
2026-08-26; its Quest is not built yet.

## The Security Report: identity and isolation

An auditor asks for it after something has gone wrong, and an isolation
violation that blocked a node's start appears in it (ADR-0022 clause 5). It
is derived, never authored: every identity context, its class and where it
runs, and every isolation rule of ADR-0022 evaluated on it with its result,
computable from the identity contexts and those rules, so a Quest probes
for it. It presents what the rules found and never says an installation
complies (ADR-0072 clause 8; ADR-0022, amendment 2026-11-10). Recovered
from the `_origins` design export, 2026-08-26; its Quest is not built yet.

## The Regulation Report

It presents what the Regulation Quest probes: the first Quest, and the
seam every later one uses (ADR-0072). One Quest, the regulation its
parameter — GDPR, the patient data law, the Cyber Resilience Act, NIS2, the
AI Act — and a module may add one. For each of the regulation's
requirements the Quest probes for the evidence or the gap, or that it does
not apply, and for the actions that close a gap; the operator applies an
action, the Quest never does. The Report presents evidence and gaps and
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
European Union. ADR-0072 lists the nine with their articles. The runtime's
library opens it through `xmip_report_v1` (`xmip_operate.h` section 25)
over a configuration file it composes the installation from, refusing a
Report that cannot be opened before the file is read and a table whose
storage names none its database is kept on as a node starting is refused;
no transport declares its wire yet, so every Location is a gap (ADR-0072
Open 1). Its answer holds the tally as each state's words and count and
says what a row is called, so no surface keeps either. The
command line opens it as `xmip report Regulation --configuration <path>
--parameter GDPR [--row <requirement>]` and PowerShell as `Get-XmipReport
-Name Regulation -Configuration <path> -Parameter GDPR [-Row
<requirement>]`, `--node` and `-Node` naming the node a `cluster.xmip` is
sliced for; both list every Report as `xmip report` and `Get-XmipReport`.
The GUI's Reports view lists every Report at `/reports` and opens this one
at `/reports?report=Regulation`, then `&parameter=GDPR`, then
`&row=<requirement>`; the desktop probes the `cluster.xmip` it edits, on
the node its `gui.toml` configures or the one `&node=` names, and the web
host, which edits none, says where its Quest probes.
