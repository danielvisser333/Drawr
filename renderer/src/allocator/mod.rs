pub mod memory_block;
pub mod memory_regions;

use ash::{Instance, vk::{PhysicalDevice, PhysicalDeviceMemoryProperties, MemoryPropertyFlags, Image, Buffer, DeviceMemory}, Device};

use self::memory_block::MemoryBlock;

pub struct Allocator{
    physical_device_memory_properties : PhysicalDeviceMemoryProperties,
    pub device : Device,
    blocks : Vec<Option<MemoryBlock>>,
}
impl Allocator{
    pub unsafe fn new(instance : &Instance, physical_device : PhysicalDevice, device : Device) -> Self{
        let physical_device_memory_properties = instance.get_physical_device_memory_properties(physical_device);
        return Self{physical_device_memory_properties,device,blocks:vec!()}
    }
    pub unsafe fn create_allocation(&mut self, size : u64, alignment : u64, memory_property_flags : MemoryPropertyFlags, memory_type_filter : u32) -> MemoryRegionPointer{
        for (i, block) in self.blocks.iter_mut().enumerate(){
            if block.is_some() && block.as_ref().unwrap().is_block_compatible(self.physical_device_memory_properties, memory_type_filter, memory_property_flags){
               let region = block.as_mut().unwrap().try_fit_region(size, alignment);
               if region.is_some(){
                   return MemoryRegionPointer{
                       block : i,
                       region : region.unwrap(),
                   };
               }
            }
        }
        let block = self.fit_block(MemoryBlock::create(&self.device, size, memory_type_filter, memory_property_flags, self.physical_device_memory_properties));
        let region = self.blocks[block].as_mut().unwrap().try_fit_region(size, alignment).unwrap();
        return MemoryRegionPointer{
            block,region,
        }
    }
    pub unsafe fn destroy_allocation(&mut self, allocation : &MemoryRegionPointer){
        self.blocks[allocation.block].as_mut().unwrap().regions.remove(allocation.region);
    }
    fn fit_block(&mut self, block : MemoryBlock) -> usize{
        for (i,memory_block) in self.blocks.iter_mut().enumerate(){
            if memory_block.is_none(){*memory_block = Some(block); return i;}
        }
        self.blocks.push(Some(block));
        return self.blocks.len() - 1;
    }
    pub fn destroy(&mut self){
        for block in self.blocks.iter_mut(){
            if block.is_some(){
                let block = block.as_mut().unwrap();
                unsafe{block.destroy(&self.device)};
            }
            *block = None;
        }
    }
    pub unsafe fn get_memory_map_data(&self, allocation : &MemoryRegionPointer) -> MemoryMapData{
        let block = self.blocks[allocation.block].as_ref().unwrap();
        let region = block.regions[allocation.region].as_ref().unwrap();
        return MemoryMapData{
            memory:block.memory,
            offset:region.offset,
            size:region.size,
        }
    }
    pub fn dump_contents(&self){
        for (i, block) in self.blocks.iter().enumerate(){
            if block.is_some(){
                println!("[{}]Block:",i);
                for (i,region) in block.as_ref().unwrap().regions.iter().enumerate(){
                    if region.is_some(){
                        println!("  [{i}]Region, Offset: {}, Size: {}",region.as_ref().unwrap().offset, region.as_ref().unwrap().size);
                    }
                    else{
                        println!("  [{}]Region, Non-existant",i);
                    }
                }
            }
            else{
                println!("[{}]Block, Non-existant",i);
            }
        }
    }
}
#[derive(Debug)]
pub struct MemoryRegionPointer{
    block : usize,
    region : usize,
}
pub struct MemoryMapData{
    pub memory : DeviceMemory,
    pub offset : u64,
    pub size : u64,
}
pub struct ImageAndAllocation{
    pub image : Image,
    pub allocation : MemoryRegionPointer,
}
impl ImageAndAllocation{
    pub unsafe fn new(allocator : &mut Allocator, image : Image, memory_property_flags : MemoryPropertyFlags) -> Self{
        let requirements = allocator.device.get_image_memory_requirements(image);
        let allocation = allocator.create_allocation(requirements.size, requirements.alignment, memory_property_flags, requirements.memory_type_bits);
        allocator.device.bind_image_memory(image, 
        allocator.blocks[allocation.block].as_ref().unwrap().memory, 
        allocator.blocks[allocation.block].as_ref().unwrap().regions[allocation.region].as_ref().unwrap().offset).expect("Failed to bind image memory");
        return Self{
            image,allocation,
        }
    }
    pub unsafe fn destroy(&self, allocator : &mut Allocator){
        allocator.destroy_allocation(&self.allocation);
        allocator.device.destroy_image(self.image, None);
    }
}
#[derive(Debug)]
pub struct BufferAndAllocation{
    pub buffer : Buffer,
    pub allocation : MemoryRegionPointer,
}
impl BufferAndAllocation{
    pub unsafe fn new(allocator : &mut Allocator, buffer : Buffer, memory_property_flags : MemoryPropertyFlags) -> Self{
        let requirements = allocator.device.get_buffer_memory_requirements(buffer);
        let allocation = allocator.create_allocation(requirements.size, requirements.alignment, memory_property_flags, requirements.memory_type_bits);
        allocator.device.bind_buffer_memory(buffer, 
        allocator.blocks[allocation.block].as_ref().unwrap().memory, 
        allocator.blocks[allocation.block].as_ref().unwrap().regions[allocation.region].as_ref().unwrap().offset).expect("Failed to bind image memory");
        return Self{
            buffer,allocation,
        }
    }
    pub unsafe fn destroy(&self, allocator : &mut Allocator){
        allocator.destroy_allocation(&self.allocation);
        allocator.device.destroy_buffer(self.buffer, None);
    }
}