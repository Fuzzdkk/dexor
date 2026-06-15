// the real test i actually care about: grab a real executable, scramble it with
// an xor key, run it back through my decode path, and check it's byte-identical
// AND still runs. if this passes i trust the tool.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use dexor::{collect_jobs, run_job, xor_in_place, DEFAULT_XOR_KEY};

fn workdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("dexor-e2e-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn decodes_a_runnable_executable() {
    // Pick a real, small executable present on this system.
    let original_exe = ["/bin/true", "/usr/bin/true", "/bin/echo"]
        .iter()
        .map(PathBuf::from)
        .find(|p| p.exists())
        .expect("need a sample executable to test against");

    let original_bytes = fs::read(&original_exe).unwrap();

    let dir = workdir("run");
    let scrambled_dir = dir.join("scrambled");
    fs::create_dir_all(&scrambled_dir).unwrap();

    // 1. Scramble: XOR the original with the key on disk.
    let mut q = original_bytes.clone();
    xor_in_place(&mut q, DEFAULT_XOR_KEY);
    let q_path = scrambled_dir.join("sample.bin");
    fs::write(&q_path, &q).unwrap();
    assert_ne!(q, original_bytes, "scrambled file must differ from original");

    // 2. Decode via the exact library path the GUI calls (queue the folder).
    //    Output should land in a sibling dexor-decoded/ folder, name prefixed.
    let jobs = collect_jobs(&[scrambled_dir.clone()]).unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        jobs[0].dest,
        dir.join("dexor-decoded/scrambled/dexor_sample.bin")
    );
    let written = run_job(&jobs[0], DEFAULT_XOR_KEY).unwrap();

    // 3. Byte-for-byte identical to the original.
    let decoded_bytes = fs::read(&written).unwrap();
    assert_eq!(
        decoded_bytes, original_bytes,
        "decoded bytes must match the original exactly"
    );

    // 4. Make it executable and actually run it — proves "runnable state".
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = fs::metadata(&written).unwrap().permissions();
        perm.set_mode(0o755);
        fs::set_permissions(&written, perm).unwrap();
    }
    let status = Command::new(&written)
        .status()
        .expect("decoded binary should be executable");
    assert!(status.success(), "decoded binary should run successfully");

    let _ = fs::remove_dir_all(&dir);
}
