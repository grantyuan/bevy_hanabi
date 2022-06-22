use bevy::{
    math::Vec3,
    render::render_resource::{std140::AsStd140, BufferVec}, prelude::shape,
};
use bytemuck::{Pod, Zeroable};
#[allow(unsafe_code)]
use ndarray::parallel::prelude::{IntoParallelIterator, ParallelIterator};
use ron::value;

use crate::{AppearArea, AppearAreaIndex, AppearAreaInfo};

struct BoxInfo {
    /// the size of x, y, z
    uvw_size: Vec3,
    center_position: Vec3,
    index: AppearAreaIndex,
    ref_appear_area: Option<AppearAreaInfo>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, AsStd140)]
pub(crate) struct MeshBox {
    pub uvw_size: [f32; 3],
    pub center_position: [f32; 3],
}

pub struct MeshBoxes {
    values: Vec<BoxInfo>,
    // buffers: BufferVec<MeshBox>,
    // all vertexs
    // vertexs: Vec<[Vec3; 8]>,
    shape: (usize, usize, usize),
}

impl MeshBoxes {
    pub fn new(shape: (usize, usize, usize)) -> Self {
        let all_cells: Vec<BoxInfo> = (0..shape.0 * shape.1 * shape.2)
            .into_par_iter()
            .map(|_m| BoxInfo {
                uvw_size: Vec3::new(1.0, 1.0, 1.0),
                center_position: Vec3::from_slice(
                    &AppearAreaIndex::from(_m as u32).to_array_f32(&shape.into()),
                ),

                index: AppearAreaIndex::from(_m as u32),
                ref_appear_area: None,
            })
            .collect();
        MeshBoxes {
            values: all_cells,
            shape,
        }
    }

    pub fn insert_mesh_box(&self, appear_area: AppearArea) -> Vec<AppearArea> {
        let result = Vec::new();
        let [x, y, z] = appear_area.position.to_array();
        let appear_index = AppearAreaIndex::new(x as u32, y as u32, z as u32, self.shape.into());
        let index: usize = u32::from(appear_index) as usize;
        let item = self.values.get_mut(index).unwrap();

        item.uvw_size = appear_area.size;
        item.center_position = appear_area.position;
        item.ref_appear_area = Some(AppearAreaInfo {
            position: appear_area.position,
            flow_direction: appear_area.flow_velocity,
        });

        result
    }

    pub(crate) fn to_render_array(&self) -> &[BoxInfo] {
        self.values.as_slice()
    }

    // pub (crate ) fn fill_buffer(&self, buffer: &Buff) ->
    pub(crate) fn get_non_empty_mapping_indexs(&self) -> Vec<u32> {
        let result: Vec<u32> = self
            .values
            .into_iter()
            .enumerate()
            .filter(|(i, v)| v.ref_appear_area.is_some())
            .map(|(i, v)| i as u32)
            .collect();
        result
    }

    pub fn get_vertexs(&self, index:(usize,usize,usize)) -> &[Vec3] {
        let (x,y,z) = index;
        let raw_index:u32 = AppearAreaIndex::new(x, y, z, self.shape.into()).into();
        let value  = &self.values[raw_index as usize];        
        let _box = shape::Box::new(value.uvw_size.x, value.uvw_size.y, value.uvw_size.z);
        
    }
}
