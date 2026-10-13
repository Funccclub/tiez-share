//! LAN subnets follow active interfaces; only explicitly trusted proxies convey client IPs.
use axum::{extract::{ConnectInfo, Request, State}, middleware::Next,
    http::{HeaderMap, StatusCode}, response::{IntoResponse, Response}};
use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::{Arc, RwLock}, time::{Duration, Instant}};
use crate::models::AppState;

#[derive(Clone, Debug, PartialEq)]
pub struct Network { address: IpAddr, prefix: u32 }
fn normalize(ip: IpAddr) -> IpAddr {
    match ip { IpAddr::V6(v) => v.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip), _ => ip }
}
fn lan_address(ip: IpAddr) -> bool {
    match normalize(ip) {
        IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local(),
        IpAddr::V6(v) => v.is_loopback() || (v.segments()[0] & 0xfe00 == 0xfc00) ||
            (v.segments()[0] & 0xffc0 == 0xfe80) || (v.segments()[0] & 0xe000 == 0x2000),
    }
}
impl Network {
    pub fn parse(text: &str) -> Result<Self, String> {
        let (ip, prefix) = text.trim().split_once('/').ok_or("Expected IP/prefix CIDR")?;
        let address: IpAddr = ip.parse().map_err(|_| "Invalid network address")?;
        let prefix: u32 = prefix.parse().map_err(|_| "Invalid prefix")?;
        let max = if address.is_ipv4() { 32 } else { 128 };
        if prefix == 0 || prefix > max || !lan_address(address) ||
            matches!(address, IpAddr::V6(v) if v.to_ipv4_mapped().is_some() || ((v.segments()[0] & 0xe000 == 0x2000) && prefix < 64)) {
            return Err("Expected a LAN CIDR; global IPv6 requires /64 or narrower".into());
        }
        Ok(Self { address, prefix })
    }
    pub fn contains(&self, ip: IpAddr) -> bool {
        let ip = normalize(ip);
        if !lan_address(ip) { return false; }
        match (self.address, ip) {
            (IpAddr::V4(a), IpAddr::V4(b)) => u32::from(a) >> (32-self.prefix) == u32::from(b) >> (32-self.prefix),
            (IpAddr::V6(a), IpAddr::V6(b)) => u128::from(a) >> (128-self.prefix) == u128::from(b) >> (128-self.prefix),
            _ => false,
        }
    }
}
fn env_list(key: &str) -> Result<Vec<Network>, String> {
    let value = std::env::var(key).unwrap_or_default();
    if value.trim().is_empty() { return Ok(Vec::new()); }
    value.split(',').map(Network::parse).collect()
}
fn choose_interface(routes: &str, interfaces: &[String]) -> Option<String> {
    if interfaces.iter().any(|i| i == "br-lan") { return Some("br-lan".into()); }
    let default = routes.lines().skip(1).filter_map(|line| {
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() < 8 || f[1] != "00000000" { return None; }
        let flags = u32::from_str_radix(f[3], 16).ok()?;
        if flags & 1 == 0 || !interfaces.iter().any(|i| i == f[0]) { return None; }
        Some((f[6].parse::<u64>().ok()?, f[0].to_string()))
    }).min_by_key(|(metric, _)| *metric).map(|(_, iface)| iface);
    default.or_else(|| if interfaces.len() == 1 { Some(interfaces[0].clone()) } else { None })
}
fn eligible_name(name: &str) -> bool {
    name != "lo" && name != "docker0" && !["br-", "veth", "tun", "tap", "wg", "tailscale", "zt"].iter().any(|prefix| name.starts_with(prefix))
}
fn discover_from(routes: &str, inet6: &str, active: &[String], preferred: Option<&str>) -> Vec<Network> {
    let mut candidates = Vec::new();
    for line in routes.lines().skip(1) {
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() < 8 || f[2] != "00000000" || !active.iter().any(|i| i == f[0]) { continue; }
        if let (Ok(dest), Ok(mask), Ok(flags)) = (u32::from_str_radix(f[1],16),
            u32::from_str_radix(f[7],16), u32::from_str_radix(f[3],16)) {
            let dest = Ipv4Addr::from(dest.to_le_bytes());
            let mask = u32::from_be_bytes(mask.to_le_bytes());
            let prefix = mask.leading_ones();
            if flags & 1 != 0 && prefix > 0 && mask == u32::MAX << (32-prefix) {
                if let Ok(net) = Network::parse(&format!("{dest}/{prefix}")) { candidates.push((f[0].to_string(), net)); }
            }
        }
    }
    for line in inet6.lines() {
        let f: Vec<_> = line.split_whitespace().collect();
        if f.len() < 6 || !active.iter().any(|i| i == f[5]) { continue; }
        if let (Ok(ip), Ok(prefix), Ok(flags)) = (u128::from_str_radix(f[0],16), u32::from_str_radix(f[2],16), u32::from_str_radix(f[4],16)) {
            // Ignore tentative, DAD-failed and deprecated addresses (old delegated prefixes).
            if flags & (0x40 | 0x08 | 0x20) != 0 { continue; }
            if let Ok(net) = Network::parse(&format!("{}/{prefix}", std::net::Ipv6Addr::from(ip))) { candidates.push((f[5].to_string(), net)); }
        }
    }
    let mut interfaces: Vec<_> = candidates.iter().map(|(iface, _)| iface.clone()).collect();
    interfaces.sort(); interfaces.dedup();
    let chosen = match preferred {
        Some(iface) => interfaces.iter().find(|i| i.as_str() == iface).cloned(),
        None => choose_interface(routes, &interfaces),
    };
    candidates.into_iter().filter(|(iface,_)| Some(iface) == chosen.as_ref()).map(|(_,net)| net).collect()
}
fn discover(preferred: Option<&str>) -> Vec<Network> {
    let active: Vec<_> = std::fs::read_dir("/sys/class/net").into_iter().flatten().flatten().filter_map(|entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        if !eligible_name(&name) && preferred != Some(name.as_str()) { return None; }
        let flags = std::fs::read_to_string(entry.path().join("flags")).ok()?;
        let flags = u32::from_str_radix(flags.trim().trim_start_matches("0x"), 16).ok()?;
        let carrier = std::fs::read_to_string(entry.path().join("carrier")).ok()?;
        if flags & 1 == 0 || carrier.trim() != "1" { return None; }
        Some(name)
    }).collect();
    discover_from(&std::fs::read_to_string("/proc/net/route").unwrap_or_default(),
        &std::fs::read_to_string("/proc/net/if_inet6").unwrap_or_default(), &active, preferred)
}
#[derive(Debug)]
pub struct Policy {
    fixed: Vec<Network>,
    interface: Option<String>,
    trusted_proxies: Vec<Network>,
    cache: RwLock<(Instant, Vec<Network>)>,
}
impl Policy {
    pub fn load() -> Result<Self, String> {
        let fixed = env_list("TIEZ_LAN_CIDRS")?;
        let trusted_proxies = env_list("TIEZ_TRUSTED_PROXIES")?;
        if trusted_proxies.iter().any(|n| n.prefix != if n.address.is_ipv4() { 32 } else { 128 }) {
            return Err("TIEZ_TRUSTED_PROXIES requires individual /32 or /128 hosts".into());
        }
        let interface = std::env::var("TIEZ_LAN_INTERFACE").ok().filter(|s| !s.trim().is_empty()).map(|s| s.trim().to_string());
        let networks = if fixed.is_empty() { discover(interface.as_deref()) } else { fixed.clone() };
        Ok(Self { fixed, interface, trusted_proxies, cache: RwLock::new((Instant::now(), networks)) })
    }
    fn allows(&self, ip: IpAddr) -> bool {
        if self.fixed.is_empty() && self.cache.read().unwrap().0.elapsed() >= Duration::from_secs(5) {
            let mut cache = self.cache.write().unwrap();
            if cache.0.elapsed() >= Duration::from_secs(5) {
                let networks = discover(self.interface.as_deref());
                if networks != cache.1 { println!("[tiez-share] refreshed LAN networks: {networks:?}"); }
                *cache = (Instant::now(), networks);
            }
        }
        allowed(ip, &self.cache.read().unwrap().1)
    }
    fn client(&self, peer: IpAddr, headers: &HeaderMap) -> Option<IpAddr> {
        client_ip(peer, headers, &self.trusted_proxies)
    }
}
pub fn allowed(ip: IpAddr, networks: &[Network]) -> bool {
    normalize(ip).is_loopback() || networks.iter().any(|n| n.contains(ip))
}
fn client_ip(peer: IpAddr, headers: &HeaderMap, proxies: &[Network]) -> Option<IpAddr> {
    let forwarded = ["forwarded", "x-forwarded-for", "x-real-ip"].iter().any(|h| headers.contains_key(*h));
    let trusted = proxies.iter().any(|p| p.contains(peer));
    if !trusted { return if forwarded { None } else { Some(peer) }; }
    // Last XFF hop is appended/overwritten by the trusted proxy. Never trust
    // attacker-controlled prepended entries. A proxy must supply client identity.
    if headers.contains_key("forwarded") || headers.get_all("x-forwarded-for").iter().count() != 1 { return None; }
    let value = headers.get("x-forwarded-for")?.to_str().ok()?;
    let ip: IpAddr = value.rsplit(',').next()?.trim().parse().ok()?;
    if normalize(ip).is_loopback() { return None; }
    Some(ip)
}
pub async fn lan_only(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|v| v.0.ip());
    let cross_origin = request.headers().get("origin").is_some_and(|origin| {
        let origin = origin.to_str().unwrap_or("");
        let authority = origin.strip_prefix("http://").or_else(|| origin.strip_prefix("https://"));
        let host = request.headers().get("host").and_then(|v| v.to_str().ok());
        authority.is_none() || authority != host
    });
    // Local container health probes do not carry proxy headers. This exception
    // grants only health status, never files, messages or WebSocket access.
    let local_probe = request.uri().path() == "/healthz" && peer.is_some_and(|ip| normalize(ip).is_loopback()) &&
        !["forwarded", "x-forwarded-for", "x-real-ip"].iter().any(|h| request.headers().contains_key(*h));
    let client = if local_probe { peer } else { peer.and_then(|ip| state.cfg.lan_networks.client(ip, request.headers())) };
    if cross_origin || !client.is_some_and(|ip| state.cfg.lan_networks.allows(ip)) {
        return (StatusCode::FORBIDDEN, "LAN access only; client must be on the server's LAN").into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restrict_to_actual_subnet_including_global_ipv6() {
        let nets = vec![Network::parse("192.168.8.0/24").unwrap(), Network::parse("fd12:3456::/64").unwrap(), Network::parse("2408:8474:5900:6aa5::/64").unwrap()];
        for ip in ["192.168.8.23", "::ffff:192.168.8.23", "127.0.0.1", "::1", "fd12:3456::2", "2408:8474:5900:6aa5:abcd::1"] {
            assert!(allowed(ip.parse().unwrap(), &nets), "{ip}");
        }
        for ip in ["192.168.9.23", "10.0.0.2", "8.8.8.8", "2001:4860::1", "fd12:3457::2", "2408:8474:5900:6aa6::1", "ff02::1"] {
            assert!(!allowed(ip.parse().unwrap(), &nets), "{ip}");
        }
        for cidr in ["0.0.0.0/0", "192.168.0.0/33", "2408::/16", "::ffff:192.168.8.1/128"] { assert!(Network::parse(cidr).is_err()); }
    }
    #[test]
    fn active_interface_selection_and_renumbering() {
        let routes = "Iface Destination Gateway Flags RefCnt Use Metric Mask\neth0 00000000 0108A8C0 0003 0 0 100 00000000\neth0 0008A8C0 00000000 0001 0 0 0 00FFFFFF\nbr-lan 0064A8C0 00000000 0001 0 0 0 00FFFFFF\ndocker0 000011AC 00000000 0001 0 0 0 0000FFFF\n";
        let active = vec!["eth0".into()];
        let inet6 = "2408847459006aa50000000000000001 02 40 00 80 eth0\n2408847459006aa40000000000000001 02 40 00 a0 eth0\nfd520000000000000000000000000001 03 40 00 80 br-lan\n";
        let nets = discover_from(routes, inet6, &active, None);
        assert!(allowed("192.168.8.20".parse().unwrap(), &nets));
        assert!(allowed("2408:8474:5900:6aa5::20".parse().unwrap(), &nets));
        for ip in ["2408:8474:5900:6aa4::20", "192.168.100.20", "172.17.0.2"] { assert!(!allowed(ip.parse().unwrap(), &nets)); }
        let changed = discover_from(&routes.replace("0008A8C0", "0009A8C0"), &inet6.replace("6aa5", "6aa6"), &active, None);
        assert!(allowed("192.168.9.20".parse().unwrap(), &changed));
        assert!(!allowed("192.168.8.20".parse().unwrap(), &changed));
        assert!(allowed("2408:8474:5900:6aa6::20".parse().unwrap(), &changed));
        assert!(discover_from(routes, inet6, &active, Some("missing0")).is_empty());
        assert_eq!(choose_interface("", &["eth0".into(), "eth1".into()]), None);
        assert!(!eligible_name("br-abcdef"));
        assert!(eligible_name("br-lan"));
    }
    #[test]
    fn proxy_spoofing_cannot_grant_access() {
        let proxy = vec![Network::parse("127.0.0.1/32").unwrap()];
        let peer = "127.0.0.1".parse().unwrap();
        let mut h = HeaderMap::new();
        assert_eq!(client_ip(peer, &h, &proxy), None);
        h.insert("x-forwarded-for", "192.168.8.2, 2001:4860::1".parse().unwrap());
        assert_eq!(client_ip(peer, &h, &proxy), Some("2001:4860::1".parse().unwrap()));
        assert!(!allowed(client_ip(peer, &h, &proxy).unwrap(), &[Network::parse("192.168.8.0/24").unwrap()]));
        assert_eq!(client_ip("192.168.8.2".parse().unwrap(), &h, &proxy), None);
        h.insert("x-forwarded-for", "2408:8474:5900:6aa5::2, 2408:8474:5900:6aa5::2".parse().unwrap());
        assert_eq!(client_ip(peer, &h, &proxy), Some("2408:8474:5900:6aa5::2".parse().unwrap()));
        h.insert("x-forwarded-for", "192.168.8.2, malformed".parse().unwrap());
        assert_eq!(client_ip(peer, &h, &proxy), None);
        h.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());
        assert_eq!(client_ip(peer, &h, &proxy), None);
    }
    #[test]
    fn refresh_removes_stale_networks_when_interface_disappears() {
        let policy = Policy {
            fixed: vec![], interface: Some("tiez-nonexistent-test-interface".into()), trusted_proxies: vec![],
            cache: RwLock::new((Instant::now() - Duration::from_secs(6), vec![Network::parse("192.168.8.0/24").unwrap()])),
        };
        assert!(!policy.allows("192.168.8.20".parse().unwrap()));
        assert!(policy.cache.read().unwrap().1.is_empty());
    }
}
