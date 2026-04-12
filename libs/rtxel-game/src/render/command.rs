
#[derive(Debug, Clone)]
pub enum UnpackCommand {
    Clear {
        grid_idx: u32,
    },
    SetUniform {
        grid_idx: u32,
        map_idx: u32,
        material_idx: u32,
        mask: [u32; 16],
    },
    SetPallete {
        grid_idx: u32,
        map_idx: u32,
        pallete_idx: u32,
        mask: [u32; 16],
        pallete: [u32; 512],
    },
}

#[derive(Debug)]
pub struct UnpackCommandEncoder {
    data: Vec<u32>,
    offsets: Vec<u32>,
}

impl UnpackCommandEncoder {
    pub const OP_CLEAR: u32 = 0;
    pub const OP_SET_UNIFORM: u32 = 1;
    pub const OP_SET_PALLETE: u32 = 2;

    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            offsets: Vec::new(),
        }
    }

    pub fn finish(self) -> (Vec<u32>, Vec<u32>) {
        (self.data, self.offsets)
    }

    pub fn write_command(&mut self, command: UnpackCommand) {
        self.offsets.push(self.data.len() as u32);

        match command {
            UnpackCommand::Clear { grid_idx } => {
                self.data.push(Self::OP_CLEAR);
                self.data.push(grid_idx);
            }
            UnpackCommand::SetUniform {
                grid_idx,
                map_idx,
                material_idx,
                mask,
            } => {
                self.data.push(Self::OP_SET_UNIFORM);
                self.data.push(grid_idx);
                self.data.push(map_idx);
                self.data.push(material_idx);
                self.data.extend_from_slice(&mask);
            }
            UnpackCommand::SetPallete {
                grid_idx,
                map_idx,
                pallete_idx,
                mask,
                pallete,
            } => {
                self.data.push(Self::OP_SET_PALLETE);
                self.data.push(grid_idx);
                self.data.push(map_idx);
                self.data.push(pallete_idx);
                self.data.extend_from_slice(&mask);
                self.data.extend_from_slice(&pallete);
            }
        }
    }
}
