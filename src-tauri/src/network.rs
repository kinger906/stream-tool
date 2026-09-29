use get_if_addrs::{get_if_addrs, IfAddr};
use std::net::IpAddr;

pub fn list_lan_ips() -> Vec<String> {
    let mut ips = Vec::new();
    if let Ok(ifaces) = get_if_addrs() {
        for iface in ifaces {
            if iface.is_loopback() {
                continue;
            }
            let ip = match iface.addr {
                IfAddr::V4(v4) => IpAddr::V4(v4.ip),
                IfAddr::V6(v6) => IpAddr::V6(v6.ip),
            };
            if ip.is_loopback() {
                continue;
            }
            let s = ip.to_string();
            if !ips.contains(&s) {
                ips.push(s);
            }
        }
    }
    if ips.is_empty() {
        ips.push(
            local_ip_address::local_ip()
                .map(|ip| ip.to_string())
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
        );
    }
    ips
}

pub fn resolve_lan_ip(selected: &Option<String>) -> String {
    if let Some(ip) = selected {
        if !ip.is_empty() {
            return ip.clone();
        }
    }
    list_lan_ips()
        .into_iter()
        .next()
        .unwrap_or_else(|| "127.0.0.1".to_string())
}
