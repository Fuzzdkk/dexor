//! the guts. a file gets "scrambled" by xoring every byte with a single key
//! byte — xor is its own inverse, so doing the exact same thing again gives you
//! the original file straight back. so all this module really does is that one
//! xor + the walk-the-folder bookkeeping. keeping it separate from the gui so
//! the tests can hit it directly.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Key the UI starts on. Single-byte XOR, change it in the app if you need to.
pub const DEFAULT_XOR_KEY: u8 = 0x77;

/// XOR every byte of `data` with `key`, in place.
///
/// This is the whole transform: it is symmetric, so the same call both
/// encodes and decodes.
pub fn xor_in_place(data: &mut [u8], key: u8) {
    for b in data.iter_mut() {
        *b ^= key;
    }
}

/// A single unit of work: read `source`, transform it, and write the result
/// to `dest_rel` resolved against the chosen output directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// Absolute path of the file to read.
    pub source: PathBuf,
    /// Path of the output file, relative to the output directory.
    pub dest_rel: PathBuf,
}

/// Expand a list of input paths (files and/or directories) into a flat list
/// of [`Job`]s.
///
/// * A *file* `foo.bin` produces one job writing `foo.bin` into the output root.
/// * A *directory* `in/` is walked recursively; every file inside is reproduced
///   under `in/...` in the output, preserving the relative tree.
pub fn collect_jobs(paths: &[PathBuf]) -> io::Result<Vec<Job>> {
    let mut jobs = Vec::new();
    for path in paths {
        let meta = fs::metadata(path)?;
        if meta.is_file() {
            let name = path
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("decoded.bin"));
            jobs.push(Job {
                source: path.clone(),
                dest_rel: name,
            });
        } else if meta.is_dir() {
            // Use the folder's own name as the root in the output.
            let root_name = path
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("decoded"));
            walk_dir(path, &root_name, &mut jobs)?;
        }
    }
    Ok(jobs)
}

fn walk_dir(dir: &Path, dest_prefix: &Path, jobs: &mut Vec<Job>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_rel = dest_prefix.join(entry.file_name());
        if file_type.is_dir() {
            walk_dir(&entry.path(), &dest_rel, jobs)?;
        } else if file_type.is_file() {
            jobs.push(Job {
                source: entry.path(),
                dest_rel,
            });
        }
        // Symlinks and other special files are skipped on purpose.
    }
    Ok(())
}

/// Run a single job: read the source, XOR it with `key`, and write the result
/// to `output_dir / job.dest_rel`, creating parent directories as needed.
/// Returns the absolute path that was written.
pub fn run_job(job: &Job, output_dir: &Path, key: u8) -> io::Result<PathBuf> {
    let mut data = fs::read(&job.source)?;
    xor_in_place(&mut data, key);

    let dest = output_dir.join(&job.dest_rel);
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dest, &data)?;
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xor_is_symmetric() {
        let original = b"MZ\x90\x00\x03 the original program bytes".to_vec();
        let mut buf = original.clone();
        xor_in_place(&mut buf, DEFAULT_XOR_KEY); // scramble
        assert_ne!(buf, original, "scrambled bytes should differ");
        xor_in_place(&mut buf, DEFAULT_XOR_KEY); // decode
        assert_eq!(buf, original, "decode must reproduce original exactly");
    }

    #[test]
    fn known_value() {
        let mut b = [0x00u8, 0xFF, 0x77];
        xor_in_place(&mut b, DEFAULT_XOR_KEY);
        assert_eq!(b, [0x77, 0x88, 0x00]);
    }

    #[test]
    fn collect_single_file_and_dir() {
        let tmp = std::env::temp_dir().join(format!("dexor-test-collect-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("q/sub")).unwrap();
        fs::write(tmp.join("loose.bin"), b"x").unwrap();
        fs::write(tmp.join("q/a.exe"), b"x").unwrap();
        fs::write(tmp.join("q/sub/b.dll"), b"x").unwrap();

        let mut jobs =
            collect_jobs(&[tmp.join("loose.bin"), tmp.join("q")]).unwrap();
        jobs.sort_by(|a, b| a.dest_rel.cmp(&b.dest_rel));

        let dests: Vec<_> = jobs.iter().map(|j| j.dest_rel.clone()).collect();
        assert_eq!(
            dests,
            vec![
                PathBuf::from("loose.bin"),
                PathBuf::from("q/a.exe"),
                PathBuf::from("q/sub/b.dll"),
            ]
        );
        let _ = fs::remove_dir_all(&tmp);
    }
}
