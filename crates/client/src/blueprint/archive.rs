use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use super::sheet::{BlueprintSheet, BlueprintSlug};
use super::SavedBlueprint;

/// Focused persistent storage contract for named player vehicle blueprints.
pub trait BlueprintArchive {
    fn load_saved(&self) -> Vec<SavedBlueprint>;
    fn store(&mut self, entry: &SavedBlueprint);
    fn expunge(&mut self, title: &str);
}

/// Directory-backed blueprint storage persisting each named vehicle as `blueprints/<slug>.toml`.
pub struct DirectoryBlueprints {
    directory: PathBuf,
}

impl DirectoryBlueprints {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(directory: PathBuf) -> Self {
        Self { directory }
    }

    pub fn in_directory(path: &str) -> Self {
        Self::new(PathBuf::from(path))
    }

    /// Secondary constructor targeting the local `blueprints/` directory.
    pub fn profile() -> Self {
        Self::in_directory("blueprints")
    }

    fn toml_paths(&self) -> Vec<PathBuf> {
        let entries = match fs::read_dir(&self.directory) {
            Ok(dir) => dir,
            Err(err) if err.kind() == ErrorKind::NotFound => return Vec::new(),
            Err(err) => panic!("Failed to read blueprints directory {:?}: {err}", self.directory),
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .and_then(|s| s.to_str())
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
            })
            .collect();
        paths.sort();
        paths
    }

    fn fresh_path(&self, title: &str) -> PathBuf {
        let stem = BlueprintSlug::new(title).stem();
        let first = self.directory.join(format!("{stem}.toml"));
        if !first.exists() {
            return first;
        }
        let mut suffix = 2usize;
        loop {
            let candidate = self.directory.join(format!("{stem}_{suffix}.toml"));
            if !candidate.exists() {
                return candidate;
            }
            suffix += 1;
        }
    }
}

impl BlueprintArchive for DirectoryBlueprints {
    fn load_saved(&self) -> Vec<SavedBlueprint> {
        let mut loaded: Vec<SavedBlueprint> = Vec::new();
        for path in self.toml_paths() {
            let Some(entry) = BlueprintSheet::from_file(&path).and_then(|s| s.decode_saved()) else {
                continue;
            };
            if let Some(slot) = loaded
                .iter_mut()
                .find(|e| e.title_text().eq_ignore_ascii_case(entry.title_text()))
            {
                *slot = entry;
            } else {
                loaded.push(entry);
            }
        }
        loaded.sort_by(|a, b| {
            a.title_text()
                .to_ascii_uppercase()
                .cmp(&b.title_text().to_ascii_uppercase())
        });
        loaded
    }

    fn store(&mut self, entry: &SavedBlueprint) {
        fs::create_dir_all(&self.directory).expect("Failed to create blueprints directory");
        let matching: Vec<PathBuf> = self
            .toml_paths()
            .into_iter()
            .filter(|path| {
                BlueprintSheet::from_file(path)
                    .and_then(|s| s.decode_saved())
                    .is_some_and(|bp| bp.title_text().eq_ignore_ascii_case(entry.title_text()))
            })
            .collect();
        let target = matching
            .first()
            .cloned()
            .unwrap_or_else(|| self.fresh_path(entry.title_text()));
        BlueprintSheet::from_saved(entry).write_to(&target);
        for extra in matching.iter().skip(1) {
            fs::remove_file(extra).expect("Failed to remove duplicate blueprint TOML file");
        }
    }

    fn expunge(&mut self, title: &str) {
        for path in self.toml_paths() {
            let matches = BlueprintSheet::from_file(&path)
                .and_then(|s| s.decode_saved())
                .is_some_and(|bp| bp.title_text().eq_ignore_ascii_case(title));
            if matches {
                fs::remove_file(&path).expect("Failed to remove blueprint TOML file");
            }
        }
    }
}

/// Isolated in-memory `BlueprintArchive` Fake Object for deterministic offline unit tests.
#[cfg(test)]
pub struct MemoryBlueprints {
    entries: Vec<SavedBlueprint>,
}

#[cfg(test)]
impl MemoryBlueprints {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(entries: Vec<SavedBlueprint>) -> Self {
        Self { entries }
    }

    pub const fn empty() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
impl BlueprintArchive for MemoryBlueprints {
    fn load_saved(&self) -> Vec<SavedBlueprint> {
        let mut sorted = self.entries.clone();
        sorted.sort_by(|a, b| {
            a.title_text()
                .to_ascii_uppercase()
                .cmp(&b.title_text().to_ascii_uppercase())
        });
        sorted
    }

    fn store(&mut self, entry: &SavedBlueprint) {
        if let Some(slot) = self
            .entries
            .iter_mut()
            .find(|e| e.title_text().eq_ignore_ascii_case(entry.title_text()))
        {
            *slot = entry.clone();
        } else {
            self.entries.push(entry.clone());
        }
    }

    fn expunge(&mut self, title: &str) {
        self.entries
            .retain(|e| !e.title_text().eq_ignore_ascii_case(title));
    }
}
