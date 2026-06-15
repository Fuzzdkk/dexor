# DeXOR

A tiny GUI for XOR-decoding files. Some tools "encrypt" files by XOR-ing every
byte with a single key — which isn't really encryption, because XOR is its own
inverse. Run the same XOR again and you get the original file straight back.
That's all this does: drag files or folders in (or hit the browse buttons), set
the key, hit Decode.

## Why I made this

This is an IR thing. When you're working an incident you often need the actual
sample back *out* of an AV/EDR quarantine — to hash it, detonate it in a
sandbox, or drop it in a disassembler. The data's right there on disk, just
XOR'd with one byte. In the moment though you don't want to be googling the key
and writing python kungfu, and half the time you can't anyway:

- the EDR got killed on the endpoint (or is half-dead), so its own "restore from
  quarantine" button isn't an option,
- you're carving files out of a quarantine folder on a dead-disk image or a
  collected triage package, with no agent to ask,
- or you're just on some locked-down box where dropping a quick script isn't
  worth the hassle.

So instead: one portable binary, no install, no deps. Copy it on, drag the
quarantine folder in, get the files back. That's the whole point.

## Using it

1. Run it (`dexor` on Linux, `dexor.exe` on Windows).
2. Set the **XOR key** (single byte, in hex). Defaults to `77`, or pick a vendor
   from the preset dropdown.
3. Add your files: drag them onto the window, or use **Add files… / Add
   folder…**.
4. Hit **Decode**.

No output folder to pick — decoded files go into a new `dexor-decoded/` folder
created right next to each input, and every output file is named
`dexor_<original>` so it's obvious it came out of here. Folders keep their
structure (reproduced under `dexor-decoded/<foldername>/...`). The log at the
bottom shows every file written and anything that failed.

## Vendor presets

Some AV products "encrypt" their quarantine by XOR-ing the whole file with one
byte. For those, DeXOR decodes the file directly — there's a preset dropdown
next to the key box that just fills in the right byte:

| Vendor | Key |
|--------|-----|
| Cisco AMP / Secure Endpoint | `0x77` |
| Microsoft MSE / Antimalware | `0xFF` |
| SentinelOne | `0xFF` |
| Microsoft Defender (macOS) | `0x25` |
| VIPRE | `0x33` |

Heads up: plenty of other vendors are *not* a plain single-byte XOR, so DeXOR
won't fully decode them on its own — e.g. ESET NQF (a byte math + `0xA5`
transform), McAfee BUP (XOR then an OLE container), Symantec VBN (offset-based
records, `0x5A`/`0xA5`), Kaspersky (8-byte rolling key), and Microsoft Defender
on Windows / Malwarebytes / Panda (RC4 / Blowfish). For those, look at DeXRAY or
unquarantine-rs. Key list cross-checked against DeXRAY.

## Building it yourself

Linux:

```
cargo build --release
# -> target/release/dexor
```

Windows .exe, cross-compiled from Linux (needs mingw):

```
sudo pacman -S mingw-w64-gcc          # one time
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# -> target/x86_64-pc-windows-gnu/release/dexor.exe
```

Or just run `./build-all.sh` and it does both, dropping them in `dist/`.

## Does it actually work?

Yeah, and `cargo test` proves it. The one that matters is in `tests/e2e.rs`: it
grabs a real executable, XORs it like a "scrambled" file would be, runs it back
through the exact decode path the GUI uses, then checks the result is
byte-for-byte identical to the original *and* still actually runs. The rest are
smaller checks on the XOR and the folder-walking.

```
cargo test
```

## How it's laid out

- `src/lib.rs` — the actual logic (the XOR + walking folders). No GUI in here on
  purpose, so the tests can hit it directly without needing a display.
- `src/main.rs` — the egui window. Drag/drop, browse buttons, key, output
  folder, a log.
- `tests/e2e.rs` — the does-it-really-work test described above.

---

made by [Fuzzdkk](https://github.com/Fuzzdkk)
