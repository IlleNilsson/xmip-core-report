//! GDPR, Regulation (EU) 2016/679, Xmip's first regulation (ADR-0072).
//!
//! Each requirement cites its article. Three are probed in what a node's
//! configuration can say today — encryption at rest, encryption on each
//! Location's wire and the audit — and six Xmip cannot support yet, how long
//! data is kept among them: each is a gap whose action says it is not yet
//! available in Xmip. A finding presents evidence and gaps, never a verdict on the
//! regulation: that is the customer's legal judgment.

use configure::ConfiguredLocation;
use xcore::settings::Applies;

use crate::finding::{Action, Behind, Finding};
use crate::quest::{AtRest, Installation, Wire};
use crate::regulation::{Regulation, Requirement};

/// GDPR.
pub struct Gdpr;

const REQUIREMENTS: &[&dyn Requirement] = &[
    &Unavailable {
        name: "Data minimization in logs and traces",
        citation: "Article 5(1)(c) and Article 25(1)",
        says: "Xmip cannot yet mask personal data in what it logs and traces.",
        action: "Masking personal data in logs and traces is not yet available in Xmip.",
    },
    &StorageLimitation,
    &Accountability,
    &Unavailable {
        name: "Right of access",
        citation: "Article 15",
        says: "Xmip cannot yet find one data subject's personal data in what it keeps.",
        action: "Finding one data subject's personal data is not yet available in Xmip.",
    },
    &Unavailable {
        name: "Right to erasure",
        citation: "Article 17",
        says: "Xmip cannot yet erase one data subject's personal data from what it keeps.",
        action: "Erasure per data subject is not yet available in Xmip.",
    },
    &Unavailable {
        name: "Records of processing activities",
        citation: "Article 30",
        says: "Xmip cannot yet declare which flows carry personal data.",
        action: "Declaring the flows that carry personal data is not yet available in Xmip.",
    },
    &EncryptionAtRest,
    &EncryptionInTransit,
    &Unavailable {
        name: "Transfers outside the European Union",
        citation: "Chapter V, Article 44",
        says: "Xmip cannot yet say where personal data is sent outside the European Union.",
        action: "Probing transfers of personal data outside the European Union is not yet \
                 available in Xmip.",
    },
];

impl Regulation for Gdpr {
    fn name(&self) -> &'static str {
        "GDPR"
    }

    fn title(&self) -> &'static str {
        "Regulation (EU) 2016/679, the General Data Protection Regulation"
    }

    fn requirements(&self) -> &[&dyn Requirement] {
        REQUIREMENTS
    }
}

/// A requirement Xmip cannot support yet: always a gap, its action saying
/// so.
struct Unavailable {
    name: &'static str,
    citation: &'static str,
    says: &'static str,
    action: &'static str,
}

impl Requirement for Unavailable {
    fn name(&self) -> &'static str {
        self.name
    }

    fn citation(&self) -> &'static str {
        self.citation
    }

    fn probe(&self, _: &dyn Installation) -> Finding {
        Finding::gap(self.says, Vec::new(), vec![not_yet(self.action)])
    }
}

fn not_yet(says: &str) -> Action {
    Action::NotYetAvailable {
        says: says.to_string(),
    }
}

fn setting(key: String, value: &str) -> Behind {
    Behind::Setting {
        key,
        value: value.to_string(),
    }
}

/// How long data is kept: a node reads no retention window yet.
struct StorageLimitation;

impl Requirement for StorageLimitation {
    fn name(&self) -> &'static str {
        "Storage limitation"
    }

    fn citation(&self) -> &'static str {
        "Article 5(1)(e)"
    }

    fn probe(&self, _: &dyn Installation) -> Finding {
        Finding::gap(
            "A node reads no retention window: retention and archiving are built and not in \
             the node, so its configuration says nothing of how long what Xmip keeps is kept.",
            Vec::new(),
            vec![not_yet(
                "A retention window on a node is not yet available in Xmip.",
            )],
        )
    }
}

/// The audit every Xmip program keeps.
struct Accountability;

impl Requirement for Accountability {
    fn name(&self) -> &'static str {
        "Accountability: the audit"
    }

    fn citation(&self) -> &'static str {
        "Article 5(2)"
    }

    fn probe(&self, installation: &dyn Installation) -> Finding {
        let behind = installation
            .configuration()
            .audit
            .iter()
            .map(|audit| setting("audit.storage".to_string(), &audit.storage))
            .collect();
        Finding::evidence(
            "Every Xmip program records what it does and every failure through xmip-core-audit, \
             each record carrying the digest of the one before it in its writer's chain, so a \
             record deleted, changed or reordered is found (ADR-0062, ADR-0070).",
            behind,
        )
    }
}

