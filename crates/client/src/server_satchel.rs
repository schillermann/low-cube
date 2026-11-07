use crate::attire::{BeltSatchel, HeldRelic};
use crate::command_flags::CommandFlags;
use crate::toml_document::TomlDocument;

/// Interface representing the configurable starting inventory stowed in the player's satchel at spawn.
pub trait StartingSatchel: Send + Sync {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel;
}

/// Baseline test kit starting satchel stowing the field camera and vehicle test parts.
pub struct TestKitSatchel;

impl TestKitSatchel {
    /// Primary constructor initializing the baseline test kit satchel.
    pub fn new() -> Self {
        Self
    }
}

impl StartingSatchel for TestKitSatchel {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        satchel.with_test_engines()
    }
}

/// Validated starting satchel parsed from a comma-separated list of `item` or `item:count` entries.
pub struct ParsedSatchel {
    items: Vec<(HeldRelic, u32)>,
}

impl ParsedSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(items: Vec<(HeldRelic, u32)>) -> Self {
        Self { items }
    }

    /// Secondary constructor parsing and fail-fast validating a comma-separated starting inventory list.
    pub fn parse(raw: &str, source: &str) -> Self {
        let trimmed = raw
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .trim();
        if trimmed.is_empty()
            || trimmed.eq_ignore_ascii_case("none")
            || trimmed.eq_ignore_ascii_case("empty")
        {
            return Self::new(Vec::new());
        }
        let mut parsed_items = Vec::new();
        for entry in trimmed.split(',') {
            let token = entry.trim().trim_matches('"').trim_matches('\'').trim();
            if token.is_empty() {
                continue;
            }
            let (name_part, qty_part) = match token.split_once(':') {
                Some((n, q)) => (n.trim(), Some(q.trim())),
                None => (token, None),
            };
            if name_part.eq_ignore_ascii_case("none") || name_part.eq_ignore_ascii_case("empty") {
                continue;
            }
            let relic = HeldRelic::EmptyPalm.parse_token(name_part).unwrap_or_else(|| {
                panic!("Invalid starting inventory item '{name_part}' in {source}: unknown item identifier.")
            });
            let count = match qty_part {
                Some(raw_qty) => {
                    let n = raw_qty.parse::<i64>().unwrap_or_else(|_| {
                        panic!("Invalid item count '{raw_qty}' for '{name_part}' in {source}: expected integer >= 1.")
                    });
                    if n < 1 {
                        panic!("Invalid item count '{raw_qty}' for '{name_part}' in {source}: minimum count is 1.");
                    }
                    n as u32
                }
                None => 1,
            };
            parsed_items.push((relic, count));
        }
        Self::new(parsed_items)
    }
}

impl StartingSatchel for ParsedSatchel {
    fn provision(&self, mut satchel: BeltSatchel) -> BeltSatchel {
        for &(relic, count) in &self.items {
            satchel = satchel.with_stowed_relic(relic, count);
        }
        satchel
    }
}

/// Configuration file-driven starting satchel decorator reading `[inventory] items` from `server.toml`.
pub struct FileSatchel {
    inner: Box<dyn StartingSatchel>,
}

impl FileSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn StartingSatchel>) -> Self {
        Self { inner }
    }

    /// Secondary constructor targeting the `server.toml` configuration file.
    pub fn server(fallback: Box<dyn StartingSatchel>) -> Self {
        Self::from_path("server.toml", fallback)
    }

    /// Secondary constructor loading starting inventory from an explicit configuration file path.
    pub fn from_path(path: &str, fallback: Box<dyn StartingSatchel>) -> Self {
        let Some(doc) = TomlDocument::from_file(path) else {
            return Self::new(fallback);
        };
        let raw = doc
            .section_property("inventory", "items", false)
            .or_else(|| doc.section_property("inventory", "satchel", false));
        if let Some(val) = raw {
            Self::new(Box::new(ParsedSatchel::parse(&val, path)))
        } else {
            Self::new(fallback)
        }
    }
}

impl StartingSatchel for FileSatchel {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        self.inner.provision(satchel)
    }
}

/// Process environment variable-driven starting satchel decorator reading `LOW_CUBE_INVENTORY`.
pub struct EnvironmentSatchel {
    inner: Box<dyn StartingSatchel>,
}

impl EnvironmentSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn StartingSatchel>) -> Self {
        Self { inner }
    }

    /// Secondary constructor reading `LOW_CUBE_INVENTORY` or `LOW_CUBE_ITEMS` from the process environment.
    pub fn process(fallback: Box<dyn StartingSatchel>) -> Self {
        if let Ok(raw) = std::env::var("LOW_CUBE_INVENTORY").or_else(|_| std::env::var("LOW_CUBE_ITEMS")) {
            Self::new(Box::new(ParsedSatchel::parse(&raw, "LOW_CUBE_INVENTORY")))
        } else {
            Self::new(fallback)
        }
    }
}

impl StartingSatchel for EnvironmentSatchel {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        self.inner.provision(satchel)
    }
}

/// Command-line argument-driven starting satchel decorator reading `--inventory` or `--items`.
pub struct ArgumentSatchel {
    inner: Box<dyn StartingSatchel>,
}

impl ArgumentSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn StartingSatchel>) -> Self {
        Self { inner }
    }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process(fallback: Box<dyn StartingSatchel>) -> Self {
        let args: Vec<String> = std::env::args().collect();
        Self::from_arguments(&args, fallback)
    }

    /// Secondary constructor parsing an explicit arguments slice.
    pub fn from_arguments(args: &[String], fallback: Box<dyn StartingSatchel>) -> Self {
        let flags = CommandFlags::from_slice(args);
        if let Some(raw) = flags.value_of("--inventory").or_else(|| flags.value_of("--items")) {
            Self::new(Box::new(ParsedSatchel::parse(&raw, "--inventory")))
        } else {
            Self::new(fallback)
        }
    }
}

impl StartingSatchel for ArgumentSatchel {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        self.inner.provision(satchel)
    }
}

/// Server launch starting satchel decorator coordinating CLI arguments, environment variables, `server.toml`, and test kit baseline.
pub struct ServerSatchel {
    inner: Box<dyn StartingSatchel>,
}

impl ServerSatchel {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(inner: Box<dyn StartingSatchel>) -> Self {
        Self { inner }
    }

    /// Secondary constructor assembling the full decorator chain in priority order.
    pub fn assembled() -> Self {
        Self::new(Box::new(ArgumentSatchel::process(Box::new(
            EnvironmentSatchel::process(Box::new(FileSatchel::server(Box::new(TestKitSatchel::new())))),
        ))))
    }
}

impl StartingSatchel for ServerSatchel {
    fn provision(&self, satchel: BeltSatchel) -> BeltSatchel {
        self.inner.provision(satchel)
    }
}
