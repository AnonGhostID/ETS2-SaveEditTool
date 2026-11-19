//! Explicit-save staging for `game.sii` edits.
//!
//! When the GUI enables staging, save edits are held in memory instead of
//! being written to disk. The staged content overlays the disk file for
//! every read (`decrypt_if_needed`), so the UI shows pending edits, and
//! nothing reaches the disk until the user presses Save in the top bar.
//!
//! Staging is OFF by default. Only the GUI binary turns it on (in
//! `main()`); tests and the data-builder binary keep the old
//! write-to-disk behavior, so existing tests pass unchanged.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Default)]
struct SessionInner {
    staging_enabled: bool,
    /// Staged working copies, keyed by save path. Normally one entry (the
    /// active save); the autosave can add a second one.
    staged: BTreeMap<PathBuf, String>,
}

fn session() -> &'static Mutex<SessionInner> {
    static SESSION: OnceLock<Mutex<SessionInner>> = OnceLock::new();
    SESSION.get_or_init(|| Mutex::new(SessionInner::default()))
}

fn lock() -> std::sync::MutexGuard<'static, SessionInner> {
    session().lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Turn staging on (GUI startup) or off. Turning it off does not flush
/// or discard staged content; use `discard` for that.
pub fn set_staging_enabled(enabled: bool) {
    lock().staging_enabled = enabled;
}

pub fn is_staging_enabled() -> bool {
    lock().staging_enabled
}

/// Store `content` as the pending version of `path`. No-op when staging
/// is disabled (callers fall through to the disk write instead).
pub fn stage_save(path: &Path, content: &str) {
    let mut inner = lock();
    if !inner.staging_enabled {
        return;
    }
    inner
        .staged
        .insert(path.to_path_buf(), content.to_string());
}

/// The staged working copy for `path`, if staging is on and one exists.
pub fn staged_content(path: &Path) -> Option<String> {
    let inner = lock();
    if !inner.staging_enabled {
        return None;
    }
    inner.staged.get(path).cloned()
}

/// Whether any save has pending (unflushed) edits.
pub fn is_dirty() -> bool {
    !lock().staged.is_empty()
}

/// Paths with pending edits, for the dirty indicator.
pub fn dirty_paths() -> Vec<PathBuf> {
    lock().staged.keys().cloned().collect()
}

/// Drop the staged working copy for one path, if any. Used when a
/// write is rolled back (SW3) so rejected content cannot be flushed
/// later by a top-bar Save.
pub fn remove_staged(path: &Path) {
    lock().staged.remove(path);
}

/// Drop all staged content. The disk files are untouched.
pub fn discard() {
    lock().staged.clear();
}

/// Take all staged edits for flushing to disk. On flush failure the
/// caller re-stages the returned entries so the user can retry.
pub fn take_for_flush() -> Vec<(PathBuf, String)> {
    std::mem::take(&mut lock().staged)
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RAII guard: enables staging for a test and restores the previous
    /// state (off + empty) on drop, so parallel tests never see it.
    struct StagingGuard {
        was_enabled: bool,
    }

    impl StagingGuard {
        fn enable() -> Self {
            let was_enabled = is_staging_enabled();
            set_staging_enabled(true);
            Self { was_enabled }
        }
    }

    impl Drop for StagingGuard {
        fn drop(&mut self) {
            discard();
            set_staging_enabled(self.was_enabled);
        }
    }

    #[test]
    fn staging_is_off_by_default() {
        // No guard: this test only reads the default state.
        assert!(!is_staging_enabled());
        assert!(!is_dirty());
    }

    #[test]
    fn staged_content_roundtrip_and_discard() {
        let _guard = StagingGuard::enable();
        let path = PathBuf::from("/tmp/ets2-staging-test/game.sii");
        let other = PathBuf::from("/tmp/ets2-staging-test/other.sii");
        assert_eq!(staged_content(&path), None);
        stage_save(&path, "staged content");
        assert!(is_dirty());
        assert_eq!(staged_content(&path).as_deref(), Some("staged content"));
        assert_eq!(staged_content(&other), None);
        assert_eq!(dirty_paths(), vec![path.clone()]);
        discard();
        assert!(!is_dirty());
        assert_eq!(staged_content(&path), None);
    }

    #[test]
    fn take_for_flush_drains_and_allows_restage() {
        let _guard = StagingGuard::enable();
        let path = PathBuf::from("/tmp/ets2-staging-test/game.sii");
        stage_save(&path, "v1");
        stage_save(&path, "v2");
        let taken = take_for_flush();
        assert_eq!(taken.len(), 1);
        assert_eq!(taken[0].1, "v2");
        assert!(!is_dirty());
        // Simulate a failed flush: re-stage so the user can retry.
        for (path, content) in taken {
            stage_save(&path, &content);
        }
        assert!(is_dirty());
        assert_eq!(staged_content(&path).as_deref(), Some("v2"));
    }

    #[test]
    fn stage_save_is_a_noop_when_disabled() {
        // Staging off (default); stage_save must not record anything.
        let path = PathBuf::from("/tmp/ets2-staging-test/disabled.sii");
        stage_save(&path, "must not stick");
        assert!(!is_dirty());
        assert_eq!(staged_content(&path), None);
    }
}
