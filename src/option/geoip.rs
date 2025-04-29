use cidr_utils::cidr::IpCidr;
use hashbrown::HashMap;
use maxminddb::{Reader, geoip2};
use std::net::IpAddr;
use std::path::Path;
use std::str::FromStr;

pub struct GeoIPSearch {
    asn: Reader<Vec<u8>>,
    asn_cache: HashMap<IpAddr, String>,
    country: Reader<Vec<u8>>,
    country_cache: HashMap<IpAddr, String>,
    city: Reader<Vec<u8>>,
    city_cache: HashMap<IpAddr, String>,
}

impl GeoIPSearch {
    pub fn new(geo_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let asn = Reader::open_readfile(geo_path.join("GeoLite2-ASN.mmdb"))?;
        let country = Reader::open_readfile(geo_path.join("GeoLite2-Country.mmdb"))?;
        let city = Reader::open_readfile(geo_path.join("GeoLite2-City.mmdb"))?;

        Ok(GeoIPSearch {
            asn,
            asn_cache: HashMap::new(),
            country,
            country_cache: HashMap::new(),
            city,
            city_cache: HashMap::new(),
        })
    }

    pub fn convert(&self, ip: &str) -> Option<IpAddr> {
        let ip = if ip.starts_with("::ffff:") {
            ip.replace("::ffff:", "")
        } else {
            ip.to_string()
        };
        if let Ok(ip) = IpAddr::from_str(&ip) {
            return Some(ip);
        }
        None
    }

    pub fn check_in_private_ip_range(&self, target_ip: &IpAddr) -> bool {
        let private_cidr = if target_ip.is_ipv4() {
            vec![
                IpCidr::from_str("10/8").unwrap(),
                IpCidr::from_str("172.16/12").unwrap(),
                IpCidr::from_str("192.168/16").unwrap(),
            ]
        } else {
            vec![
                IpCidr::from_str("::/128").unwrap(),    // IPv6 Unspecified
                IpCidr::from_str("2000::/3").unwrap(),  // IPv6 Global Unicast
                IpCidr::from_str("FE80::/10").unwrap(), // IPv6 Link Local Unicast
                IpCidr::from_str("FC00::/7").unwrap(),  // IPv6 Unique Local Address
                IpCidr::from_str("FD00::/8").unwrap(),  // IPv6 Unique Local Address
                IpCidr::from_str("FF00::/8").unwrap(),  // IPv6 Multicast Address
            ]
        };
        for cidr in private_cidr {
            if cidr.contains(target_ip) {
                return true;
            }
        }
        false
    }

    pub fn get_asn(&mut self, ip: IpAddr) -> String {
        if ip.is_loopback() {
            return "Local".to_string();
        }
        if self.check_in_private_ip_range(&ip) {
            return "Private".to_string();
        }
        if let Some(asn) = self.asn_cache.get(&ip) {
            return asn.to_string();
        }
        match self.asn.lookup::<geoip2::Asn>(ip) {
            Ok(Some(asn)) => {
                let asn_str = asn.autonomous_system_organization.unwrap_or("-");
                self.asn_cache.insert(ip, asn_str.to_string());
                asn_str.to_string()
            }
            _ => "-".to_string(),
        }
    }

    pub fn get_country(&mut self, ip: IpAddr) -> String {
        if ip.is_loopback() || self.check_in_private_ip_range(&ip) {
            return "-".to_string();
        }
        if let Some(country) = self.country_cache.get(&ip) {
            return country.to_string();
        }
        match self.country.lookup::<geoip2::Country>(ip) {
            Ok(Some(country)) => {
                let mut ret = "-";
                if let Some(country) = country.country {
                    if let Some(name_tree) = country.names {
                        ret = name_tree.get("en").unwrap_or(&"-")
                    }
                }
                ret.to_string()
            }
            _ => "-".to_string(),
        }
    }

    pub fn get_city(&mut self, ip: IpAddr) -> String {
        if ip.is_loopback() || self.check_in_private_ip_range(&ip) {
            return "-".to_string();
        }
        if let Some(city) = self.city_cache.get(&ip) {
            return city.to_string();
        }
        match self.city.lookup::<geoip2::City>(ip) {
            Ok(Some(city)) => {
                let mut ret = "-";
                if let Some(city) = city.city {
                    if let Some(name_tree) = city.names {
                        ret = name_tree.get("en").unwrap_or(&"-")
                    }
                }
                ret.to_string()
            }
            _ => "-".to_string(),
        }
    }
}
