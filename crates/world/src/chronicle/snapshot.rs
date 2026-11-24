/// Width and height of a `SectorSnapshot` in `0.5 m` `GridCell` units (`8 × 8` cells = `4.0 m × 4.0 m`).
pub const SNAPSHOT_SECTOR_CELLS: i32 = 8;

/// Sentinel archetype code identifying an undercut natural crown shape (`240 % 16 == 0`).
pub const NATURAL_OVERHANG_STAMP: u8 = 0xF0;

/// Single placed or overhanging voxel block inside an `8 × 8` `SectorSnapshot` (`<= 4` fields).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotBlock {
    local_uv: u8,
    floor_quarters: i16,
    stamp: [u8; 3],
    verdant: bool,
}

impl SnapshotBlock {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(local_uv: u8, floor_quarters: i16, stamp: [u8; 3], verdant: bool) -> Self {
        Self { local_uv, floor_quarters, stamp, verdant }
    }

    pub const fn local_uv(self) -> u8 { self.local_uv }
    pub const fn floor_elevation(self) -> f32 { (self.floor_quarters as f32) * 0.25 }
    pub const fn stamp(self) -> [u8; 3] { self.stamp }
    pub const fn verdant(self) -> bool { self.verdant }
}

/// Deployed cantilever crane and its suspended craft cells inside an `8 × 8` `SectorSnapshot` (`<= 4` fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RigSnapshot {
    local_uv: u8,
    arm: (i16, i8, i8, u8),
    craft_cells: Vec<(i16, i16, i16, [u8; 3])>,
}

impl RigSnapshot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        local_uv: u8,
        arm: (i16, i8, i8, u8),
        craft_cells: Vec<(i16, i16, i16, [u8; 3])>,
    ) -> Self {
        Self { local_uv, arm, craft_cells }
    }

    pub const fn local_uv(&self) -> u8 { self.local_uv }
    pub const fn arm(&self) -> (i16, i8, i8, u8) { self.arm }
    pub fn craft_cells(&self) -> &[(i16, i16, i16, [u8; 3])] { &self.craft_cells }
}

/// Net cell-state contents of an `8 × 8` (`4 m × 4 m`) terrain sector (`<= 4` fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorColumnState {
    shafts: Vec<(u8, i16)>,
    strata: Vec<(u8, i16, u8)>,
    blocks: Vec<SnapshotBlock>,
    rigs: Vec<RigSnapshot>,
}

impl SectorColumnState {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        shafts: Vec<(u8, i16)>,
        strata: Vec<(u8, i16, u8)>,
        blocks: Vec<SnapshotBlock>,
        rigs: Vec<RigSnapshot>,
    ) -> Self {
        Self { shafts, strata, blocks, rigs }
    }

    pub const fn untouched() -> Self {
        Self::new(Vec::new(), Vec::new(), Vec::new(), Vec::new())
    }

    pub fn pristine(&self) -> bool {
        self.shafts.is_empty() && self.strata.is_empty() && self.blocks.is_empty() && self.rigs.is_empty()
    }

    pub fn shafts(&self) -> &[(u8, i16)] { &self.shafts }
    pub fn strata(&self) -> &[(u8, i16, u8)] { &self.strata }
    pub fn blocks(&self) -> &[SnapshotBlock] { &self.blocks }
    pub fn rigs(&self) -> &[RigSnapshot] { &self.rigs }
}

/// Raycast-free net cell snapshot for a single `(ClimateZone, SectorU, SectorV)` `4 m × 4 m` plot (`<= 4` fields).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SectorSnapshot {
    face: u8,
    sector: (i32, i32),
    bare_mask: u64,
    columns: SectorColumnState,
}

impl SectorSnapshot {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(
        face: u8,
        sector: (i32, i32),
        bare_mask: u64,
        columns: SectorColumnState,
    ) -> Self {
        Self { face, sector, bare_mask, columns }
    }

    pub const fn face(&self) -> u8 { self.face }
    pub const fn sector(&self) -> (i32, i32) { self.sector }
    pub const fn bare_mask(&self) -> u64 { self.bare_mask }
    pub const fn columns(&self) -> &SectorColumnState { &self.columns }
    pub fn pristine(&self) -> bool { self.bare_mask == 0 && self.columns.pristine() }

    pub const fn pack_uv(u: i32, v: i32, _gsu: i32, _gsv: i32) -> u8 {
        let du = (u.rem_euclid(SNAPSHOT_SECTOR_CELLS) & 7) as u8;
        let dv = (v.rem_euclid(SNAPSHOT_SECTOR_CELLS) & 7) as u8;
        du | (dv << 3)
    }

    pub const fn unpack_uv(local_uv: u8, gsu: i32, gsv: i32) -> (i32, i32) {
        let du = (local_uv & 0x07) as i32;
        let dv = ((local_uv >> 3) & 0x07) as i32;
        (gsu * SNAPSHOT_SECTOR_CELLS + du, gsv * SNAPSHOT_SECTOR_CELLS + dv)
    }

