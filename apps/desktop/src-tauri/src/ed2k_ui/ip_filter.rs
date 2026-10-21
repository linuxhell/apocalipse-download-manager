//! IPv4 range filter for ED2K/Kad peers, loaded from an `ipfilter.dat` file.
//!
//! Two common text layouts are read: the eMule/aMule form
//! `001.002.003.000 - 001.002.003.255 , 100 , description` and the
//! PeerGuardian form `description:001.002.003.000-001.002.003.255`. As in
//! those clients, a range is blocked when its access level is below the filter
//! level (127 unless changed); PeerGuardian lines carry no level and count as 0.
//! IPv6 lines and malformed lines are skipped and counted, never fatal.
use std::{net::Ipv4Addr, path::Path};

pub const DEFAULT_LEVEL: u16 = 127;
const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug, Default)]
pub struct IpFilter {
    /// Sorted, non-overlapping, non-adjacent inclusive ranges.
    ranges: Vec<(u32, u32)>,
    skipped: usize,
}

fn octet(text: &str) -> Option<u32> {
    let text = text.trim();
    if text.is_empty() || text.len() > 3 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<u32>().ok().filter(|value| *value <= 255)
}

/// Dotted quad that tolerates the zero padding used by these lists (`001.002.003.004`).
fn address(text: &str) -> Option<u32> {
    let mut parts = text.trim().split('.');
    let mut value = 0u32;
    for _ in 0..4 {
        value = (value << 8) | octet(parts.next()?)?;
    }
    parts.next().is_none().then_some(value)
}

fn range(text: &str) -> Option<(u32, u32)> {
    let (start, end) = text.split_once('-')?;
    let (start, end) = (address(start)?, address(end)?);
    (start <= end).then_some((start, end))
}

impl IpFilter {
    pub fn parse(text: &str, level: u16) -> Self {
        let mut ranges = Vec::new();
        let mut skipped = 0;
        for line in text.lines() {
            let line = line.trim().trim_start_matches('\u{feff}');
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            let parsed = match line.split_once(',') {
                // eMule layout: "range , level , description".
                Some((span, rest)) => range(span).map(|span| {
                    let access = rest
                        .split(',')
                        .next()
                        .and_then(|value| value.trim().parse::<u16>().ok())
                        .unwrap_or(0);
                    (span, access)
                }),
                None => None,
            }
            .or_else(|| {
                // PeerGuardian layout: "description:range"; no level means blocked.
                line.rsplit_once(':')
                    .and_then(|(_, span)| range(span))
                    .map(|span| (span, 0))
            });
            match parsed {
                Some((span, access)) if access < level => ranges.push(span),
                Some(_) => {}
                None => skipped += 1,
            }
        }
        ranges.sort_unstable();
        let mut merged: Vec<(u32, u32)> = Vec::with_capacity(ranges.len());
        for (start, end) in ranges {
            match merged.last_mut() {
                Some(last) if start <= last.1.saturating_add(1) => last.1 = last.1.max(end),
                _ => merged.push((start, end)),
            }
        }
        Self {
            ranges: merged,
            skipped,
        }
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let metadata = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
            return Err("ip_filter_file_invalid".into());
        }
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        Ok(Self::parse(&String::from_utf8_lossy(&bytes), DEFAULT_LEVEL))
    }

    pub fn blocks(&self, ip: Ipv4Addr) -> bool {
        let value = u32::from(ip);
        let after = self.ranges.partition_point(|(start, _)| *start <= value);
        after > 0 && self.ranges[after - 1].1 >= value
    }

    /// Number of distinct blocked ranges after merging.
    pub fn len(&self) -> usize {
        self.ranges.len()
    }

    /// Lines that were neither comments nor a readable IPv4 range.
    pub fn skipped(&self) -> usize {
        self.skipped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(text: &str) -> Ipv4Addr {
        text.parse().unwrap()
    }

    #[test]
    fn emule_ranges_block_below_the_level_and_tolerate_zero_padding() {
        let filter = IpFilter::parse(
            "# comment\n\
             // another\n\
             001.002.003.000 - 001.002.003.255 , 100 , blocked range\n\
             010.000.000.000 - 010.000.000.255 , 200 , allowed range\n\
             8.8.8.8-8.8.8.8, 127, boundary level is allowed\n",
            DEFAULT_LEVEL,
        );
        assert!(filter.blocks(ip("1.2.3.0")));
        assert!(filter.blocks(ip("1.2.3.77")));
        assert!(filter.blocks(ip("1.2.3.255")));
        assert!(!filter.blocks(ip("1.2.2.255")));
        assert!(!filter.blocks(ip("1.2.4.0")));
        assert!(!filter.blocks(ip("10.0.0.5")), "level 200 is not blocked");
        assert!(
            !filter.blocks(ip("8.8.8.8")),
            "level equal to the filter is allowed"
        );
        assert_eq!(filter.len(), 1);
        assert_eq!(filter.skipped(), 0);
    }

    #[test]
    fn peerguardian_lines_block_and_garbage_is_counted_not_fatal() {
        let filter = IpFilter::parse(
            "Some Org:5.6.7.0-5.6.7.255\n\
             IPv6 range:2001:db8::-2001:db8::ffff\n\
             nonsense line\n\
             999.1.1.1 - 999.1.1.2 , 1 , bad octet\n\
             9.9.9.9 - 9.9.9.1 , 1 , reversed\n",
            DEFAULT_LEVEL,
        );
        assert!(filter.blocks(ip("5.6.7.200")));
        assert!(!filter.blocks(ip("5.6.8.1")));
        assert_eq!(filter.len(), 1);
        assert_eq!(filter.skipped(), 4);
    }

    #[test]
    fn overlapping_and_adjacent_ranges_merge_and_lookup_is_exact_at_the_edges() {
        let filter = IpFilter::parse(
            "1.0.0.0-1.0.0.9,1\n1.0.0.5-1.0.0.20,1\n1.0.0.21-1.0.0.30,1\n1.0.0.40-1.0.0.50,1\n\
             0.0.0.0-0.0.0.0,1\n255.255.255.255-255.255.255.255,1\n",
            DEFAULT_LEVEL,
        );
        assert_eq!(filter.len(), 4);
        for blocked in [
            "1.0.0.0",
            "1.0.0.30",
            "1.0.0.40",
            "1.0.0.50",
            "0.0.0.0",
            "255.255.255.255",
        ] {
            assert!(filter.blocks(ip(blocked)), "{blocked}");
        }
        for allowed in [
            "0.255.255.255",
            "1.0.0.31",
            "1.0.0.39",
            "1.0.0.51",
            "255.255.255.254",
        ] {
            assert!(!filter.blocks(ip(allowed)), "{allowed}");
        }
        assert!(!IpFilter::default().blocks(ip("1.2.3.4")));
    }

    #[test]
    fn loading_rejects_directories_and_missing_files() {
        let root = tempfile::tempdir().unwrap();
        assert!(IpFilter::load(root.path()).is_err());
        assert!(IpFilter::load(&root.path().join("missing.dat")).is_err());
        let path = root.path().join("ipfilter.dat");
        std::fs::write(&path, "\u{feff}7.7.7.0 - 7.7.7.255 , 0 , bom first line\n").unwrap();
        assert!(IpFilter::load(&path).unwrap().blocks(ip("7.7.7.1")));
    }
}
