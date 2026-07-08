//! Shared, hardened I/O helpers for reading user-supplied input files.
//!
//! Every parser in the workspace (IFC, DXF, native `.ocm`) ultimately reads a
//! file whose path and contents come from outside the application. Two things
//! must be true for that to be safe:
//!
//!  1. The path must not escape via `..` traversal.
//!  2. The read must be bounded — a malformed or hostile multi-gigabyte file
//!     must fail cleanly rather than exhaust memory and crash the process
//!     (a trivial denial-of-service otherwise).
//!
//! Centralizing both concerns here means every entry point gets the same
//! guarantees instead of each parser re-implementing (and potentially
//! forgetting) them.

use anyhow::{bail, Result};
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path};

/// Hard cap on the size of any user-supplied file we read fully into memory.
///
/// Real-world IFC building models — the largest input this tool handles — run
/// to a few hundred megabytes even for very large projects. 1 GiB leaves ample
/// headroom for legitimate files while still refusing the pathological
/// multi-gigabyte inputs that would otherwise OOM the app. Exposed publicly so
/// callers (and tests) can reason about the limit.
pub const MAX_INPUT_FILE_BYTES: u64 = 1024 * 1024 * 1024; // 1 GiB

/// Rejects a path containing a `..` component (directory traversal).
///
/// Absolute paths are intentionally allowed: the desktop file-open dialog
/// hands us absolute paths, and that is the normal, legitimate case. The guard
/// exists to stop a crafted relative path (e.g. from a project file or an
/// automation script) from reaching outside the intended directory.
pub fn reject_path_traversal(path: &Path) -> Result<()> {
    if path.components().any(|c| c == Component::ParentDir) {
        bail!("Invalid input path (contains '..'): {}", path.display());
    }
    Ok(())
}

/// Reads a reader into a `String`, but never more than [`MAX_INPUT_FILE_BYTES`].
///
/// The bound is enforced *during* the read via [`Read::take`], so it holds
/// regardless of what the filesystem reports for the file's size — it is not
/// fooled by a growing file, a named pipe, or a stat/read race. At most
/// `MAX_INPUT_FILE_BYTES + 1` bytes are ever buffered.
fn read_capped<R: Read>(reader: R) -> Result<String> {
    let mut limited = reader.take(MAX_INPUT_FILE_BYTES + 1);
    let mut buf = String::new();
    limited.read_to_string(&mut buf)?;
    if buf.len() as u64 > MAX_INPUT_FILE_BYTES {
        bail!(
            "Input file exceeds the maximum allowed size of {} bytes",
            MAX_INPUT_FILE_BYTES
        );
    }
    Ok(buf)
}

/// Opens `path` and reads it fully into a `String`, applying both the
/// path-traversal guard and the size cap. This is the single entry point every
/// text-based parser (IFC, `.ocm`) should use instead of [`std::fs::read_to_string`].
pub fn read_to_string_bounded(path: &Path) -> Result<String> {
    reject_path_traversal(path)?;
    let file = File::open(path)?;
    read_capped(file)
}

/// Validates a path destined for a reader-based parser (e.g. the DXF loader,
/// which reads the file internally) without reading it ourselves.
///
/// Applies the traversal guard and refuses files whose on-disk size already
/// exceeds the cap, giving a clean error before the third-party parser
/// allocates anything. Returns an error if the file cannot be stat-ed.
pub fn guard_input_file(path: &Path) -> Result<()> {
    reject_path_traversal(path)?;
    let len = std::fs::metadata(path)?.len();
    if len > MAX_INPUT_FILE_BYTES {
        bail!(
            "Input file exceeds the maximum allowed size of {} bytes",
            MAX_INPUT_FILE_BYTES
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_capped_accepts_normal_input() {
        let data = "IFCWALL('x');\n".repeat(10);
        let out = read_capped(Cursor::new(data.clone().into_bytes())).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn test_read_capped_rejects_oversized_input() {
        // A reader that would yield more than the cap must be refused, and must
        // never buffer more than cap + 1 bytes regardless of how much it could
        // have produced. `io::repeat` is effectively infinite — if the cap were
        // not enforced during the read this test would hang/OOM.
        let endless = std::io::repeat(b'#');
        let err = read_capped(endless).unwrap_err();
        assert!(
            err.to_string().contains("maximum allowed size"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_reject_path_traversal() {
        assert!(reject_path_traversal(Path::new("../../etc/passwd")).is_err());
        assert!(reject_path_traversal(Path::new("model.ifc")).is_ok());
        // Absolute paths (as produced by the file dialog) are allowed.
        assert!(reject_path_traversal(Path::new("/home/user/model.ifc")).is_ok());
    }

    #[test]
    fn test_read_to_string_bounded_reads_file() {
        let path = std::env::temp_dir().join(format!("ocm_io_test_{}.txt", uuid::Uuid::new_v4()));
        std::fs::write(&path, "hello bounded read").unwrap();
        let out = read_to_string_bounded(&path).unwrap();
        assert_eq!(out, "hello bounded read");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_read_to_string_bounded_rejects_traversal() {
        assert!(read_to_string_bounded(Path::new("../../etc/passwd")).is_err());
    }

    #[test]
    fn test_guard_input_file_accepts_small_file() {
        let path = std::env::temp_dir().join(format!("ocm_io_guard_{}.dxf", uuid::Uuid::new_v4()));
        std::fs::write(&path, "0\nSECTION\n").unwrap();
        assert!(guard_input_file(&path).is_ok());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_guard_input_file_rejects_traversal() {
        assert!(guard_input_file(Path::new("../../etc/passwd.dxf")).is_err());
    }
}
