//! Unpack .bun section from Bun standalone executables.
//!
//! Parses StandaloneModuleGraph format (trailer, Offsets, CompiledModuleGraphFile[])
//! and writes each module's name/contents/sourcemap/bytecode to an output directory.
//! See docs/bun/compile-principle.md and docs/bun/decompile-tooling.md.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use goblin::Object;

const BUN_TRAILER: &[u8] = b"\n---- Bun! ----\n"; // 16 bytes

/// Offsets struct at end of blob (Bun StandaloneModuleGraph.zig).
/// All fields little-endian. StringPointer = offset (u32) + length (u32).
#[repr(C)]
struct Offsets {
    byte_count: u64,           // usize on 64-bit
    modules_ptr_offset: u32,
    modules_ptr_length: u32,
    entry_point_id: u32,
    _compile_exec_argv_offset: u32,
    _compile_exec_argv_length: u32,
    flags: u32,
}

/// CompiledModuleGraphFile (minimal layout for name/contents/sourcemap/bytecode).
/// Schema.StringPointer = offset(u32) + length(u32). We use 36 bytes per module.
const MODULE_STRUCT_SIZE: usize = 4 * 8 + 4; // 4×StringPointer + encoding + loader + module_format + side

fn main() -> Result<()> {
    let args = Args::parse();
    let blob = if let Some(blob_path) = &args.blob {
        fs::read(blob_path).with_context(|| format!("read blob {:?}", blob_path))?
    } else {
        let exe_path = args.exe.as_deref().context("missing <EXE> when not using --blob")?;
        extract_bun_blob(exe_path)?
    };
    let out_dir = args
        .output
        .as_deref()
        .unwrap_or(Path::new("./unpacked"));
    unpack_blob(&blob, out_dir)?;
    println!("Unpacked to {}", out_dir.display());
    Ok(())
}

#[derive(Parser)]
#[command(name = "bun-unpack", about = "Unpack .bun section from Bun standalone executables")]
struct Args {
    /// Path to a Bun standalone executable (PE / ELF / Mach-O). Required unless --blob is set.
    exe: Option<PathBuf>,

    /// Parse a raw .bun blob file instead of an executable (e.g. previously extracted).
    #[arg(long)]
    blob: Option<PathBuf>,

