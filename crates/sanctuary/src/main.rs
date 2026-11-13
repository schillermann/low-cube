use std::env;
use sanctuary::{SanctuaryPlan, SanctuaryToml};

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    let toml = SanctuaryToml::from_file("server.toml");
    let plan = SanctuaryPlan::assembled(&raw_args, &toml);
    if let Err(err) = plan.execute() {
        panic!("Sanctuary startup failed: {err}");
    }
}