    pub fn inscribe(&self, out: &mut Vec<u8>) {
        out.push(self.face);
        out.extend_from_slice(&self.sector.0.to_le_bytes());
        out.extend_from_slice(&self.sector.1.to_le_bytes());
        out.extend_from_slice(&self.bare_mask.to_le_bytes());
        let shafts = &self.columns.shafts;
        out.push(shafts.len().min(64) as u8);
        for &(uv, delta) in shafts.iter().take(64) {
            out.push(uv & 0x3F);
            out.extend_from_slice(&delta.to_le_bytes());
        }
        let strata = &self.columns.strata;
        out.extend_from_slice(&(strata.len().min(512) as u16).to_le_bytes());
        for &(uv, step, med) in strata.iter().take(512) {
            out.push(uv & 0x3F);
            out.extend_from_slice(&step.to_le_bytes());
            out.push(med);
        }
        let blocks = &self.columns.blocks;
        out.extend_from_slice(&(blocks.len().min(512) as u16).to_le_bytes());
        for b in blocks.iter().take(512) {
            let uv_flag = (b.local_uv & 0x3F) | if b.verdant { 0x80 } else { 0 };
            out.push(uv_flag);
            out.extend_from_slice(&b.floor_quarters.to_le_bytes());
            out.extend_from_slice(&b.stamp);
        }
        let rigs = &self.columns.rigs;
        out.push(rigs.len().min(16) as u8);
        for r in rigs.iter().take(16) {
            out.push(r.local_uv & 0x3F);
            out.extend_from_slice(&r.arm.0.to_le_bytes());
            out.push(r.arm.1 as u8);
            out.push(r.arm.2 as u8);
            out.push(r.arm.3 & 3);
            let cells = r.craft_cells();
            out.extend_from_slice(&(cells.len().min(512) as u16).to_le_bytes());
            for &(du, dv, fq, st) in cells.iter().take(512) {
                out.extend_from_slice(&du.to_le_bytes());
                out.extend_from_slice(&dv.to_le_bytes());
                out.extend_from_slice(&fq.to_le_bytes());
                out.extend_from_slice(&st);
            }
        }
    }

    pub fn decode(bytes: &[u8]) -> Option<(Self, usize)> {
        if bytes.len() < 23 {
            return None;
        }
        let face = bytes[0];
        if face >= 6 {
            return None;
        }
        let gsu = i32::from_le_bytes(bytes[1..5].try_into().ok()?);
        let gsv = i32::from_le_bytes(bytes[5..9].try_into().ok()?);
        let bare_mask = u64::from_le_bytes(bytes[9..17].try_into().ok()?);
        let shaft_len = bytes[17] as usize;
        if shaft_len > 64 {
            return None;
        }
        let mut cursor = 18;
        if bytes.len() < cursor + shaft_len * 3 + 5 {
            return None;
        }
        let mut shafts = Vec::with_capacity(shaft_len);
        for _ in 0..shaft_len {
            let uv = bytes[cursor] & 0x3F;
            let delta = i16::from_le_bytes([bytes[cursor + 1], bytes[cursor + 2]]);
            shafts.push((uv, delta));
            cursor += 3;
        }
        let strata_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        if strata_len > 512 || bytes.len() < cursor + strata_len * 4 + 3 {
            return None;
        }
        let mut strata = Vec::with_capacity(strata_len);
        for _ in 0..strata_len {
            let uv = bytes[cursor] & 0x3F;
            let step = i16::from_le_bytes([bytes[cursor + 1], bytes[cursor + 2]]);
            let med = bytes[cursor + 3];
            strata.push((uv, step, med));
            cursor += 4;
        }
        let block_len = u16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]) as usize;
        cursor += 2;
        if block_len > 512 || bytes.len() < cursor + block_len * 6 + 1 {
            return None;
        }
        let mut blocks = Vec::with_capacity(block_len);
        for _ in 0..block_len {
            let raw = bytes[cursor];
            let fq = i16::from_le_bytes([bytes[cursor + 1], bytes[cursor + 2]]);
            let stamp = [bytes[cursor + 3], bytes[cursor + 4], bytes[cursor + 5]];
            blocks.push(SnapshotBlock::new(raw & 0x3F, fq, stamp, (raw & 0x80) != 0));
            cursor += 6;
        }
        let rig_len = *bytes.get(cursor)? as usize;
        cursor += 1;
        if rig_len > 16 {
            return None;
        }
        let mut rigs = Vec::with_capacity(rig_len);
        for _ in 0..rig_len {
            if bytes.len() < cursor + 8 {
                return None;
            }
            let uv = bytes[cursor] & 0x3F;
            let gq = i16::from_le_bytes([bytes[cursor + 1], bytes[cursor + 2]]);
            let arm = (gq, bytes[cursor + 3] as i8, bytes[cursor + 4] as i8, bytes[cursor + 5] & 3);
            let clen = u16::from_le_bytes([bytes[cursor + 6], bytes[cursor + 7]]) as usize;
            cursor += 8;
            if clen > 512 || bytes.len() < cursor + clen * 9 {
                return None;
            }
            let mut craft_cells = Vec::with_capacity(clen);
            for _ in 0..clen {
                let du = i16::from_le_bytes([bytes[cursor], bytes[cursor + 1]]);
                let dv = i16::from_le_bytes([bytes[cursor + 2], bytes[cursor + 3]]);
                let fq = i16::from_le_bytes([bytes[cursor + 4], bytes[cursor + 5]]);
                let st = [bytes[cursor + 6], bytes[cursor + 7], bytes[cursor + 8]];
                craft_cells.push((du, dv, fq, st));
                cursor += 9;
            }
            rigs.push(RigSnapshot::new(uv, arm, craft_cells));
        }
        let cols = SectorColumnState::new(shafts, strata, blocks, rigs);
        Some((Self::new(face, (gsu, gsv), bare_mask, cols), cursor))
    }
}
