//! An installation as a test composes it, placed once for every test of
//! this crate: a configuration read from TOML, each Location's wire by its
//! name, and each data domain kept as said.

use std::collections::BTreeMap;

use configure::{ConfiguredLocation, XmipConfigurationDocument, parse_toml};
use xcore::settings::Applies;

use crate::quest::{AtRest, Installation, Quest, Wire};
use crate::regulation::{Probed, RegulationQuest, Regulations};

/// The `[service]` every test's configuration opens with: node `d`.
pub const HEAD: &str = "[service]\nname = \"n\"\ncluster_name = \"c\"\nnode_name = \"d\"\n";

/// A Receive Location `in` and a Send Location `out`.
pub const LOCATIONS: &str = "[[receive_locations]]\nname = \"in\"\naddress = \"https://h/in\"\n\
                             [[send_locations]]\nname = \"out\"\naddress = \"sftp://h/out\"\n";

/// The installation: every wire undeclared and every domain sealed unless
/// said otherwise.
pub struct Composed {
    configuration: XmipConfigurationDocument,
    wires: BTreeMap<&'static str, Wire>,
    at_rest: BTreeMap<&'static str, AtRest>,
}

impl Composed {
    /// The node `d` with `tables` after its `[service]`.
    pub fn of(tables: &str) -> Self {
        Self {
            configuration: parse_toml(&format!("{HEAD}{tables}")).expect("parses"),
            wires: BTreeMap::new(),
            at_rest: BTreeMap::new(),
        }
    }

    /// The Location `name`'s wire declared `wire`.
    pub fn wire(mut self, name: &'static str, wire: Wire) -> Self {
        self.wires.insert(name, wire);
        self
    }

    /// The data domain `domain` kept `at_rest`.
    pub fn kept(mut self, domain: &'static str, at_rest: AtRest) -> Self {
        self.at_rest.insert(domain, at_rest);
        self
    }

    /// What the GDPR Quest finds of it.
    pub fn probed(&self) -> Probed {
        let regulations = Regulations::standard();
        RegulationQuest::of(&regulations, "GDPR")
            .expect("held")
            .probe(self)
    }
}

impl Installation for Composed {
    fn configuration(&self) -> &XmipConfigurationDocument {
        &self.configuration
    }

    fn wire(&self, location: &ConfiguredLocation, _: Applies) -> Wire {
        self.wires
            .get(location.name.as_str())
            .copied()
            .unwrap_or(Wire::Undeclared)
    }

    fn at_rest(&self, domain: &str) -> AtRest {
        self.at_rest.get(domain).copied().unwrap_or(AtRest::Sealed)
    }
}
