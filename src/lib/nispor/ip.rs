use crate::{InterfaceIpAddr, InterfaceIpv4, InterfaceIpv6};

pub(crate) fn np_ipv4_to_nmstate(
    np_iface: &nispor::Iface,
) -> Option<InterfaceIpv4> {
    if let Some(np_ip) = &np_iface.ipv4 {
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
        // IP might just disabled
        if np_iface.controller == None {
            Some(InterfaceIpv4 {
                enabled: false,
                ..Default::default()
            })
        } else {
            None
        }
    }
}

pub(crate) fn np_ipv6_to_nmstate(
    np_iface: &nispor::Iface,
) -> Option<InterfaceIpv6> {
    if let Some(np_ip) = &np_iface.ipv6 {
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
        // IP might just disabled
        if np_iface.controller == None {
            Some(InterfaceIpv6 {
                enabled: false,
                ..Default::default()
            })
        } else {
            None
        }
    }
}
