use ash::{Device, vk::{CommandPoolCreateFlags, CommandPool, CommandPoolCreateInfo, StructureType, CommandBufferUsageFlags, CommandBuffer, CommandBufferAllocateInfo, CommandBufferLevel, CommandBufferBeginInfo, Pipeline, DescriptorSet, RenderPass, ClearColorValue, ClearValue, RenderPassBeginInfo, Framebuffer, Extent2D, ClearDepthStencilValue, Rect2D, Offset2D, SubpassContents, PipelineBindPoint, PipelineLayout}};

use crate::allocator::BufferAndAllocation;

pub unsafe fn create_command_pool(device : &Device, queue_family_index : u32) -> CommandPool{
    let command_pool_create_info = CommandPoolCreateInfo{
        s_type : StructureType::COMMAND_POOL_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : CommandPoolCreateFlags::empty(),
        queue_family_index,
    };
    return device.create_command_pool(&command_pool_create_info, None).expect("Failed to create command pool");
}
pub unsafe fn create_command_buffers(device : &Device, command_pool : CommandPool, count : u32, secondary : bool) -> Vec<CommandBuffer>{
    let allocate_info = CommandBufferAllocateInfo{
        s_type : StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next : std::ptr::null(),
        level : if secondary{CommandBufferLevel::SECONDARY}else{CommandBufferLevel::PRIMARY},
        command_buffer_count : count,
        command_pool,
    };
    return device.allocate_command_buffers(&allocate_info).expect("Failed to allocate command buffers");
}
pub unsafe fn begin_primary_command_buffers(device : &Device, buffers : &Vec<CommandBuffer>, flags : CommandBufferUsageFlags){
    let begin_info = CommandBufferBeginInfo{
        s_type : StructureType::COMMAND_BUFFER_BEGIN_INFO,
        p_next : std::ptr::null(),
        flags,
        p_inheritance_info : std::ptr::null(),
    };
    for &command_buffer in buffers.iter(){
        device.begin_command_buffer(command_buffer, &begin_info).expect("Failed to begin command buffers");
    }
}
pub unsafe fn end_command_buffers(device : &Device, buffers : &Vec<CommandBuffer>){
    for &command_buffer in buffers.iter(){
        device.end_command_buffer(command_buffer).expect("Failed to finish command buffer recording");
    }
}
pub unsafe fn begin_render_pass(device : &Device, command_buffers : &Vec<CommandBuffer>, render_pass : RenderPass, framebuffers : &Vec<Framebuffer>,extent : Extent2D, secondary : bool){
    for (i,&command_buffer) in command_buffers.iter().enumerate(){
        let clear_values = [
            ClearValue{color:ClearColorValue{float32:[1.0,1.0,1.0,0.0]}},
            ClearValue{depth_stencil:ClearDepthStencilValue{depth:1.0,stencil:0}},
        ];
        let render_area = Rect2D{
            offset : Offset2D{x:0,y:0},
            extent,
        };
        let begin_info = RenderPassBeginInfo{
            s_type : StructureType::RENDER_PASS_BEGIN_INFO,
            p_next : std::ptr::null(),
            framebuffer : framebuffers[i],
            render_pass,
            clear_value_count : clear_values.len() as u32,
            p_clear_values : clear_values.as_ptr(),
            render_area,
        };
        device.cmd_begin_render_pass(command_buffer, &begin_info, if secondary{SubpassContents::SECONDARY_COMMAND_BUFFERS}else{SubpassContents::INLINE});
    }
}
pub unsafe fn end_render_pass(device : &Device, command_buffers : &Vec<CommandBuffer>){
    for &command_buffer in command_buffers.iter(){
        device.cmd_end_render_pass(command_buffer);
    }
}
pub unsafe fn create_drawing_command_buffers(device : &Device, command_pool : CommandPool,pipeline_layout : PipelineLayout, pipelines : &Vec<Pipeline>, render_pass : RenderPass, framebuffers : &Vec<Framebuffer>, descriptor_sets : &Vec<DescriptorSet>, vertex_buffers : &Vec<(u32,BufferAndAllocation)>, extent : Extent2D) -> Vec<CommandBuffer>{
    let command_buffers = create_command_buffers(device, command_pool, descriptor_sets.len() as u32, false);
    begin_primary_command_buffers(device, &command_buffers, CommandBufferUsageFlags::empty());
    begin_render_pass(device, &command_buffers, render_pass, framebuffers, extent, false);
    
    for (i,&command_buffer) in command_buffers.iter().enumerate(){
        device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, pipelines[0]);
        let vertex_buffers_pass = [vertex_buffers[0].1.buffer, vertex_buffers[2].1.buffer];
        device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers_pass, &[0,0]);
        device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &[descriptor_sets[i]], &[]);
        device.cmd_draw(command_buffer, vertex_buffers[0].0, vertex_buffers[2].0, 0, 0);
        device.cmd_bind_pipeline(command_buffer, PipelineBindPoint::GRAPHICS, pipelines[1]);
        let vertex_buffers_pass = [vertex_buffers[1].1.buffer];
        device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers_pass, &[0]);
        device.cmd_bind_descriptor_sets(command_buffer, PipelineBindPoint::GRAPHICS, pipeline_layout, 0, &[descriptor_sets[i]], &[]);
        device.cmd_draw(command_buffer, vertex_buffers[1].0, 1, 0, 0);
    }

    end_render_pass(device, &command_buffers);
    end_command_buffers(device, &command_buffers);
    return command_buffers;
}