use ash::{Device, vk::{DescriptorSetLayout, DescriptorSetLayoutCreateInfo, StructureType, DescriptorSetLayoutCreateFlags, DescriptorPool, DescriptorPoolCreateInfo, DescriptorPoolCreateFlags, DescriptorPoolSize, DescriptorType, DescriptorSet, DescriptorSetAllocateInfo, DescriptorBufferInfo, PhysicalDeviceLimits, Buffer, WriteDescriptorSet}};

use crate::math::UniformBuffer;

pub unsafe fn create_descriptor_set_layout(device : &Device) -> DescriptorSetLayout{
    let descriptor_set_layout_binding = UniformBuffer::get_bindings();
    let descriptor_layout_create_info = DescriptorSetLayoutCreateInfo{
        s_type : StructureType::DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DescriptorSetLayoutCreateFlags::empty(),
        binding_count : descriptor_set_layout_binding.len() as u32,
        p_bindings : descriptor_set_layout_binding.as_ptr(),
    };
    return device.create_descriptor_set_layout(&descriptor_layout_create_info, None).expect("Failed to create descriptor set layouts");
}
pub unsafe fn create_descriptor_pool(device : &Device, count : u32) -> DescriptorPool{
    let pool_size = [
        DescriptorPoolSize{
            descriptor_count : count,
            ty : DescriptorType::UNIFORM_BUFFER,
        }
    ];
    let descriptor_pool_create_info = DescriptorPoolCreateInfo{
        s_type : StructureType::DESCRIPTOR_POOL_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DescriptorPoolCreateFlags::empty(),
        max_sets : count,
        p_pool_sizes : pool_size.as_ptr(),
        pool_size_count : pool_size.len() as u32,
    };
    return device.create_descriptor_pool(&descriptor_pool_create_info, None).expect("Failed to create descriptor pool");
}
pub unsafe fn create_descriptor_sets(device : &Device, layout : DescriptorSetLayout, descriptor_pool : DescriptorPool, count : u32, uniform_buffer : Buffer ,device_limits : &PhysicalDeviceLimits) -> Vec<DescriptorSet>{
    let layouts = vec!(layout;count as usize);
    let descriptor_set_allocate_info = DescriptorSetAllocateInfo{
        s_type : StructureType::DESCRIPTOR_SET_ALLOCATE_INFO,
        p_next : std::ptr::null(),
        descriptor_pool,
        descriptor_set_count : count,
        p_set_layouts : layouts.as_ptr(),
    };
    let descriptor_sets = device.allocate_descriptor_sets(&descriptor_set_allocate_info).expect("Failed to allocate descriptor sets");
    let mut descriptor_writes = vec!();
    for (i,&descriptor_set) in descriptor_sets.iter().enumerate(){
        let buffer_object_size = std::mem::size_of::<UniformBuffer>();
        let alignment_offset = device_limits.min_uniform_buffer_offset_alignment;
        let alignment_size_increase = alignment_offset - (buffer_object_size as u64 % alignment_offset);
        let true_buffer_size = buffer_object_size as u64 + alignment_size_increase;
        let buffer_info = DescriptorBufferInfo{
            buffer: uniform_buffer,
            offset : true_buffer_size*i as u64,
            range : buffer_object_size as u64,
        };
        descriptor_writes.push(WriteDescriptorSet{
            s_type : StructureType::WRITE_DESCRIPTOR_SET,
            p_next : std::ptr::null(),
            dst_set : descriptor_set,
            descriptor_count : 1,
            descriptor_type : DescriptorType::UNIFORM_BUFFER,
            dst_array_element : 0,
            dst_binding : 0,
            p_buffer_info : &buffer_info,
            p_image_info : std::ptr::null(),
            p_texel_buffer_view : std::ptr::null(),
        });
    }
    device.update_descriptor_sets(&descriptor_writes, &[]);
    return descriptor_sets;
}