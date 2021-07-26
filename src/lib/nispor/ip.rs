use crate::{InterfaceIpAddr, InterfaceIpv4, InterfaceIpv6};

pub(crate) fn np_ipv4_to_nmstate(
    np_ip: &Option<nispor::Ipv4Info>,
) -> Option<InterfaceIpv4> {
    if let Some(np_ip) = np_ip {
        let mut ip = InterfaceIpv4::default();
        if np_ip.addresses.len() > 0 {
            ip.enabled = true;
        }
        for np_addr in &np_ip.addresses {
            if np_addr.valid_lft != "forever" {
                ip.dhcp = true;
            }
            ip.addresses.push(InterfaceIpAddr {
                ip: np_addr.address.clone(),
                prefix_length: np_addr.prefix_len as u32,
            });
        }
        Some(ip)
    } else {
        None
    }
}

pub(crate) fn np_ipv6_to_nmstate(
    np_ip: &Option<nispor::Ipv6Info>,
) -> Option<InterfaceIpv6> {
    if let Some(np_ip) = np_ip {
        let mut ip = InterfaceIpv6::default();
        if np_ip.addresses.len() > 0 {
            ip.enabled = true;
        }
        for np_addr in &np_ip.addresses {
            if np_addr.valid_lft != "forever" {
                ip.autoconf = true;
            }
            ip.addresses.push(InterfaceIpAddr {
                ip: np_addr.address.clone(),
                prefix_length: np_addr.prefix_len as u32,
            });
        }
        Some(ip)
    } else {
        None
    }
}
