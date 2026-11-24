use crate::Face;
use super::ModifiedSlices;

/// Multi-core `ClimateZone` shard coordinator keeping uninhabited dormant faces at `0 %` CPU (`<= 4` fields).
#[derive(Clone, Copy, Debug)]
pub struct ZoneShards {
    dt: f32,
    occupied_mask: u8,
}

impl ZoneShards {
    /// Primary constructor doing nothing except assigning arguments to fields.
    pub const fn new(dt: f32, occupied_mask: u8) -> Self {
        Self { dt, occupied_mask }
    }

    /// Secondary constructor waking all faces that have active surface or flora simulation.
    pub const fn autonomous(dt: f32) -> Self {
        Self::new(dt, 0)
    }

    fn face_awake(&self, idx: usize, face: &dyn Face) -> bool {
        let occupied = idx < 8 && ((self.occupied_mask >> idx) & 1) != 0;
        occupied || face.active_simulation()
    }

    pub fn awake_count(&self, faces: &[Box<dyn Face>]) -> usize {
        faces
            .iter()
            .enumerate()
            .filter(|&(idx, f)| self.face_awake(idx, f.as_ref()))
            .count()
    }

    pub fn step_faces(&self, faces: &mut [Box<dyn Face>], slices: &mut ModifiedSlices) -> bool {
        let len = faces.len();
        let mut awake_flags = [false; 8];
        let mut awake_total = 0usize;
        for (idx, face) in faces.iter().enumerate().take(8) {
            if self.face_awake(idx, face.as_ref()) {
                awake_flags[idx] = true;
                awake_total += 1;
            }
        }
        if awake_total == 0 {
            return false;
        }
        let mut outcomes = [(false, false); 8];
        if awake_total == 1 {
            for (idx, face) in faces.iter_mut().enumerate().take(8) {
                if awake_flags[idx] {
                    outcomes[idx] = (face.step_surface(self.dt), face.step_flora(self.dt));
                }
            }
        } else {
            let dt = self.dt;
            std::thread::scope(|scope| {
                let mut handles = Vec::with_capacity(awake_total);
                for (idx, face) in faces.iter_mut().enumerate().take(8) {
                    if awake_flags[idx] {
                        handles.push((
                            idx,
                            scope.spawn(move || (face.step_surface(dt), face.step_flora(dt))),
                        ));
                    }
                }
                for (idx, handle) in handles {
                    outcomes[idx] = handle.join().expect("ZoneShards worker panicked");
                }
            });
        }
        let mut changed = false;
        for (idx, &(surf, flora)) in outcomes.iter().enumerate().take(len.min(8)) {
            if surf {
                slices.record(idx);
                changed = true;
            }
            if flora {
                slices.record(len + idx);
                changed = true;
            }
        }
        changed
    }
}
