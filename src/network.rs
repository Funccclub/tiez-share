//! Direct-peer LAN restriction. Forwarding headers never grant access.
use axum::{extract::{ConnectInfo, Request, State}, middleware::Next,
    http::StatusCode, response::{IntoResponse, Response}};
use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, sync::Arc};
use crate::models::AppState;

#[derive(Clone, Debug)]
pub struct Network { address: IpAddr, prefix: u32 }

fn local(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => v.is_private() || v.is_loopback() || v.is_link_local(),
        IpAddr::V6(v) => v.to_ipv4_mapped().map(|v| local(v.into())).unwrap_or(
            v.is_loopback() || (v.segments()[0] & 0xfe00 == 0xfc00) ||
            (v.segments()[0] & 0xffc0 == 0xfe80)),
    }
}

impl Network {
    pub fn parse(text: &str) -> Result<Self, String> {
        let (ip, prefix) = text.trim().split_once('/').ok_or("Expected CIDR, e.g. 192.168.1.0/24")?;
        let address: IpAddr = ip.parse().map_err(|_| "Invalid network address")?;
        let prefix: u32 = prefix.parse().map_err(|_| "Invalid prefix")?;
        let max = if address.is_ipv4() { 32 } else { 128 };
        if prefix == 0 || prefix > max || !local(address) {
            return Err("LAN CIDR must be a local address with a nonzero valid prefix".into());
        }
        Ok(Self { address, prefix })
    }
    pub fn contains(&self, ip: IpAddr) -> bool {
        let ip = match ip { IpAddr::V6(v) => v.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip), _ => ip };
        if !local(ip) { return false; }
        match (self.address, ip) {
            (IpAddr::V4(a), IpAddr::V4(b)) => u32::from(a) >> (32-self.prefix) == u32::from(b) >> (32-self.prefix),
            (IpAddr::V6(a), IpAddr::V6(b)) => u128::from(a) >> (128-self.prefix) == u128::from(b) >> (128-self.prefix),
            _ => false,
        }
    }
}

pub fn networks() -> Result<Vec<Network>, String> {
    if let Ok(value) = std::env::var("TIEZ_LAN_CIDRS") {
        if !value.trim().is_empty() { return value.split(',').map(Network::parse).collect(); }
    }
    let mut out = Vec::new();
    // Linux routing table: only directly connected networks, never the default route.
    if let Ok(routes) = std::fs::read_to_string("/proc/net/route") {
        for line in routes.lines().skip(1) {
            let f: Vec<_> = line.split_whitespace().collect();
            if f.len() < 8 || f[2] != "00000000" { continue; }
            if let (Ok(dest), Ok(mask), Ok(flags)) = (u32::from_str_radix(f[1],16),
                u32::from_str_radix(f[7],16), u32::from_str_radix(f[3],16)) {
                let dest = Ipv4Addr::from(dest.to_le_bytes());
                let mask = u32::from_be_bytes(mask.to_le_bytes());
                let prefix = mask.leading_ones();
                if flags & 1 != 0 && prefix > 0 && mask == u32::MAX << (32-prefix) {
                    if let Ok(net) = Network::parse(&format!("{dest}/{prefix}")) { out.push(net); }
                }
            }
        }
    }
    if let Ok(interfaces) = std::fs::read_to_string("/proc/net/if_inet6") {
        for line in interfaces.lines() {
            let f: Vec<_> = line.split_whitespace().collect();
            if f.len() >= 3 {
                if let (Ok(ip), Ok(prefix)) = (u128::from_str_radix(f[0],16), u32::from_str_radix(f[2],16)) {
                    let ip = std::net::Ipv6Addr::from(ip);
                    if let Ok(net) = Network::parse(&format!("{ip}/{prefix}")) { out.push(net); }
                }
            }
        }
    }
    Ok(out)
}

pub fn allowed(ip: IpAddr, networks: &[Network]) -> bool {
    let ip = match ip { IpAddr::V6(v) => v.to_ipv4_mapped().map(IpAddr::V4).unwrap_or(ip), _ => ip };
    ip.is_loopback() || networks.iter().any(|n| n.contains(ip))
}

pub async fn lan_only(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    // A proxy can hide the real remote address. Reject explicitly forwarded requests.
    let forwarded = ["forwarded", "x-forwarded-for", "x-real-ip"].iter().any(|h| request.headers().contains_key(*h));
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|v| v.0.ip());
    let cross_origin = request.headers().get("origin").is_some_and(|origin| {
        let origin = origin.to_str().unwrap_or("");
        let authority = origin.strip_prefix("http://").or_else(|| origin.strip_prefix("https://"));
        let host = request.headers().get("host").and_then(|v| v.to_str().ok());
        authority.is_none() || authority != host
    });
    if forwarded || cross_origin || !peer.is_some_and(|ip| allowed(ip, &state.cfg.lan_networks)) {
        return (StatusCode::FORBIDDEN, "LAN access only; use a direct connection").into_response();
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restrict_to_actual_subnet() {
        let nets = vec![Network::parse("192.168.8.0/24").unwrap(), Network::parse("fd12:3456::/64").unwrap()];
        for ip in ["192.168.8.23", "::ffff:192.168.8.23", "127.0.0.1", "::1", "fd12:3456::2"] {
            assert!(allowed(ip.parse().unwrap(), &nets), "{ip}");
        }
        for ip in ["192.168.9.23", "10.0.0.2", "8.8.8.8", "2001:4860::1", "fd12:3457::2"] {
            assert!(!allowed(ip.parse().unwrap(), &nets), "{ip}");
        }
        assert!(Network::parse("0.0.0.0/0").is_err());
        assert!(Network::parse("192.168.0.0/33").is_err());
    }
}
