use ash::{Device, vk::{PhysicalDeviceLimits, BufferCreateInfo, StructureType, BufferCreateFlags, SharingMode, BufferUsageFlags, MemoryPropertyFlags, Buffer, CommandPool, Queue, CommandBufferUsageFlags, Fence, SubmitInfo, BufferCopy, MemoryMapFlags}};
use cgmath::{Matrix4, SquareMatrix};

use crate::{allocator::{Allocator, BufferAndAllocation, MemoryMapData}, math::{UniformBuffer, Vertex}};

pub unsafe fn create_uniform_buffers(device : &Device, allocator : &mut Allocator, count : u32, device_limits : &PhysicalDeviceLimits) -> UniformBufferObject{
    let buffer_object_size = std::mem::size_of::<UniformBuffer>();
    let alignment_offset = device_limits.min_uniform_buffer_offset_alignment;
    let alignment_size_increase = alignment_offset - (buffer_object_size as u64 % alignment_offset);
    let true_buffer_size = buffer_object_size as u64 + alignment_size_increase;
    let total_size = true_buffer_size*(count as u64);
    let buffer_create_info = BufferCreateInfo{
        s_type : StructureType::BUFFER_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : BufferCreateFlags::empty(),
        p_queue_family_indices : std::ptr::null(),
        queue_family_index_count : 0,
        sharing_mode : SharingMode::EXCLUSIVE,
        size : total_size,
        usage : BufferUsageFlags::UNIFORM_BUFFER,
    };
    let buffer = device.create_buffer(&buffer_create_info, None).expect("Failed to create uniform buffer");
    let buffer = BufferAndAllocation::new(allocator, buffer, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT);
    let data_map = allocator.get_memory_map_data(&buffer.allocation);
    let uniform_buffer = [
        UniformBuffer{matrix:Matrix4::identity()}
    ];
    for i in 0..count{
        let data_ptr = device.map_memory(data_map.memory, data_map.offset+i as u64 * true_buffer_size, true_buffer_size, MemoryMapFlags::empty()).expect("Failed to map uniform buffer") as *mut UniformBuffer;
        data_ptr.copy_from(uniform_buffer.as_ptr(), std::mem::size_of::<UniformBuffer>());
        device.unmap_memory(data_map.memory);
    }
    
    return UniformBufferObject{
        buffer,data_map,size:true_buffer_size,
    };
}
pub unsafe fn create_staging_buffer(device : &Device, allocator : &mut Allocator, size : u64) -> BufferAndAllocation{
    let buffer_create_info = BufferCreateInfo{
        s_type : StructureType::BUFFER_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : BufferCreateFlags::empty(),
        p_queue_family_indices : std::ptr::null(),
        queue_family_index_count : 0,
        sharing_mode : SharingMode::EXCLUSIVE,
        size,
        usage : BufferUsageFlags::TRANSFER_SRC,
    };
    let buffer = device.create_buffer(&buffer_create_info, None).expect("Failed to create staging buffer");
    return BufferAndAllocation::new(allocator, buffer, MemoryPropertyFlags::HOST_VISIBLE | MemoryPropertyFlags::HOST_COHERENT);
}
pub unsafe fn copy_buffer_regions(device : &Device, src : Buffer, dst : Buffer, command_pool : CommandPool, queue : Queue, regions : &[BufferCopy]){
    let command_buffer = super::command::create_command_buffers(device,command_pool,1,false);
    super::command::begin_primary_command_buffers(device, &command_buffer, CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    device.cmd_copy_buffer(command_buffer[0], src, dst, regions);
    super::command::end_command_buffers(device, &command_buffer);
    let submits = [
        SubmitInfo{
            s_type : StructureType::SUBMIT_INFO,
            p_next : std::ptr::null(),
            command_buffer_count : 1,
            p_command_buffers : command_buffer.as_ptr(),
            p_signal_semaphores : std::ptr::null(),
            p_wait_dst_stage_mask : std::ptr::null(),
            p_wait_semaphores : std::ptr::null(),
            signal_semaphore_count : 0,
            wait_semaphore_count : 0,
        }
    ];
    device.queue_submit(queue, &submits, Fence::null()).expect("Failed to submit command buffers");
    device.queue_wait_idle(queue).expect("Failed to wait for queue");
    device.free_command_buffers(command_pool, &command_buffer);
}
pub unsafe fn create_vertex_buffer<T>(device : &Device, allocator : &mut Allocator, command_pool : CommandPool, queue : Queue, vertices : Vec<T>) -> BufferAndAllocation{
    let data_size = vertices.len() * std::mem::size_of::<T>();
    let staging_buffer = create_staging_buffer(device, allocator, data_size as u64);
    let map_data = allocator.get_memory_map_data(&staging_buffer.allocation);
    let data_ptr = device.map_memory(map_data.memory, map_data.offset, map_data.size, MemoryMapFlags::empty()).expect("Failed to map staging data") as *mut T;
    data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
    device.unmap_memory(map_data.memory);
    let vertex_buffer_create_info = BufferCreateInfo{
        s_type : StructureType::BUFFER_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : BufferCreateFlags::empty(),
        p_queue_family_indices : std::ptr::null(),
        queue_family_index_count : 0,
        sharing_mode : SharingMode::EXCLUSIVE,
        size : (vertices.len() * std::mem::size_of::<T>()) as u64,
        usage : BufferUsageFlags::TRANSFER_DST | BufferUsageFlags::VERTEX_BUFFER,
    };
    let vertex_buffer = device.create_buffer(&vertex_buffer_create_info, None).expect("Failed to create vertex buffer");
    let vertex_buffer = BufferAndAllocation::new(allocator, vertex_buffer, MemoryPropertyFlags::DEVICE_LOCAL);
    let buffer_copies = [
        BufferCopy{
            size : (vertices.len() * std::mem::size_of::<T>()) as u64,
            src_offset : 0,
            dst_offset : 0,
        }
    ];
    copy_buffer_regions(device, staging_buffer.buffer, vertex_buffer.buffer, command_pool, queue, &buffer_copies);
    staging_buffer.destroy(allocator);
    return vertex_buffer;
}
pub unsafe fn copy_vertices_to_gpu(device : &Device, allocator : &mut Allocator, command_pool : CommandPool, queue : Queue, vertices : Vec<Vertex>, vertex_buffer : &BufferAndAllocation){
    let staging_buffer = create_staging_buffer(device, allocator, (vertices.len() * std::mem::size_of::<Vertex>()) as u64);
    let map_data = allocator.get_memory_map_data(&staging_buffer.allocation);
    let data_ptr = device.map_memory(map_data.memory, map_data.offset, map_data.size, MemoryMapFlags::empty()).expect("Failed to map staging data") as *mut Vertex;
    data_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len() * std::mem::size_of::<Vertex>());
    device.unmap_memory(map_data.memory);
    let buffer_copies = [
        BufferCopy{
            size : (vertices.len() * std::mem::size_of::<Vertex>()) as u64,
            src_offset : 0,
            dst_offset : 0,
        }
    ];
    copy_buffer_regions(device, staging_buffer.buffer, vertex_buffer.buffer, command_pool, queue, &buffer_copies);
    staging_buffer.destroy(allocator);
}
pub struct UniformBufferObject{
    pub buffer : BufferAndAllocation,
    data_map : MemoryMapData,
    size : u64,
}
impl UniformBufferObject{
    pub unsafe fn update_uniform_buffer(&self, object : UniformBuffer, image : u32, device : &Device){
        let data_ptr = device.map_memory(self.data_map.memory, self.data_map.offset+image as u64 * self.size, self.size, MemoryMapFlags::empty()).expect("Failed to map uniform buffer") as *mut UniformBuffer;
        data_ptr.copy_from_nonoverlapping([object].as_ptr(), 1);
        device.unmap_memory(self.data_map.memory);
    }
}