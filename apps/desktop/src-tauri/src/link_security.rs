//! Path containment and shared authentication admission for Apocalipse Link.
use std::{
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};

/// Checks the deepest existing ancestor, including dangling symbolic links.
/// Local changes to links between this check and opening a file remain a separate race.
pub(crate) fn stays_inside_share(root: &Path, path: &Path) -> bool {
    let Ok(root) = root.canonicalize() else {
        return false;
    };
    let mut probe = path;
    loop {
        match probe.symlink_metadata() {
            Ok(_) => {
                // An existing but unresolvable entry (e.g. a dangling symlink)
                // must not be treated as a harmless new filename.
                return probe
                    .canonicalize()
                    .is_ok_and(|resolved| resolved.starts_with(&root));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return false,
        }
        match probe.parent() {
            Some(parent) => probe = parent,
            None => return false,
        }
    }
}

pub(crate) struct AuthBudget {
    attempts: Mutex<Vec<Instant>>,
}

impl AuthBudget {
    const MAX_ATTEMPTS: usize = 8;
    const WINDOW: Duration = Duration::from_secs(60);

    pub(crate) const fn new() -> Self {
        Self {
            attempts: Mutex::new(Vec::new()),
        }
    }

    /// Reserve before authenticating, so parallel slow attempts cannot bypass the limit.
    /// Applies to successful and failed logins; an established token is reused afterwards.
    pub(crate) fn try_acquire(&self, now: Instant) -> bool {
        let mut attempts = self.attempts.lock().unwrap_or_else(|e| e.into_inner());
        attempts.retain(|at| now.saturating_duration_since(*at) < Self::WINDOW);
        if attempts.len() >= Self::MAX_ATTEMPTS {
            return false;
        }
        attempts.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_logins_reserve_only_eight_slots_before_any_authentication_finishes() {
        let budget = std::sync::Arc::new(AuthBudget::new());
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(32));
        let now = Instant::now();
        let tasks: Vec<_> = (0..32)
            .map(|_| {
                let budget = budget.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    budget.try_acquire(now)
                })
            })
            .collect();
        assert_eq!(
            tasks
                .into_iter()
                .filter_map(|t| t.join().ok())
                .filter(|allowed| *allowed)
                .count(),
            8
        );
        assert!(!budget.try_acquire(now + Duration::from_secs(59)));
        assert!(budget.try_acquire(now + Duration::from_secs(60)));
    }

    #[cfg(unix)]
    #[test]
    fn shares_reject_external_and_dangling_links_but_allow_internal_aliases_and_new_children() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("adm-link-audit-{}-{unique}", std::process::id()));
        let shared = root.join("shared");
        let outside = root.join("outside");
        std::fs::create_dir_all(shared.join("inside")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("secret"), b"secret").unwrap();
        std::os::unix::fs::symlink(&outside, shared.join("escape")).unwrap();
        std::os::unix::fs::symlink(outside.join("not-created"), shared.join("dangling")).unwrap();
        std::os::unix::fs::symlink(shared.join("inside"), shared.join("alias")).unwrap();
        assert!(!stays_inside_share(&shared, &shared.join("escape/secret")));
        assert!(!stays_inside_share(&shared, &shared.join("escape/new")));
        assert!(!stays_inside_share(&shared, &shared.join("dangling")));
        assert!(!stays_inside_share(&shared, &shared.join("dangling/new")));
        assert!(stays_inside_share(&shared, &shared.join("alias/new")));
        assert!(stays_inside_share(
            &shared,
            &shared.join("inside/new/child")
        ));
        assert!(!stays_inside_share(&shared, &outside));
        std::fs::remove_dir_all(root).unwrap();
    }
}
