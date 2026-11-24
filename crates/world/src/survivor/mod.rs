mod catalog;
mod record;

use crate::{BlockMedium, CompanionPresence, CraftBlueprint, SurfaceFind};
pub use self::catalog::{RelicCatalog, RELIC_SLOT_COUNT};
pub use self::record::{SurvivorRecord, SurvivorWelcome};

/// Authoritative per-survivor inventory totals (`0..=33`) and equipped `(right, left)` palm codes (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SatchelManifest {
    totals: [u32; RELIC_SLOT_COUNT],
    hands: (u8, u8),
}

impl SatchelManifest {
    const CRANE_REPLENISH: [(u8, u32); 10] = [
        (19, 1), (27, 1), (16, 1), (25, 6), (26, 4), (15, 1), (14, 1), (18, 8), (17, 2), (20, 2),
    ];

    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(totals: [u32; RELIC_SLOT_COUNT], hands: (u8, u8)) -> Self {
        Self { totals, hands }
    }

    pub const fn empty() -> Self {
        Self::new([0; RELIC_SLOT_COUNT], (0, 0))
    }

    /// Server starter kit matching `TestKitSatchel` plus a right-hand `CantileverCrane`.
    pub fn starter_kit() -> Self {
        let mut totals = [0u32; RELIC_SLOT_COUNT];
        totals[13] = 1;
        for (code, qty) in Self::CRANE_REPLENISH {
            totals[code as usize] = qty;
        }
        Self::new(totals, (13, 0))
    }

