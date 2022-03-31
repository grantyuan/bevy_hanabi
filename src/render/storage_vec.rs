use bevy::render::{
    render_resource::{
        std430::{self, AsStd430, Std430},
        BindingResource, Buffer, BufferAddress, BufferBinding, BufferDescriptor, BufferUsages,
    },
    renderer::{RenderDevice, RenderQueue},
};
use std::{borrow::Cow, num::NonZeroU64, ops::Range};

pub struct StorageVec<T: AsStd430> {
    values: Vec<T>,
    scratch: Vec<u8>,
    storage_buffer: Option<Buffer>,
    capacity: usize,
    item_size: usize,
    label: Option<Cow<'static, str>>,
}

impl<T: AsStd430> Default for StorageVec<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            scratch: Vec::new(),
            storage_buffer: None,
            capacity: 0,
            item_size: (T::std430_size_static() + <T as AsStd430>::Output::ALIGNMENT - 1)
                & !(<T as AsStd430>::Output::ALIGNMENT - 1),
            label: None,
        }
    }
}

impl<T: AsStd430> StorageVec<T> {
    pub fn new(label: Cow<'static, str>) -> Self {
        Self {
            label: Some(label),
            ..Default::default()
        }
    }

    #[inline]
    pub fn storage_buffer(&self) -> Option<&Buffer> {
        self.storage_buffer.as_ref()
    }

    #[inline]
    pub fn binding(&self) -> Option<BindingResource> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.storage_buffer()?,
            offset: 0,
            size: Some(NonZeroU64::new(self.item_size as u64).unwrap()),
        }))
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn push(&mut self, value: T) -> usize {
        let index = self.values.len();
        self.values.push(value);
        index
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.values[index]
    }

    pub fn reserve(&mut self, capacity: usize, device: &RenderDevice) -> bool {
        if capacity > self.capacity {
            self.capacity = capacity;
            let size = self.item_size * capacity;
            self.scratch.resize(size, 0);
            self.storage_buffer = Some(device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                size: size as BufferAddress,
                usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
                mapped_at_creation: false,
            }));
            true
        } else {
            false
        }
    }

    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        if self.values.is_empty() {
            return;
        }
        self.reserve(self.values.len(), device);
        if let Some(storage_buffer) = &self.storage_buffer {
            let range = 0..self.item_size * self.values.len();
            let mut writer = std430::Writer::new(&mut self.scratch[range.clone()]);
            writer.write(self.values.as_slice()).unwrap();
            queue.write_buffer(storage_buffer, 0, &self.scratch[range]);
        }
    }

    pub fn write_slice(&mut self, slice: Range<u32>, device: &RenderDevice, queue: &RenderQueue) {
        if self.values.is_empty() {
            return;
        }
        self.reserve(self.values.len(), device);
        if let Some(storage_buffer) = &self.storage_buffer {
            let range = slice.start as usize * self.item_size..slice.end as usize * self.item_size;
            let mut writer = std430::Writer::new(&mut self.scratch[range.clone()]);
            writer
                .write(&self.values[slice.start as usize..slice.end as usize])
                .unwrap();
            queue.write_buffer(
                storage_buffer,
                range.start as BufferAddress,
                &self.scratch[range],
            );
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }

    pub fn values_mut(&mut self) -> &mut [T] {
        &mut self.values
    }
}
