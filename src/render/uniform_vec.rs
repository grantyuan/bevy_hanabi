use bevy::render::{
    render_resource::{
        std140::{self, AsStd140, DynamicUniform, Std140},
        BindingResource, Buffer, BufferAddress, BufferBinding, BufferDescriptor, BufferUsages,
    },
    renderer::{RenderDevice, RenderQueue},
};
use std::{borrow::Cow, num::NonZeroU64};

pub struct NamedUniformVec<T: AsStd140> {
    values: Vec<T>,
    scratch: Vec<u8>,
    uniform_buffer: Option<Buffer>,
    capacity: usize,
    item_size: usize,
    label: Option<Cow<'static, str>>,
}

impl<T: AsStd140> Default for NamedUniformVec<T> {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            scratch: Vec::new(),
            uniform_buffer: None,
            capacity: 0,
            item_size: (T::std140_size_static() + <T as AsStd140>::Output::ALIGNMENT - 1)
                & !(<T as AsStd140>::Output::ALIGNMENT - 1),
            label: None,
        }
    }
}

impl<T: AsStd140> NamedUniformVec<T> {
    pub fn new(label: Cow<'static, str>) -> Self {
        Self {
            label: Some(label),
            ..Default::default()
        }
    }

    #[inline]
    pub fn uniform_buffer(&self) -> Option<&Buffer> {
        self.uniform_buffer.as_ref()
    }

    #[inline]
    pub fn binding(&self) -> Option<BindingResource> {
        Some(BindingResource::Buffer(BufferBinding {
            buffer: self.uniform_buffer()?,
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
            self.uniform_buffer = Some(device.create_buffer(&BufferDescriptor {
                label: self.label.as_deref(),
                size: size as BufferAddress,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
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
        if let Some(uniform_buffer) = &self.uniform_buffer {
            let range = 0..self.item_size * self.values.len();
            let mut writer = std140::Writer::new(&mut self.scratch[range.clone()]);
            writer.write(self.values.as_slice()).unwrap();
            queue.write_buffer(uniform_buffer, 0, &self.scratch[range]);
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn values(&self) -> &[T] {
        &self.values
    }
}
