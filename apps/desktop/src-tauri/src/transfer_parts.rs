//! Read-only, bounded views of the engines' confirmed block journals.
//! Aggregate progress never establishes the position of bytes inside a file.
use apocalipse_core::{chunk_directory, partial_path, DownloadState, DownloadTask};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::Path};

const MAX_JOURNAL_BYTES: u64 = 1 << 20;
const MAX_PARTS: u64 = 2_000_000;
const MAP_CELLS: usize = 128;

#[derive(Clone, Debug, Serialize)]
pub struct PartCell {
    pub start: u64,
    pub end: u64,
    pub confirmed: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct PartsSnapshot {
    pub available: bool,
    pub reason: &'static str,
    pub total: u64,
    pub parts_count: usize,
    pub completed_count: usize,
    pub cells: Vec<PartCell>,
}

impl PartsSnapshot {
    fn unavailable(reason: &'static str, total: u64) -> Self {
        Self {
            available: false,
            reason,
            total,
            parts_count: 0,
            completed_count: 0,
            cells: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
struct XtremeJournal {
    version: u32,
    total: u64,
    block: u64,
    origin: String,
    done: String,
    etag: Option<String>,
    modified: Option<String>,
}

#[derive(Deserialize)]
struct NativeIdentity {
    total: u64,
}

#[derive(Deserialize)]
struct NativeJournal {
    version: u8,
    generation: u64,
    identity: NativeIdentity,
    chunk_size: u64,
    completed: Vec<bool>,
}

fn read_journal<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(MAX_JOURNAL_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_JOURNAL_BYTES {
        return None;
    }
    serde_json::from_slice(&bytes).ok()
}

fn decode_blocks(hex: &str, count: usize) -> Option<Vec<bool>> {
    if !hex.is_ascii() || hex.len() != count.div_ceil(8) * 2 {
        return None;
    }
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok())
        .collect::<Option<Vec<_>>>()?;
    Some((0..count).map(|i| bytes[i / 8] & (1 << (i % 8)) != 0).collect())
}

fn aggregate(total: u64, block: u64, done: &[bool]) -> Option<PartsSnapshot> {
    if total == 0 || block == 0 || total.div_ceil(block) != done.len() as u64 {
        return None;
    }
    let group = done.len().div_ceil(MAP_CELLS).max(1);
    let cells = done
        .chunks(group)
        .enumerate()
        .map(|(index, flags)| {
            let first = index * group;
            let start = first as u64 * block;
            let end = ((first + flags.len()) as u64).saturating_mul(block).min(total);
            let confirmed = flags
                .iter()
                .enumerate()
                .filter(|(_, finished)| **finished)
                .map(|(offset, _)| total.saturating_sub((first + offset) as u64 * block).min(block))
                .sum();
            PartCell { start, end, confirmed }
        })
        .collect();
    Some(PartsSnapshot {
        available: true,
        reason: "checkpoint",
        total,
        parts_count: done.len(),
        completed_count: done.iter().filter(|finished| **finished).count(),
        cells,
    })
}

pub fn snapshot(task: &DownloadTask) -> PartsSnapshot {
    let total = task.total.unwrap_or(0);
    let source = task.source.to_ascii_lowercase();
    let destination = task.destination.to_string_lossy().to_ascii_lowercase();
    if task.is_live || destination.contains(".recording") {
        return PartsSnapshot::unavailable("sequential", total);
    }
    if source.starts_with("magnet:") || source.split(['?', '#']).next().is_some_and(|s| s.ends_with(".torrent")) {
        return PartsSnapshot::unavailable("torrent_unavailable", total);
    }
    if total == 0 {
        return PartsSnapshot::unavailable("unknown_total", total);
    }
    if task.state == DownloadState::Completed {
        // Publication is confirmed by the task, not inferred from received >= total.
        if fs::metadata(&task.destination).ok().is_some_and(|m| m.len() == total) {
            return aggregate(total, total, &[true]).unwrap();
        }
        return PartsSnapshot::unavailable("unavailable", total);
    }
    let mut part = task.destination.as_os_str().to_owned();
    part.push(".nibiru-http.part");
    let mut path = part.clone();
    path.push(".json");
    if let Some(journal) = read_journal::<XtremeJournal>(Path::new(&path)) {
        let origin = url::Url::parse(&task.source).ok().map(|mut url| {
            url.set_query(None);
            url.set_fragment(None);
            format!("{:x}", Sha256::digest(url.as_str().as_bytes()))
        });
        if journal.version == 1
            && journal.total == total
            && journal.block > 0
            && total.div_ceil(journal.block) <= MAX_PARTS
            && origin.as_deref() == Some(journal.origin.as_str())
            && (journal.etag.is_some() || journal.modified.is_some())
            && fs::metadata(Path::new(&part)).ok().is_some_and(|m| m.len() == total)
        {
            if let Some(done) = decode_blocks(&journal.done, total.div_ceil(journal.block) as usize) {
                if let Some(snapshot) = aggregate(total, journal.block, &done) {
                    if snapshot.cells.iter().map(|cell| cell.confirmed).sum::<u64>() <= task.received {
                        return snapshot;
                    }
                }
            }
        }
        return PartsSnapshot::unavailable("unavailable", total);
    }
    let dir = chunk_directory(&task.destination);
    let journal = [dir.join("segments-a.json"), dir.join("segments-b.json")]
        .iter()
        .filter_map(|path| read_journal::<NativeJournal>(path))
        .max_by_key(|journal| journal.generation);
    if let Some(journal) = journal {
        if journal.version == 1
            && journal.identity.total == total
            && journal.chunk_size > 0
            && total.div_ceil(journal.chunk_size) <= MAX_PARTS
            && fs::metadata(partial_path(&task.destination)).ok().is_some_and(|m| m.len() == total)
        {
            if let Some(snapshot) = aggregate(total, journal.chunk_size, &journal.completed) {
                if snapshot.cells.iter().map(|cell| cell.confirmed).sum::<u64>() <= task.received {
                    return snapshot;
                }
            }
        }
    }
    PartsSnapshot::unavailable("unavailable", total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmap_preserves_holes_and_last_short_block() {
        let flags = decode_blocks("05", 3).unwrap();
        let map = aggregate(25, 10, &flags).unwrap();
        assert_eq!(map.completed_count, 2);
        assert_eq!(map.cells.iter().map(|cell| cell.confirmed).collect::<Vec<_>>(), vec![10, 0, 5]);
        assert!(decode_blocks("zz", 3).is_none());
        assert!(decode_blocks("0", 3).is_none());
        assert!(aggregate(25, 0, &flags).is_none());
        assert!(aggregate(40, 10, &flags).is_none());
    }

    #[test]
    fn maps_are_bounded_and_preserve_confirmed_bytes() {
        let flags: Vec<_> = (0..10_000).map(|i| i % 3 == 0).collect();
        let map = aggregate(99_995, 10, &flags).unwrap();
        assert!(map.cells.len() <= MAP_CELLS);
        assert_eq!(map.cells.iter().map(|cell| cell.confirmed).sum::<u64>(), 33_335);
    }

    #[test]
    fn aggregate_percent_never_invents_a_byte_map() {
        let directory = tempfile::tempdir().unwrap();
        let mut task = DownloadTask::new("https://example.com/file", directory.path().join("file"));
        task.total = Some(100);
        task.received = 90;
        assert!(!snapshot(&task).available);
        task.is_live = true;
        assert_eq!(snapshot(&task).reason, "sequential");
        task.is_live = false;
        task.source = "magnet:?xt=urn:btih:example".to_owned();
        assert_eq!(snapshot(&task).reason, "torrent_unavailable");
    }

    #[test]
    fn xtreme_checkpoint_is_bound_to_source_size_and_real_partial() {
        let directory = tempfile::tempdir().unwrap();
        let destination = directory.path().join("file");
        let mut task = DownloadTask::new("https://example.com/file?token=new", &destination);
        task.total = Some(25);
        task.received = 15;
        fs::File::create(directory.path().join("file.nibiru-http.part")).unwrap().set_len(25).unwrap();
        let path = directory.path().join("file.nibiru-http.part.json");
        let origin = format!("{:x}", Sha256::digest(b"https://example.com/file"));
        let mut record = serde_json::json!({"version":1,"total":25,"block":10,"origin":origin,"done":"05","etag":"v1","modified":null});
        fs::write(&path, record.to_string()).unwrap();
        assert_eq!(snapshot(&task).completed_count, 2);
        record["origin"] = serde_json::json!("another-resource");
        fs::write(&path, record.to_string()).unwrap();
        assert!(!snapshot(&task).available);
        fs::write(&path, vec![b' '; MAX_JOURNAL_BYTES as usize + 1]).unwrap();
        assert!(!snapshot(&task).available);
    }
}
