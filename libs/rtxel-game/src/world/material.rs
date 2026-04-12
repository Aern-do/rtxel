use std::mem;

use bevy_ecs::resource::Resource;
use encase::ShaderType;
use glam::Vec3;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Material {
    pub color: Vec3,
}

impl Material {
    pub fn new(color: Vec3) -> Self {
        Self { color, }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialId(usize);

impl MaterialId {
    pub const AIR: MaterialId = MaterialId(0);

    pub fn id(&self) -> usize {
        self.0
    }
}

impl Default for MaterialId {
    fn default() -> Self {
        Self::AIR
    }
}

#[derive(Debug, Default, Clone, Resource)]
pub struct MaterialManager {
    pub materials: Vec<Material>,
    pub is_dirty: bool,
}

impl MaterialManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, material: Material) -> MaterialId {
        self.is_dirty = true;
        let idx = self.materials.len();
        self.materials.push(material);

        MaterialId(idx + 1)
    }

    pub fn take_dirty(&mut self) -> bool {
        mem::replace(&mut self.is_dirty, false)
    }
}
