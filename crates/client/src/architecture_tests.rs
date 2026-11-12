use std::path::PathBuf;

/// Polymorphic architectural audit verifying Elegant Objects invariants without static helpers.
trait CodebaseAudit {
    fn verify(&self);
}

/// Immutable source-file audit enforcing 300-line ceiling, 4-field encapsulation, and local `impl` co-location.
struct SourceFileAudit {
    path: PathBuf,
}

impl SourceFileAudit {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn verify_local_impl(&self, content: &str, trimmed: &str) {
        if (trimmed.starts_with("impl ") || trimmed.starts_with("impl<"))
            && !trimmed.contains(" for ")
            && trimmed.ends_with('{')
        {
            let after_impl = trimmed.trim_start_matches("impl");
            let after_gen = after_impl
                .strip_prefix('<')
                .and_then(|s| s.split_once('>'))
                .map_or(after_impl, |(_, r)| r);
            let name = after_gen.trim().trim_end_matches('{').trim().split('<').next().unwrap_or("").trim();
            let declared = name.is_empty()
                || content.contains(&format!("struct {name}"))
                || content.contains(&format!("enum {name}"));
            assert!(declared, "{} splits inherent impl for `{name}` across files", self.path.display());
        }
    }
}

impl CodebaseAudit for SourceFileAudit {
    fn verify(&self) {
        let content = std::fs::read_to_string(&self.path).expect("read_to_string failed");
        let line_count = content.lines().count();
        assert!(line_count <= 300, "{} exceeds 300 lines ({} lines)", self.path.display(), line_count);
        assert!(
            !content.contains(concat!("#[allow(", "dead_code)]")),
            "{} contains dead_code suppression",
            self.path.display()
        );
        let (mut in_struct, mut struct_header, mut field_count) = (false, String::new(), 0usize);
        for line in content.lines() {
            let clean = line.split("//").next().unwrap_or("");
            self.verify_local_impl(&content, clean.trim());
            if !in_struct && clean.contains("struct ") && clean.contains('{') {
                let after_brace = clean.split('{').nth(1).unwrap_or("");
                if clean.contains('}') {
                    let inside = after_brace.split('}').next().unwrap_or("");
                    let c = inside.replace("::", "").matches(':').count();
                    assert!(c <= 4, "{} inline struct exceeds 4 fields ({}): {}", self.path.display(), c, clean.trim());
                    continue;
                }
                in_struct = true;
                struct_header = clean.trim().to_string();
                field_count = after_brace.replace("::", "").matches(':').count();
            } else if in_struct {
                if clean.contains('}') {
                    field_count += clean.split('}').next().unwrap_or("").replace("::", "").matches(':').count();
                    assert!(field_count <= 4, "{} exceeds 4 fields ({}): {}", self.path.display(), field_count, struct_header);
                    in_struct = false;
                } else {
                    field_count += clean.replace("::", "").matches(':').count();
                }
            }
        }
    }
}

/// Immutable directory-tree audit composing `SourceFileAudit` and nested `SourceTreeAudit` instances.
struct SourceTreeAudit {
    root: PathBuf,
}

impl SourceTreeAudit {
    /// Primary constructor doing nothing except assigning arguments to fields.
    const fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl CodebaseAudit for SourceTreeAudit {
    fn verify(&self) {
        for entry in std::fs::read_dir(&self.root).expect("read_dir failed").flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::new(path).verify();
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                SourceFileAudit::new(path).verify();
            }
        }
    }
}

#[test]
fn test_workspace_architecture_guard_enforces_line_and_field_limits() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let crates_dir = manifest_dir.parent().expect("crates dir");
    for subdir in ["world/src", "engine/src", "client/src"] {
        SourceTreeAudit::new(crates_dir.join(subdir)).verify();
    }
}
