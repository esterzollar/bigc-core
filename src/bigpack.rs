use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const MAGIC: &[u8] = b"BIGPAK";
const VERSION: u8 = 1;
const DEFAULT_KEY: &str = "BIGC_POWER_05_DEFAULT_KEY";

pub struct BigPack;

impl BigPack {
    pub fn pack(source_folder: &str, output_file: &str, key_opt: Option<String>) {
        let key = key_opt.unwrap_or_else(|| DEFAULT_KEY.to_string());
        println!(
            "BigPack: Packing [{}] -> [{}]...",
            source_folder, output_file
        );

        let mut file_map: HashMap<String, (u64, u64)> = HashMap::new();
        let mut raw_data: Vec<u8> = Vec::new();
        let source_path = Path::new(source_folder);

        if !source_path.exists() {
            println!("BigPack Error: Source folder does not exist.");
            return;
        }

        let mut files = Vec::new();
        if let Err(e) = Self::visit_dirs(source_path, &mut files) {
            println!("BigPack Error: Failed to scan: {}", e);
            return;
        }

        for path in files {
            let relative_path = path
                .strip_prefix(source_path)
                .unwrap()
                .to_string_lossy()
                .replace("\\", "/");
            let mut f = File::open(&path).expect("BigPack: Open Failed ");
            let mut buffer = Vec::new();
            if f.read_to_end(&mut buffer).is_err() {
                println!("BigPack Warning: Skipping file [{}]", relative_path);
                continue;
            }

            let offset = raw_data.len() as u64;
            let size = buffer.len() as u64;

            file_map.insert(relative_path, (offset, size));
            raw_data.extend_from_slice(&buffer);
        }

        let index_json = serde_json::to_string(&file_map).unwrap();
        let index_bytes = Self::stream_cipher(index_json.as_bytes(), &key);
        let index_len = index_bytes.len() as u64;

        let mut out = File::create(output_file).expect("BigPack: Create Failed ");
        let _ = out.write_all(MAGIC);
        let _ = out.write_all(&[VERSION]);
        let _ = out.write_all(&index_len.to_le_bytes());
        let _ = out.write_all(&index_bytes);
        let _ = out.write_all(&raw_data);

        println!("BigPack: Success! Packed {} items.", file_map.len());
    }

    pub fn unpack(archive_file: &str, key_opt: Option<String>) {
        let key = key_opt.unwrap_or_else(|| DEFAULT_KEY.to_string());
        println!("BigPack: Unpacking [{}]...", archive_file);

        let mut f = File::open(archive_file).expect("BigPack: Open Failed ");

        let mut magic = [0u8; 6];
        if f.read_exact(&mut magic).is_err() || magic != MAGIC {
            println!("BigPack Error: Invalid signature.");
            return;
        }

        let mut ver = [0u8; 1];
        let _ = f.read_exact(&mut ver);
        if ver[0] != VERSION {
            println!("BigPack Error: Version mismatch.");
            return;
        }

        let mut len_bytes = [0u8; 8];
        let _ = f.read_exact(&mut len_bytes);
        let index_len = u64::from_le_bytes(len_bytes);

        let mut index_raw = vec![0u8; index_len as usize];
        let _ = f.read_exact(&mut index_raw);

        let index_decoded = Self::stream_cipher(&index_raw, &key);
        let index_str = match String::from_utf8(index_decoded) {
            Ok(s) => s,
            Err(_) => {
                println!("BigPack Error: Decryption failed (Wrong Key?)");
                return;
            }
        };

        let file_map: HashMap<String, (u64, u64)> = match serde_json::from_str(&index_str) {
            Ok(m) => m,
            Err(_) => {
                println!("BigPack Error: Corrupt Index ");
                return;
            }
        };

        let data_start = 6 + 1 + 8 + index_len;

        for (path_str, (offset, size)) in file_map {
            println!("BigPack: Extracting -> {}", path_str);

            let p = Path::new(&path_str);
            if let Some(parent) = p.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let _ = f.seek(SeekFrom::Start(data_start + offset));
            let mut file_buf = vec![0u8; size as usize];
            let _ = f.read_exact(&mut file_buf);

            let mut out = File::create(p).expect("BigPack: Write Failed ");
            let _ = out.write_all(&file_buf);
        }
        println!("BigPack: Done.");
    }

    pub fn fetch(archive_file: &str, target_file: &str) -> Option<Vec<u8>> {
        let key = DEFAULT_KEY;

        let mut f = File::open(archive_file).ok()?;
        let _ = f.seek(SeekFrom::Start(7));

        let mut len_bytes = [0u8; 8];
        let _ = f.read_exact(&mut len_bytes);
        let index_len = u64::from_le_bytes(len_bytes);

        let mut index_raw = vec![0u8; index_len as usize];
        let _ = f.read_exact(&mut index_raw);
        let index_decoded = Self::stream_cipher(&index_raw, key);

        if let Ok(index_str) = String::from_utf8(index_decoded) {
            if let Ok(file_map) = serde_json::from_str::<HashMap<String, (u64, u64)>>(&index_str) {
                let lookup = target_file.replace("\\", "/");
                if let Some((offset, size)) = file_map.get(&lookup) {
                    let data_start = 7 + 8 + index_len;
                    let _ = f.seek(SeekFrom::Start(data_start + offset));
                    let mut buffer = vec![0u8; *size as usize];
                    if f.read_exact(&mut buffer).is_ok() {
                        return Some(buffer);
                    }
                }
            }
        }
        None
    }

    fn visit_dirs(dir: &Path, cb: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    Self::visit_dirs(&path, cb)?;
                } else {
                    cb.push(path);
                }
            }
        }
        Ok(())
    }

    fn stream_cipher(data: &[u8], key: &str) -> Vec<u8> {
        let mut rng = BigRng::new(key);
        let mut out = Vec::with_capacity(data.len());
        for b in data {
            out.push(b ^ rng.next_byte());
        }
        out
    }
}

struct BigRng {
    state: u64,
}
impl BigRng {
    fn new(key: &str) -> Self {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in key.bytes() {
            h = (h ^ b as u64).wrapping_mul(0x100000001b3);
        }
        Self { state: h }
    }
    fn next_byte(&mut self) -> u8 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state ^ (self.state >> 18)) >> 27) as u8
    }
}