    /// Parses server `[inventory] items`, stripping admin `SpawnStone`s (`code 29`) from guest starter kits.
    pub fn parse_server_kit(raw: &str, source: &str) -> Self {
        let trimmed = raw.trim().trim_start_matches('[').trim_end_matches(']').trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") || trimmed.eq_ignore_ascii_case("empty") {
            return Self::empty();
        }
        let mut totals = [0u32; RELIC_SLOT_COUNT];
        totals[13] = 1;
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
            let code = RelicCatalog::parse_token(name_part).unwrap_or_else(|| {
                panic!("Invalid starting inventory item '{name_part}' in {source}: unknown item identifier.")
            });
            let count = match qty_part {
                Some(raw_qty) => {
                    let n = raw_qty.parse::<i64>().unwrap_or_else(|_| {
                        panic!("Invalid item count '{raw_qty}' for '{name_part}' in {source}: expected integer >= 1.")
                    });
                    assert!(n >= 1, "Invalid item count '{raw_qty}' for '{name_part}' in {source}: minimum count is 1.");
                    n as u32
                }
                None => 1,
            };
            if RelicCatalog::new(code).guest_permitted() {
                totals[code as usize] = totals[code as usize].saturating_add(count);
            }
        }
        Self::new(totals, (13, 0)).sanitized()
    }

    pub const fn count_of(&self, code: u8) -> u32 {
        if (code as usize) < RELIC_SLOT_COUNT { self.totals[code as usize] } else { 0 }
    }

    pub const fn held_codes(&self) -> (u8, u8) {
        self.hands
    }

    pub fn sanitized(mut self) -> Self {
        self.totals[0] = 0;
        self.totals[29] = 0;
        let r = if RelicCatalog::new(self.hands.0).guest_permitted() && self.totals[self.hands.0 as usize] >= 1 {
            self.hands.0
        } else {
            0
        };
        let needed_l = 1 + u32::from(r != 0 && r == self.hands.1);
        let l = if RelicCatalog::new(self.hands.1).guest_permitted() && self.totals[self.hands.1 as usize] >= needed_l {
            self.hands.1
        } else {
            0
        };
        self.hands = (r, l);
        self
    }

    pub fn credit_code(&mut self, code: u8, qty: u32) {
        if RelicCatalog::new(code).guest_permitted() && qty > 0 {
            self.totals[code as usize] = self.totals[code as usize].saturating_add(qty);
        }
    }

    pub fn credit_medium(&mut self, medium: BlockMedium) {
        let code = RelicCatalog::from_medium(medium);
        if code == 13 {
            for (part, qty) in Self::CRANE_REPLENISH {
                if self.totals[part as usize] == 0 {
                    self.totals[part as usize] = qty;
                }
            }
        }
        self.credit_code(code, 1);
    }

    pub fn credit_timber_felled(&mut self) {
        self.credit_code(10, 3);
    }

    pub fn credit_find(&mut self, find: SurfaceFind) {
        self.credit_code(RelicCatalog::from_find(find), 1);
    }

    pub fn deduct_code_if_present(&mut self, code: u8, qty: u32) {
        if code > 0 && (code as usize) < RELIC_SLOT_COUNT && qty > 0 {
            self.totals[code as usize] = self.totals[code as usize].saturating_sub(qty);
            *self = self.sanitized();
        }
    }

    pub fn deduct_medium_if_present(&mut self, medium: BlockMedium) {
        self.deduct_code_if_present(RelicCatalog::from_medium(medium), 1);
    }

    pub fn deduct_find_if_present(&mut self, find: SurfaceFind) {
        self.deduct_code_if_present(RelicCatalog::from_find(find), 1);
    }

    pub fn deduct_blueprint_if_present(&mut self, blueprint: &CraftBlueprint) {
        let mut counts = [0u32; RELIC_SLOT_COUNT];
        blueprint.tally_media(|m| {
            let c = RelicCatalog::from_medium(m) as usize;
            if c < RELIC_SLOT_COUNT {
                counts[c] = counts[c].saturating_add(1);
            }
        });
        for (c, qty) in counts.into_iter().enumerate() {
            if qty > 0 {
                self.deduct_code_if_present(c as u8, qty);
            }
        }
    }

    pub fn apply_recipe(&mut self, recipe_idx: u8) -> bool {
        let Some((product, batch_yield, ingredients)) = RelicCatalog::recipe_spec(recipe_idx) else {
            return false;
        };
        for (ing, needed) in ingredients {
            if needed > 0 && self.count_of(ing) < needed {
                return false;
            }
        }
        for (ing, needed) in ingredients {
            if needed > 0 {
                self.totals[ing as usize] -= needed;
            }
        }
        self.credit_code(product, batch_yield);
        *self = self.sanitized();
        if self.hands.0 == 0 && self.count_of(product) >= 1 + u32::from(self.hands.1 == product) {
            self.hands.0 = product;
        }
        true
    }

    pub fn consume_edible(&mut self, code: u8) -> bool {
        if code != 24 || self.totals[24] == 0 {
            return false;
        }
        self.totals[24] -= 1;
        *self = self.sanitized();
        true
    }

    pub fn reconcile_hands(&mut self, right: u8, left: u8) {
        self.hands = (right, left);
        *self = self.sanitized();
    }

    pub fn inventory_summary(&self) -> String {
        let mut entries = Vec::new();
        for code in 1..(RELIC_SLOT_COUNT as u8) {
            if !RelicCatalog::new(code).guest_permitted() {
                continue;
            }
            let qty = self.count_of(code);
            if qty > 0 {
                entries.push(format!("{}:{qty}", RelicCatalog::new(code).slug()));
            }
        }
        if entries.is_empty() {
            "empty".to_string()
        } else {
            entries.join(", ")
        }
    }

    pub fn reconcile_from_presence(&mut self, presence: CompanionPresence) {
        let (r, l) = presence.transfer(|_, _, _, attire| {
            attire.transfer(|g, _, _| {
                let rc = (g[5] & 0x0F) | (((g[4] >> 4) & 1) << 4) | (((g[4] >> 6) & 1) << 5);
                let lc = (g[5] >> 4) | (((g[4] >> 5) & 1) << 4) | (((g[4] >> 7) & 1) << 5);
                (rc, lc)
            })
        });
        self.reconcile_hands(r, l);
    }

    pub fn inscribe_sparse(&self, out: &mut Vec<u8>) {
        out.push(self.hands.0);
        out.push(self.hands.1);
        let non_zero = (1..RELIC_SLOT_COUNT)
            .filter(|&i| RelicCatalog::new(i as u8).guest_permitted() && self.totals[i] > 0)
            .count() as u8;
        out.push(non_zero);
        for code in 1..RELIC_SLOT_COUNT {
            if !RelicCatalog::new(code as u8).guest_permitted() {
                continue;
            }
            let qty = self.totals[code];
            if qty > 0 {
                out.push(code as u8);
                out.extend_from_slice(&qty.to_le_bytes());
            }
        }
    }

    pub fn decode_sparse(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 3 {
            return None;
        }
        let (r, l, count) = (bytes[0], bytes[1], bytes[2] as usize);
        if count >= RELIC_SLOT_COUNT || bytes.len() < 3 + count * 5 {
            return None;
        }
        let mut totals = [0u32; RELIC_SLOT_COUNT];
        let mut cursor = 3;
        for _ in 0..count {
            let code = bytes[cursor];
            let qty = u32::from_le_bytes(bytes[cursor + 1..cursor + 5].try_into().ok()?);
            if RelicCatalog::new(code).guest_permitted() {
                totals[code as usize] = totals[code as usize].saturating_add(qty);
            }
            cursor += 5;
        }
        Some((Self::new(totals, (r, l)).sanitized(), cursor))
    }
}