    /// Output directory. Default: ./unpacked
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn extract_bun_blob(exe_path: &Path) -> Result<Vec<u8>> {
    let buf = fs::read(exe_path).with_context(|| format!("read exe {:?}", exe_path))?;
    match Object::parse(&buf)? {
        Object::PE(pe) => {
            for section in &pe.sections {
                let is_bun = section.name[..4] == b".bun"
                    && section.name[4..].iter().all(|&b| b == 0);
                if is_bun {
                    let start = section.pointer_to_raw_data as usize;
                    let size = section.size_of_raw_data as usize;
                    if start + size > buf.len() {
                        anyhow::bail!(".bun section out of bounds (start={}, size={}, file_len={})", start, size, buf.len());
                    }
                    return Ok(buf[start..start + size].to_vec());
                }
            }
            anyhow::bail!("no .bun section found in PE");
        }
        Object::Elf(elf) => {
            // Linux: last 8 bytes = module graph length (u64 LE), preceding bytes = blob
            if buf.len() < 8 {
                anyhow::bail!("ELF too small for trailing length");
            }
            let len_end = buf.len() - 8;
            let len = u64::from_le_bytes(buf[len_end..].try_into().unwrap()) as usize;
            if len == 0 || len_end < len {
                anyhow::bail!("invalid module graph length at file end (len={}, file_len={})", len, buf.len());
            }
            let start = len_end - len;
            Ok(buf[start..len_end].to_vec())
        }
        Object::Mach(mach) => {
            // Mach-O: look for __bun or similar; Bun injects into a segment. Use heuristic:
            // search for trailer in last 20MB and take from there
            const TRAILER: &[u8] = b"\n---- Bun! ----\n";
            let search_end = buf.len().min(20 * 1024 * 1024);
            let search_start = buf.len().saturating_sub(search_end);
            if let Some(pos) = buf[search_start..].windows(TRAILER.len()).position(|w| w == TRAILER) {
                let trailer_pos = search_start + pos;
                if trailer_pos < std::mem::size_of::<Offsets>() {
                    anyhow::bail!("trailer too close to start for Offsets");
                }
                let offsets_start = trailer_pos - std::mem::size_of::<Offsets>();
                let offsets_bytes = &buf[offsets_start..trailer_pos];
                let byte_count = u64::from_le_bytes(offsets_bytes[0..8].try_into().unwrap()) as usize;
                if byte_count > trailer_pos || byte_count == 0 {
                    anyhow::bail!("invalid byte_count in Mach-O blob");
                }
                let blob_start = trailer_pos - byte_count;
                return Ok(buf[blob_start..trailer_pos + TRAILER.len()].to_vec());
            }
            anyhow::bail!("no Bun trailer found in Mach-O (trailer search in last 20MB)");
        }
        _ => anyhow::bail!("unsupported executable format (use --blob with raw .bun)"),
    }
}

fn unpack_blob(blob: &[u8], out_dir: &Path) -> Result<()> {
    if blob.len() < BUN_TRAILER.len() + std::mem::size_of::<Offsets>() {
        anyhow::bail!("blob too small for trailer + Offsets");
    }
    let trailer_start = blob
        .len()
        .checked_sub(BUN_TRAILER.len())
        .unwrap();
    if &blob[trailer_start..] != BUN_TRAILER {
        anyhow::bail!("invalid trailer (expected \"\\n---- Bun! ----\\n\" at end)");
    }
    let offsets_start = trailer_start
        .checked_sub(std::mem::size_of::<Offsets>())
        .context("blob too small for Offsets")?;
    let off = unsafe {
        &*(blob[offsets_start..].as_ptr() as *const Offsets)
    };
    let byte_count = off.byte_count as usize;
    if byte_count > blob.len() || byte_count == 0 {
        anyhow::bail!("invalid Offsets.byte_count");
    }
    let modules_offset = off.modules_ptr_offset as usize;
    let modules_len = off.modules_ptr_length as usize;
    if modules_offset >= blob.len()
        || modules_len > blob.len()
        || modules_offset + modules_len > blob.len()
    {
        anyhow::bail!("invalid modules_ptr (offset={}, length={})", modules_offset, modules_len);
    }
    if modules_len % MODULE_STRUCT_SIZE != 0 {
        anyhow::bail!(
            "modules_ptr length {} not divisible by module struct size {}",
            modules_len,
            MODULE_STRUCT_SIZE
        );
    }
    let num_modules = modules_len / MODULE_STRUCT_SIZE;
    fs::create_dir_all(out_dir).with_context(|| format!("create out dir {:?}", out_dir))?;

    let mut manifest = Vec::new();
    manifest.push(format!(
        "entry_point_id={} (0-based index into modules)",
        off.entry_point_id
    ));
    manifest.push(format!("modules_count={}", num_modules));
    manifest.push(String::new());

    for i in 0..num_modules {
        let base = modules_offset + i * MODULE_STRUCT_SIZE;
        let name_o = u32::from_le_bytes(blob[base..base + 4].try_into().unwrap()) as usize;
        let name_l = u32::from_le_bytes(blob[base + 4..base + 8].try_into().unwrap()) as usize;
        let contents_o = u32::from_le_bytes(blob[base + 8..base + 12].try_into().unwrap()) as usize;
        let contents_l = u32::from_le_bytes(blob[base + 12..base + 16].try_into().unwrap()) as usize;
        let sourcemap_o = u32::from_le_bytes(blob[base + 16..base + 20].try_into().unwrap()) as usize;
        let sourcemap_l = u32::from_le_bytes(blob[base + 20..base + 24].try_into().unwrap()) as usize;
        let bytecode_o = u32::from_le_bytes(blob[base + 24..base + 28].try_into().unwrap()) as usize;
        let bytecode_l = u32::from_le_bytes(blob[base + 28..base + 32].try_into().unwrap()) as usize;

        let name = slice_at(blob, name_o, name_l).unwrap_or_default();
        let name_str = String::from_utf8_lossy(name);
        let rel_path = name_to_rel_path(&name_str);
        if rel_path.is_empty() {
            continue;
        }
        let full = out_dir.join(&rel_path);
        if let Some(p) = full.parent() {
            fs::create_dir_all(p).with_context(|| format!("create dir {:?}", p))?;
        }

        let mut has_any = false;
        if contents_l > 0 {
            if let Some(b) = slice_at(blob, contents_o, contents_l) {
                let p = full.as_path();
                fs::write(p, b).with_context(|| format!("write {:?}", p))?;
                has_any = true;
                manifest.push(format!("[{}] {} (contents {} bytes)", i, rel_path, b.len()));
            }
        }
        if sourcemap_l > 0 {
            if let Some(b) = slice_at(blob, sourcemap_o, sourcemap_l) {
                let map_rel = format!("{}.map", rel_path);
                let map_full = out_dir.join(&map_rel);
                if let Some(parent) = map_full.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&map_full, b).with_context(|| format!("write sourcemap {:?}", map_full))?;
                has_any = true;
                manifest.push(format!("  -> sourcemap {} ({} bytes)", map_rel, b.len()));
            }
        }
        if bytecode_l > 0 {
            if let Some(b) = slice_at(blob, bytecode_o, bytecode_l) {
                let jsc_rel = if rel_path.ends_with(".jsc") {
                    rel_path.clone()
                } else {
                    format!("{}.jsc", rel_path)
                };
                let jsc_full = out_dir.join(&jsc_rel);
                if let Some(parent) = jsc_full.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&jsc_full, b).with_context(|| format!("write bytecode {:?}", jsc_full))?;
                has_any = true;
                manifest.push(format!("  -> bytecode {} ({} bytes, JSC)", jsc_rel, b.len()));
            }
        }
        if !has_any {
            manifest.push(format!("[{}] {} (no contents/sourcemap/bytecode)", i, rel_path));
        }
    }

    fs::write(out_dir.join("manifest.txt"), manifest.join("\n"))
        .context("write manifest.txt")?;
    Ok(())
}

fn slice_at(blob: &[u8], offset: usize, length: usize) -> Option<&[u8]> {
    if length == 0 {
        return Some(&[]);
    }
    blob.get(offset..).and_then(|s| s.get(..length))
}

fn name_to_rel_path(name: &str) -> String {
    let s = name
        .trim_start_matches("file:///")
        .trim_start_matches("B:\\~BUN\\")
        .trim_start_matches("B:/~BUN/")
        .trim_start_matches("/$bunfs/");
    let s = s.replace('\\', "/");
    if s.contains("..") || s.starts_with('/') {
        return String::new();
    }
    if s.is_empty() {
        return String::new();
    }
    s
}
