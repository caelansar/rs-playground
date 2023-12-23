use ip_network::IpNetwork;
use ip_network_table::IpNetworkTable;

use std::collections::VecDeque;
use std::net::IpAddr;

/// A trie of IP/cidr addresses
#[derive(Default)]
pub struct AllowedIps<D> {
    ips: IpNetworkTable<D>,
}

impl<D> AllowedIps<D> {
    pub fn new() -> Self {
        Self {
            ips: IpNetworkTable::new(),
        }
    }

    pub fn clear(&mut self) {
        self.ips = IpNetworkTable::new();
    }

    pub fn insert(&mut self, key: IpAddr, cidr: u32, data: D) -> Option<D> {
        self.ips.insert(
            IpNetwork::new_truncate(key, cidr as u8).expect("cidr is valid length"),
            data,
        )
    }

    pub fn find(&self, key: IpAddr) -> Option<&D> {
        self.ips.longest_match(key).map(|(_net, data)| data)
    }

    pub fn remove(&mut self, predicate: &dyn Fn(&D) -> bool) {
        self.ips.retain(|_, v| !predicate(v));
    }

    pub fn iter(&self) -> Iter<D> {
        Iter(
            self.ips
                .iter()
                .map(|(ipa, d)| (d, ipa.network_address(), ipa.netmask()))
                .collect(),
        )
    }
}

pub struct Iter<'a, D: 'a>(VecDeque<(&'a D, IpAddr, u8)>);

impl<'a, D> Iterator for Iter<'a, D> {
    type Item = (&'a D, IpAddr, u8);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_allowed_ips() -> AllowedIps<&'static str> {
        let mut map: AllowedIps<&'static str> = Default::default();
        map.insert(IpAddr::from([127, 0, 0, 1]), 32, "client 1");
        map.insert(IpAddr::from([127, 0, 15, 1]), 16, "client 2");
        map.insert(IpAddr::from([127, 1, 15, 1]), 24, "client 3");
        map
    }

    #[test]
    fn test_allowed_ips_insert_find() {
        let map = build_allowed_ips();
        assert_eq!(map.find(IpAddr::from([127, 0, 0, 1])), Some(&"client 1"));
        assert_eq!(
            map.find(IpAddr::from([127, 0, 255, 255])),
            Some(&"client 2")
        );
        assert_eq!(map.find(IpAddr::from([127, 1, 15, 255])), Some(&"client 3"));
        assert_eq!(
            map.find(IpAddr::from([127, 0, 255, 255])),
            Some(&"client 2")
        );
        assert_eq!(map.find(IpAddr::from([127, 1, 15, 255])), Some(&"client 3"));
        assert_eq!(map.find(IpAddr::from([127, 2, 15, 255])), None);
    }
}
