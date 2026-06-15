//! the guts. a file gets "scrambled" by xoring every byte with a single key
//! byte — xor is its own inverse, so doing the exact same thing again gives you
//! the original file straight back. so all this module really does is that one
//! xor + the walk-the-folder bookkeeping. keeping it separate from the gui so
//! the tests can hit it directly.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Key the UI starts on. Single-byte XOR, change it in the app if you need to.
pub const DEFAULT_XOR_KEY: u8 = 0x77;

/// Name of the folder created next to each input to hold the decoded files.
pub const OUTPUT_DIR_NAME: &str = "dexor-decoded";

/// Prepended to every output filename so it's obvious it came out of here.
pub const FILENAME_PREFIX: &str = "dexor_";

/// XOR every byte of `data` with `key`, in place.
///
/// This is the whole transform: it is symmetric, so the same call both
/// encodes and decodes.
pub fn xor_in_place(data: &mut [u8], key: u8) {
    for b in data.iter_mut() {
        *b ^= key;
    }
}

/// `dexor_` + the original file name.
fn prefixed(name: &OsStr) -> OsString {
    let mut s = OsString::from(FILENAME_PREFIX);
    s.push(name);
    s
}

/// A single unit of work: read `source`, transform it, write to `dest`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// Path of the file to read.
    pub source: PathBuf,
    /// Absolute path to write the decoded file to.
    pub dest: PathBuf,
}

/// Expand a list of input paths (files and/or directories) into a flat list
/// of [`Job`]s.
///
/// There's no separate output folder — each input writes into a new
/// `dexor-decoded/` folder created right next to it, and every output file name
/// is prefixed with `dexor_`:
///
/// * A *file* `/a/b/foo.bin` -> `/a/b/dexor-decoded/dexor_foo.bin`.
/// * A *folder* `/a/b/in/` is walked recursively, reproduced under
///   `/a/b/dexor-decoded/in/...` with each leaf file prefixed.
pub fn collect_jobs(paths: &[PathBuf]) -> io::Result<Vec<Job>> {
    let mut jobs = Vec::new();
    for path in paths {
        let meta = fs::metadata(path)?;
        // the new folder sits in the same directory as the dropped item.
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let out_base = parent.join(OUTPUT_DIR_NAME);

        if meta.is_file() {
            let name = path.file_name().unwrap_or_else(|| OsStr::new("decoded.bin"));
            jobs.push(Job {
                source: path.clone(),
                dest: out_base.join(prefixed(name)),
            });
        } else if meta.is_dir() {
            let dir_name = path.file_name().unwrap_or_else(|| OsStr::new("decoded"));
            walk_dir(path, &out_base.join(dir_name), &mut jobs)?;
        }
    }
    Ok(jobs)
}

fn walk_dir(dir: &Path, dest_dir: &Path, jobs: &mut Vec<Job>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            // sub-folder names are kept as-is; only leaf files get prefixed.
            walk_dir(&entry.path(), &dest_dir.join(entry.file_name()), jobs)?;
        } else if file_type.is_file() {
            jobs.push(Job {
                source: entry.path(),
                dest: dest_dir.join(prefixed(&entry.file_name())),
            });
        }
        // Symlinks and other special files are skipped on purpose.
    }
    Ok(())
}

/// Run a single job: read the source, XOR it with `key`, and write the result
/// to `job.dest`, creating parent directories as needed. Returns the path that
/// was written.
pub fn run_job(job: &Job, key: u8) -> io::Result<PathBuf> {
    let mut data = fs::read(&job.source)?;
    xor_in_place(&mut data, key);

    if let Some(parent) = job.dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&job.dest, &data)?;
    Ok(job.dest.clone())
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
    fn collect_puts_prefixed_files_in_sibling_folder() {
        let tmp = std::env::temp_dir().join(format!("dexor-test-collect-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(tmp.join("q/sub")).unwrap();
        fs::write(tmp.join("loose.bin"), b"x").unwrap();
        fs::write(tmp.join("q/a.exe"), b"x").unwrap();
        fs::write(tmp.join("q/sub/b.dll"), b"x").unwrap();

        let mut jobs = collect_jobs(&[tmp.join("loose.bin"), tmp.join("q")]).unwrap();
        jobs.sort_by(|a, b| a.dest.cmp(&b.dest));

        // output lands in a sibling `dexor-decoded/` folder, leaf names prefixed,
        // sub-folder names untouched.
        let dests: Vec<_> = jobs.iter().map(|j| j.dest.clone()).collect();
        assert_eq!(
            dests,
            vec![
                tmp.join("dexor-decoded/dexor_loose.bin"),
                tmp.join("dexor-decoded/q/dexor_a.exe"),
                tmp.join("dexor-decoded/q/sub/dexor_b.dll"),
            ]
        );
        let _ = fs::remove_dir_all(&tmp);
    }
}
