
use ash::util::*;
use ash::vk;

use gpu_allocator::vulkan::*;
use gpu_allocator::MemoryLocation;

use std::mem::align_of;
use std::sync::{Arc, Mutex};

pub struct Buffer {
    device: Option<ash::Device>,
    buffer_size: Option<u64>,
    buffer: Option<vk::Buffer>,
    allocation: Option<Allocation>,
    allocator: Option<Arc<Mutex<Allocator>>>,
}

impl Buffer {
    pub fn new(
        device: ash::Device,
        buffer_size: u64,
        buffer_usage_flags: vk::BufferUsageFlags,
        allocator: Arc<Mutex<Allocator>>,
        memory_location: MemoryLocation,
    ) -> Self {
        let buffer_create_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(buffer_usage_flags)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = unsafe {
            device.create_buffer(&buffer_create_info, None).unwrap()
        };

        let buffer_memory_req = unsafe {
            device.get_buffer_memory_requirements(buffer)
        };

        let allocation = allocator
            .lock().unwrap()
            .allocate(&AllocationCreateDesc {
                name: "buffer memory allocation",
                requirements: buffer_memory_req,
                location: memory_location,
                linear: true, // Buffers are always linear
            }).unwrap();

        unsafe {
            device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();
        }

        Self {
            device: Some(device),
            buffer_size: Some(buffer_size),
            buffer: Some(buffer),
            allocation: Some(allocation),
            allocator: Some(allocator),
        }
    }

    pub fn set_data<T>(&mut self, data: &Vec<T>) 
    where
        T: std::marker::Copy
    {
        let allocation = self.allocation.as_ref().unwrap();

        unsafe {
            let ptr = allocation.mapped_ptr().unwrap().as_ptr();

            let mut vert_align = Align::new(
                ptr,
                align_of::<T>() as u64,
                allocation.size(),
            );

            vert_align.copy_from_slice(data.as_slice());
        }
    }

    pub fn transform_from(&mut self, queue: &vk::Queue, command_buffer: &vk::CommandBuffer, src: &Self) {
        let device = self.device.as_ref()
            .expect("Could not get `device`.");

        let src_buffer = src.buffer.as_ref()
            .expect("Could not get `src_buffer`.");
        
        let dst_buffer = self.buffer.as_ref()
            .expect("Could not get `dst_buffer`.");

        let buffer_size = self.buffer_size.as_ref().unwrap();

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        let copy_region = vk::BufferCopy::builder()
            .src_offset(0)
            .dst_offset(0)
            .size(*buffer_size);

        unsafe {
            device.begin_command_buffer(*command_buffer, &command_buffer_begin_info)
                .expect("Could not begin command buffer.");
            device.cmd_copy_buffer(*command_buffer, *src_buffer, *dst_buffer, &[copy_region.build()]);
            device.end_command_buffer(*command_buffer)
                .expect("Could not end command buffer.");

            let command_buffers = vec![*command_buffer];

            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&command_buffers);
        
            device.queue_submit(
                *queue,
                &[submit_info.build()],
                vk::Fence::null(),
            )
            .expect("queue submit failed.");

            device.device_wait_idle().unwrap();
        }
        
    }

    pub fn buffer(&self) -> &vk::Buffer {
        self.buffer.as_ref().unwrap()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let device = self.device.as_ref().unwrap();
        let allocator = self.allocator.take().unwrap();
        let allocation = self.allocation.take().unwrap();

        allocator.lock().unwrap().free(allocation).unwrap();

        unsafe {
            // device.free_memory(self.buffer_memory.unwrap(), None);
            device.destroy_buffer(self.buffer.unwrap(), None);
        }
        
    }
}