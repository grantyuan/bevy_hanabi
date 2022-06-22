use bevy::{math::Vec3, render::render_resource::std430::AsStd430};
use bytemuck::{Pod, Zeroable};

/// the area to present particles
#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, AsStd430, Default)]
pub(crate) struct ParticleAppearArea {
    /// the area position , mesh box's center.
    pub position: [f32; 3],
    actived: i32, // if actived == -1i32, then disable current AppreaArea
    pub flow_direction: [f32; 3],
    pub flow_speed: f32,
    // / The appearArea space long,width,height
    // pub box_size: f32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppearAreaIndex(u32);

// impl From<(u32, u32, u32)> for Vec3 {
//     fn from(v: (u32, u32, u32)) -> Self {
//         let (x, y, z) = v;
//         Vec3::new(x, y, z)
//     }
// }

pub struct AppearArea {
    pub position: Vec3,
    pub flow_velocity: Vec3,
    pub size: Vec3,
}

/// area for particles to show
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Copy)]
pub struct AppearAreaInfo {
    /// appear area's position
    pub position: Vec3,
    // pub flow_velocity: Vec3,
    // pub box_size: Vec3,
    /// flow direction ,x y z TODO! remove it
    pub flow_direction: Vec3,
    // / flow speed //TODO remove it
    // pub flow_speed: f32,
    // pub direction: fk32,
    // pub space:shape::Box,
}

impl AppearAreaInfo {
    /// init with x,y,z
    pub fn new(
        position: Vec3,
        flow_direction: Vec3, /* flow_speed: f32  *//* size:f32 */
    ) -> Self {
        AppearAreaInfo {
            position,
            flow_direction,
            // flow_speed,
            // space:shape::Box::new()
        }
    }

    pub(crate) fn to_particle_appear_area(&self) -> ParticleAppearArea {
        ParticleAppearArea {
            position: self.position.to_array(),
            flow_direction: self.flow_direction.to_array(),
            flow_speed: self.flow_speed,
            ..Default::default()
        }
    }

    pub(crate) fn none() -> ParticleAppearArea {
        ParticleAppearArea {
            actived: -1i32,
            ..Default::default()
        }
    }
}

impl AppearAreaIndex {
    pub fn to_position(&self, shape: &D3Shape) -> (u32, u32, u32) {
        let x_y_dim = shape.x * shape.y;
        let z = self.0 / x_y_dim;
        let y_offset = self.0 - x_y_dim * z;
        let y = y_offset / shape.x;
        let x = y_offset % shape.x;
        (x, y, z)
    }

    pub fn to_array_f32(&self, shape: &D3Shape) -> [f32; 3] {
        let (x, y, z) = self.to_position(shape);
        [x as f32, y as f32, z as f32]
    }

    pub fn to_array_u32(&self, shape: &D3Shape) -> [u32; 3] {
        let (x, y, z) = self.to_position(shape);
        [x, y, z]
    }
}

impl From<u32> for AppearAreaIndex {
    fn from(val: u32) -> Self {
        AppearAreaIndex(val)
    }
}

impl From<AppearAreaIndex> for u32 {
    fn from(val: AppearAreaIndex) -> Self {
        val.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct D3Shape {
    x: u32,
    y: u32,
    z: u32,
}

impl From<(u32, u32, u32)> for D3Shape {
    fn from(val: (u32, u32, u32)) -> Self {
        D3Shape {
            x: val.0,
            y: val.1,
            z: val.2,
        }
    }
}

impl From<(usize, usize, usize)> for D3Shape {
    fn from(val: (usize, usize, usize)) -> Self {
        D3Shape {
            x: val.0 as _,
            y: val.1 as _,
            z: val.2 as _,
        }
    }
}

impl D3Shape {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
}

impl AppearAreaIndex {
    pub fn new(x: u32, y: u32, z: u32, shape: D3Shape) -> AppearAreaIndex {
        let index = z * (shape.x * shape.y) + y * shape.x + x;
        AppearAreaIndex(index)
    }
}

#[cfg(test)]
mod test {
    use crate::render::appear_area::AppearAreaIndex;

    use super::D3Shape;

    #[test]
    fn test_appear_new() {
        let values: [[[u32; 4]; 4]; 4] = [
            [
                [1, 2, 3, 4],
                [5, 6, 7, 8],
                [9, 10, 11, 12],
                [13, 14, 15, 16],
            ],
            [
                [17, 18, 19, 20],
                [21, 22, 23, 24],
                [25, 26, 27, 28],
                [29, 30, 31, 32],
            ],
            [
                [33, 34, 35, 36],
                [37, 38, 39, 40],
                [41, 42, 43, 44],
                [45, 46, 47, 48],
            ],
            [
                [49, 50, 51, 52],
                [53, 54, 55, 56],
                [57, 58, 59, 60],
                [61, 62, 63, 64],
            ],
        ];
        let shape: D3Shape = (4, 4, 4).into();
        let index: AppearAreaIndex = AppearAreaIndex::new(0, 0, 1, shape);
        assert_eq!(16u32, index.into());
        assert_eq!(index.to_position(&shape), (0, 0, 1));
        let i = index.to_position(&shape);
        let v = values[0][0][1];
        assert_eq!(values[i.0 as usize][i.1 as usize][i.2 as usize], v);

        let index: AppearAreaIndex = AppearAreaIndex::new(3, 2, 3, shape);
        // assert_eq!(16u32, index.into());
        dbg!(&index.0);
        assert_eq!(index.to_position(&shape), (3, 2, 3));
        let i = index.to_position(&shape);
        // assert_eq!(values[i.0][i.1][i.2], values[3][2][3]);
        let v = values[3][2][3];
        assert_eq!(values[i.0 as usize][i.1 as usize][i.2 as usize], v);

        let index: AppearAreaIndex = AppearAreaIndex::new(0, 2, 3, shape);
        // assert_eq!(16u32, index.into());
        dbg!(&index.0);
        assert_eq!(index.to_position(&shape), (0, 2, 3));
        let i = index.to_position(&shape);
        let v = values[0][2][3];
        assert_eq!(values[i.0 as usize][i.1 as usize][i.2 as usize], v);
    }
}
