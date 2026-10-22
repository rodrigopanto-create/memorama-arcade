use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct HighScore {
    pub name: String,
    pub score: i32,
}

pub struct HighScores {
    pub entries: Vec<HighScore>,
    pub writable: bool,
}

fn normalize(entries: &mut Vec<HighScore>) -> Result<(), String> {
    if entries.iter().any(|entry| {
        entry.name.len() != 3
            || !entry.name.bytes().all(|ch| ch.is_ascii_uppercase())
            || entry.score < 0
    }) {
        return Err(
            "JSON de records invalido: se requieren 3 letras A-Z y puntajes no negativos.".into(),
        );
    }
    entries.sort_by(|a, b| b.score.cmp(&a.score));
    entries.truncate(10);
    Ok(())
}

impl HighScores {
    pub fn defaults(writable: bool) -> Self {
        Self {
            entries: [("RET", 2500), ("ARC", 2000), ("CPU", 1500)]
                .into_iter()
                .map(|(name, score)| HighScore {
                    name: name.into(),
                    score,
                })
                .collect(),
            writable,
        }
    }

    pub fn load() -> Result<Self, String> {
        match storage::read()? {
            None => Ok(Self::defaults(true)),
            Some(json) => {
                let mut entries = serde_json::from_slice(&json)
                    .map_err(|err| format!("No se pudo leer el JSON de records: {err}"))?;
                normalize(&mut entries)?;
                Ok(Self {
                    entries,
                    writable: true,
                })
            }
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        if !self.writable {
            return Err(
                "Guardado desactivado: corrige los datos guardados y reinicia el juego.".into(),
            );
        }
        normalize(&mut self.entries)?;
        let json = serde_json::to_vec_pretty(&self.entries)
            .map_err(|err| format!("No se pudo serializar la tabla: {err}"))?;
        storage::write(&json)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod storage {
    use std::{fs, io::ErrorKind, path::Path};

    pub fn read() -> Result<Option<Vec<u8>>, String> {
        read_file(Path::new("highscores.json"))
    }

    fn read_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
            Err(err) => Err(format!("No se pudo leer highscores.json: {err}")),
        }
    }

    pub fn write(json: &[u8]) -> Result<(), String> {
        write_file(Path::new("highscores.json"), json)
    }

    fn write_file(path: &Path, json: &[u8]) -> Result<(), String> {
        let temporary = path.with_extension("json.tmp");
        fs::write(&temporary, json)
            .and_then(|_| fs::rename(&temporary, path))
            .map_err(|err| format!("No se pudo guardar highscores.json: {err}"))
    }

    #[test]
    fn persists_json_and_replaces_previous_table() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "memorama-highscores-{}-{unique}.json",
            std::process::id()
        ));
        assert_eq!(read_file(&path).unwrap(), None);
        let first = br#"[{"name":"ABC","score":9000}]"#;
        let second = br#"[{"name":"XYZ","score":9999}]"#;
        write_file(&path, first).unwrap();
        assert_eq!(read_file(&path).unwrap().unwrap(), first);
        write_file(&path, second).unwrap();
        assert_eq!(read_file(&path).unwrap().unwrap(), second);
        assert!(!path.with_extension("json.tmp").exists());
        fs::remove_file(path).unwrap();
    }
}

#[cfg(target_arch = "wasm32")]
mod storage {
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn highscores_size() -> i32;
        fn highscores_read(ptr: *mut u8, len: usize) -> i32;
        fn highscores_write(ptr: *const u8, len: usize) -> i32;
    }

    pub fn read() -> Result<Option<Vec<u8>>, String> {
        // El plugin JavaScript copia JSON UTF-8 entre localStorage y la memoria WASM.
        let size = unsafe { highscores_size() };
        match size {
            -1 => Err("No se pudo acceder a los records del navegador.".into()),
            0 => Ok(None),
            size if size > 0 => {
                let mut bytes = vec![0; size as usize];
                if unsafe { highscores_read(bytes.as_mut_ptr(), bytes.len()) } != size {
                    return Err("No se pudo leer la tabla del navegador.".into());
                }
                Ok(Some(bytes))
            }
            _ => Err("Tamano de records invalido.".into()),
        }
    }

    pub fn write(json: &[u8]) -> Result<(), String> {
        if unsafe { highscores_write(json.as_ptr(), json.len()) } == 0 {
            Ok(())
        } else {
            Err("No se pudo guardar la tabla en el navegador.".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trip() {
        let scores = HighScores::defaults(true).entries;
        let json = serde_json::to_string(&scores).unwrap();
        assert_eq!(
            serde_json::from_str::<Vec<HighScore>>(&json).unwrap(),
            scores
        );
    }

    #[test]
    fn sorts_and_limits_to_ten() {
        let mut scores = (0..15)
            .map(|score| HighScore {
                name: "AAA".into(),
                score,
            })
            .collect();
        normalize(&mut scores).unwrap();
        assert_eq!(scores.len(), 10);
        assert_eq!(scores[0].score, 14);
        assert_eq!(scores[9].score, 5);
    }

    #[test]
    fn rejects_invalid_entries() {
        for (name, score) in [
            ("", 1),
            ("AB", 1),
            ("ABCD", 1),
            ("abc", 1),
            ("A1C", 1),
            ("AAA", -1),
        ] {
            assert!(normalize(&mut vec![HighScore {
                name: name.into(),
                score
            }])
            .is_err());
        }
        assert!(serde_json::from_str::<Vec<HighScore>>("not json").is_err());
    }

    #[test]
    fn disabled_storage_does_not_write() {
        assert!(HighScores::defaults(false).save().is_err());
    }
}
