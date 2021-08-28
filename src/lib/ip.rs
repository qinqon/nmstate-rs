use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct InterfaceIpv4 {
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip)]
    pub prop_list: Vec<&'static str>,
    #[serde(default)]
    pub dhcp: bool,
    #[serde(rename = "address", default)]
    pub addresses: Vec<InterfaceIpAddr>,
}

impl Serialize for InterfaceIpv4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serial_struct = serializer.serialize_struct(
            "ipv4",
            if self.enabled {
                1
            } else {
                3 // fields count
            },
        )?;
        serial_struct.serialize_field("enabled", &self.enabled)?;
        if self.enabled {
            serial_struct.serialize_field("dhcp", &self.dhcp)?;
            serial_struct.serialize_field("addresses", &self.addresses)?;
        }
        serial_struct.end()
    }
}

impl InterfaceIpv4 {
    pub(crate) fn update(&mut self, other: &Self) {
        if other.prop_list.contains(&"enabled") {
            self.enabled = other.enabled;
        }
        if other.prop_list.contains(&"dhcp") {
            self.dhcp = other.dhcp;
        }
        if other.prop_list.contains(&"addresses") {
            self.addresses = other.addresses.clone();
        }
    }

    pub(crate) fn pre_verify_cleanup(&mut self) {}
}

#[derive(Debug, Clone, PartialEq, Deserialize, Default)]
pub struct InterfaceIpv6 {
    #[serde(default)]
    pub enabled: bool,
    #[serde(skip)]
    pub prop_list: Vec<&'static str>,
    #[serde(default)]
    pub dhcp: bool,
    #[serde(default)]
    pub autoconf: bool,
    #[serde(rename = "address", default)]
    pub addresses: Vec<InterfaceIpAddr>,
}

impl Serialize for InterfaceIpv6 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serial_struct = serializer.serialize_struct(
            "ipv6",
            if self.enabled {
                1
            } else {
                4 // fields count
            },
        )?;
        serial_struct.serialize_field("enabled", &self.enabled)?;
        if self.enabled {
            serial_struct.serialize_field("dhcp", &self.dhcp)?;
            serial_struct.serialize_field("autoconf", &self.autoconf)?;
            serial_struct.serialize_field("addresses", &self.addresses)?;
        }
        serial_struct.end()
    }
}

impl InterfaceIpv6 {
    pub(crate) fn update(&mut self, other: &Self) {
        if other.prop_list.contains(&"enabled") {
            self.enabled = other.enabled;
        }
        if other.prop_list.contains(&"dhcp") {
            self.dhcp = other.dhcp;
        }
        if other.prop_list.contains(&"autoconf") {
            self.dhcp = other.dhcp;
        }
        if other.prop_list.contains(&"addresses") {
            self.addresses = other.addresses.clone();
        }
    }

    // Remove link-local address
    pub(crate) fn pre_verify_cleanup(&mut self) {
        self.addresses.retain(|addr| {
            !is_ipv6_unicast_link_local(&addr.ip, addr.prefix_length)
        });
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct InterfaceIpAddr {
    pub ip: String,
    pub prefix_length: u32,
}

fn is_ipv6_addr(addr: &str) -> bool {
    addr.contains(':')
}

// TODO: Rust offical has std::net::Ipv6Addr::is_unicast_link_local() in
// experimental.
fn is_ipv6_unicast_link_local(ip: &str, prefix: u32) -> bool {
    // The unicast link local address range is fe80::/10.
    is_ipv6_addr(ip)
        && ip.len() >= 3
        && ["fe8", "fe9", "fea", "feb"].contains(&&ip[..3])
        && prefix >= 10
}
