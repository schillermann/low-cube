/// Encapsulated command-line argument sequence resolving flag options (`--flag value` or `--flag=value`).
#[derive(Clone, Debug)]
pub struct CommandFlags {
    arguments: Vec<String>,
}

impl CommandFlags {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub fn new(arguments: Vec<String>) -> Self {
        Self { arguments }
    }

    /// Secondary constructor capturing arguments from the active process environment.
    pub fn process() -> Self {
        Self::new(std::env::args().collect())
    }

    /// Secondary constructor copying from an explicit slice of argument strings.
    pub fn from_slice(arguments: &[String]) -> Self {
        Self::new(arguments.to_vec())
    }

    /// Checks whether a standalone boolean switch (e.g. `--tron` or `tron`) is present.
    pub fn contains_switch(&self, switch: &str) -> bool {
        self.arguments.iter().any(|arg| arg == switch)
    }

    /// Resolves the string value associated with `flag` (`--flag val` or `--flag=val`).
    pub fn value_of(&self, flag: &str) -> Option<String> {
        self.arguments
            .windows(2)
            .find(|pair| pair[0] == flag)
            .map(|pair| pair[1].clone())
            .or_else(|| {
                self.arguments
                    .iter()
                    .find(|arg| arg.starts_with(flag) && arg[flag.len()..].starts_with('='))
                    .and_then(|arg| arg.split_once('='))
                    .map(|(_, val)| val.to_string())
            })
    }

    /// Resolves and parses a floating-point value associated with `flag`.
    pub fn float_of(&self, flag: &str) -> Option<f32> {
        self.value_of(flag).and_then(|val| val.parse::<f32>().ok())
    }
}