/// Encryption of the records Xmip keeps.
struct EncryptionAtRest;

impl Requirement for EncryptionAtRest {
    fn name(&self) -> &'static str {
        "Encryption at rest"
    }

    fn citation(&self) -> &'static str {
        "Article 32(1)(a)"
    }

    fn probe(&self, installation: &dyn Installation) -> Finding {
        let configuration = installation.configuration();
        let mut sealed = Vec::new();
        let mut servers = Vec::new();
        for (domain, _) in configuration.domains() {
            match installation.at_rest(domain) {
                AtRest::Sealed => sealed.push(domain),
                AtRest::Server(word) => servers.push(setting(format!("{domain}.storage"), word)),
                AtRest::StorageNodes => {}
            }
        }
        if !servers.is_empty() {
            return Finding::gap(
                "A database is kept on a database server IT runs: its encryption at rest is \
                 IT's (ADR-0063), and Xmip holds no evidence of it.",
                servers,
                vec![Action::Outside {
                    says: "IT encrypts the database server's data at rest, by the server's own \
                           encryption or encrypted disks, and keeps the evidence."
                        .to_string(),
                }],
            );
        }
        if sealed.is_empty() {
            let nodes: Vec<String> = configuration
                .storage
                .nodes
                .iter()
                .map(ToString::to_string)
                .collect();
            return Finding::not_applicable(
                "This node keeps no records: the Storage nodes it lists keep them, each probed \
                 on its own.",
                vec![setting("storage.nodes".to_string(), &nodes.join(", "))],
            );
        }
        Finding::evidence(
            format!(
                "The {} databases are kept on this node's embedded Storage node, and every record \
                 is sealed with authenticated encryption before its engine sees it, under a \
                 data key the key store wraps (ADR-0063).",
                sealed.join(", ")
            ),
            vec![setting(
                "store.key_store".to_string(),
                configuration.store.key_store(),
            )],
        )
    }
}

/// Encryption on each Location's wire.
struct EncryptionInTransit;

impl Requirement for EncryptionInTransit {
    fn name(&self) -> &'static str {
        "Encryption in transit"
    }

    fn citation(&self) -> &'static str {
        "Article 32(1)(a)"
    }

    fn probe(&self, installation: &dyn Installation) -> Finding {
        let configuration = installation.configuration();
        let locations: Vec<(Applies, &ConfiguredLocation)> = configuration
            .receive_locations
            .iter()
            .map(|location| (Applies::Receive, location))
            .chain(
                configuration
                    .send_locations
                    .iter()
                    .map(|location| (Applies::Send, location)),
            )
            .collect();
        if locations.is_empty() {
            return Finding::not_applicable(
                "The node has no Receive or Send Location.",
                Vec::new(),
            );
        }
        let mut encrypted = Vec::new();
        let mut open = Vec::new();
        let mut actions = Vec::new();
        let mut undeclared = Vec::new();
        for (side, location) in locations {
            let behind = Behind::Location {
                side,
                name: location.name.clone(),
            };
            match installation.wire(location, side) {
                Wire::Encrypted => encrypted.push(behind),
                Wire::Plain => {
                    actions.push(Action::Configure {
                        setting: format!("{}_locations.{}", side.word(), location.name),
                        says: format!(
                            "Enter '{}' with the address or setting its technology encrypts \
                             with.",
                            location.name
                        ),
                    });
                    open.push(behind);
                }
                Wire::Undeclared => {
                    undeclared.push(location.name.as_str());
                    open.push(behind);
                }
            }
        }
        if open.is_empty() {
            return Finding::evidence(
                "Every Location's technology declares its wire encrypted.",
                encrypted,
            );
        }
        if !undeclared.is_empty() {
            actions.push(not_yet(&format!(
                "Probing the wire of {}: its technology declares nothing of it, and that \
                 declaration is not yet available in Xmip.",
                undeclared.join(", ")
            )));
        }
        Finding::gap(
            "A Location's wire is not declared encrypted.",
            open,
            actions,
        )
    }
}

#[cfg(test)]
mod tests;
