use glam::{IVec3, Vec3};

use crate::{BrickGrid, MaterialId};

#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    pub pos: IVec3,
    pub normal: IVec3,
    pub distance: f32,
    pub material: MaterialId,
}

// later rewrite into dda, for now its fine
#[derive(Debug, Clone, Copy)]
pub struct Raycast<'grid> {
    pub grid: &'grid BrickGrid,
}

impl<'grid> Raycast<'grid> {
    pub fn new(grid: &'grid BrickGrid) -> Self {
        Self { grid }
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RaycastHit> {
        let dir = direction.normalize();
        let step = 0.01;
        let mut distance = 0.0;
        let mut prev_pos = origin.floor().as_ivec3();

        while distance < max_distance {
            let p = origin + dir * distance;
            let pos = IVec3::new(p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);

            if pos != prev_pos {
                if let Some(material) = self.grid.get_voxel(pos) {
                    let normal = prev_pos - pos;
                    return Some(RaycastHit {
                        pos,
                        normal,
                        distance,
                        material,
                    });
                }
                prev_pos = pos;
            }

            distance += step;
        }

        None
    }
}
