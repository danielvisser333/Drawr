use ash::{Device, vk::{CommandPool, Queue}};

use crate::{allocator::{BufferAndAllocation, Allocator}, math::{Vertex, ModelMatrix, InstanceVertex}};

pub unsafe fn create_vertex_buffers(device : &Device, allocator : &mut Allocator, command_pool : CommandPool, queue : Queue, instance_positions : Vec<ModelMatrix>) -> Vec<(u32,BufferAndAllocation)>{
    let vertex_data = InstanceVertex::get_initial_vertex_data();
    let grid_data = Vertex::get_grid();
    let mut buffers = vec!();
    buffers.push((vertex_data.len() as u32,super::buffer::create_vertex_buffer(device, allocator, command_pool, queue, vertex_data.to_vec())));
    buffers.push((grid_data.len() as u32, super::buffer::create_vertex_buffer(device, allocator, command_pool, queue, grid_data.to_vec())));
    buffers.push((instance_positions.len()as u32, super::buffer::create_vertex_buffer(device, allocator, command_pool, queue, instance_positions)));
    println!("{}",std::mem::size_of::<ModelMatrix>());
    return buffers;
}