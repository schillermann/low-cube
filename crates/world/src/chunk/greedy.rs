use super::VOLUME_EDGE_LENGTH;

/// 2D slice buffer performing greedy quad merging on exposed coplanar voxel facets.
pub struct FacetSlice {
    materials: [u8; VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH],
    visited: [bool; VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH],
}

impl FacetSlice {
    /// Primary constructor initializing an empty unvisited facet slice.
    pub fn new() -> Self {
        Self {
            materials: [0u8; VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH],
            visited: [false; VOLUME_EDGE_LENGTH * VOLUME_EDGE_LENGTH],
        }
    }

    /// Records an exposed voxel face at 2D slice coordinates (u, v).
    pub fn record(&mut self, u: usize, v: usize, material_id: u8) {
        if u < VOLUME_EDGE_LENGTH && v < VOLUME_EDGE_LENGTH {
            self.materials[u + v * VOLUME_EDGE_LENGTH] = material_id;
        }
    }

    /// Emits merged rectangular regions as (u, v, width, height, material_id) tuples.
    pub fn merge_into(&mut self, consumer: &mut dyn FnMut(usize, usize, usize, usize, u8)) {
        let size = VOLUME_EDGE_LENGTH;
        for v in 0..size {
            for u in 0..size {
                let idx = u + v * size;
                let mat = self.materials[idx];
                if mat == 0 || self.visited[idx] {
                    continue;
                }

                // Compute maximal width along u
                let mut width = 1;
                while u + width < size {
                    let next_idx = (u + width) + v * size;
                    if self.visited[next_idx] || self.materials[next_idx] != mat {
                        break;
                    }
                    width += 1;
                }

                // Compute maximal height along v
                let mut height = 1;
                'expand_v: while v + height < size {
                    for row_u in u..(u + width) {
                        let check_idx = row_u + (v + height) * size;
                        if self.visited[check_idx] || self.materials[check_idx] != mat {
                            break 'expand_v;
                        }
                    }
                    height += 1;
                }

                // Mark visited
                for dv in 0..height {
                    for du in 0..width {
                        self.visited[(u + du) + (v + dv) * size] = true;
                    }
                }

                consumer(u, v, width, height, mat);
            }
        }
    }
}
