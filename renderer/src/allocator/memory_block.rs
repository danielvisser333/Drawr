use std::cmp::max;

use ash::{vk::{DeviceMemory, PhysicalDeviceMemoryProperties, MemoryPropertyFlags, MemoryAllocateInfo, StructureType}, Device};

use super::memory_regions::MemoryRegion;

pub const MIN_BLOCK_SIZE : u64 = 32_000_000;

pub struct MemoryBlock{
    pub memory : DeviceMemory,
    size : u64,
    memory_type : u32,
    pub regions : Vec<Option<MemoryRegion>>,
}
impl MemoryBlock{
    pub unsafe fn create(device : &Device, size : u64, memory_type_filter : u32, memory_property_flags : MemoryPropertyFlags, physical_device_memory_properties : PhysicalDeviceMemoryProperties) -> Self{
        let memory_type = Self::get_memory_type(physical_device_memory_properties, memory_property_flags, memory_type_filter);
        let allocate_info = MemoryAllocateInfo{
            s_type : StructureType::MEMORY_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            allocation_size : max(MIN_BLOCK_SIZE, size),
            memory_type_index : memory_type,
        };
        let memory = device.allocate_memory(&allocate_info, None).expect("Failed to allocate GPU memory");
        return Self{
            memory,
            size : max(MIN_BLOCK_SIZE, size),
            memory_type,
            regions : vec!(),
        }
    }
    fn get_memory_type(physical_device_memory_properties : PhysicalDeviceMemoryProperties, memory_property_flags : MemoryPropertyFlags, memory_type_filter : u32) -> u32{
        for (i, memory_type) in physical_device_memory_properties.memory_types.iter().enumerate(){
            if memory_type.property_flags.contains(memory_property_flags) && (memory_type_filter & (1 << i)) > 0{
                return i as u32;
            }
        }
        panic!("Requested unsupported memory type");
    }
    pub fn is_block_compatible(&self, physical_device_memory_properties : PhysicalDeviceMemoryProperties, memory_type_filter : u32, memory_property_flags : MemoryPropertyFlags) -> bool{
        return physical_device_memory_properties.memory_types[self.memory_type as usize].property_flags.contains(memory_property_flags) && (memory_type_filter & (1 << self.memory_type)) > 0;
    }
    pub fn fit_region(&mut self, region : MemoryRegion) -> usize{
        for (i,memory_region) in self.regions.iter_mut().enumerate(){
            if memory_region.is_none(){*memory_region = Some(region);return i}
        }
        self.regions.push(Some(region));
        return self.regions.len() - 1;
    }
    pub fn try_fit_region(&mut self, size : u64, alignment : u64) -> Option<usize>{
        let regions = self.regions.clone();
        let mut regions = regions.iter().filter_map(|region|{*region}).collect::<Vec<_>>();
        regions.sort_unstable_by_key(|region|region.offset);
        if regions.len() == 0 && self.size >= size{
            let region = MemoryRegion{size,offset:0};
            return Some(self.fit_region(region));
        }
        for (i, region) in regions.iter().enumerate(){
            if i == 0 && region.offset >= size{
                let region = MemoryRegion{size,offset:0};
                return Some(self.fit_region(region));
            }
            if i != 0{
                let start_seek = regions[i-1].offset+regions[i-1].size;
                let alignment_offset = (alignment - (start_seek % alignment)) % alignment;
                let offset = start_seek + alignment_offset;
                if offset >= region.offset && region.offset - offset >= size{
                    let region = MemoryRegion{
                        size,offset,
                    };
                    return Some(self.fit_region(region));
                }
            }
            if i != 0 && i == regions.len() - 1{
                let start_seek = regions[i-1].offset+regions[i-1].size;
                let alignment_offset = (alignment - (start_seek % alignment)) % alignment;
                let offset = start_seek + alignment_offset;
                if offset + size <= self.size{
                    let region = MemoryRegion{
                        size,offset
                    };
                    return Some(self.fit_region(region))
                }
            }
        }
        return None;
    }
    pub unsafe fn destroy(&mut self, device : &Device){
        device.free_memory(self.memory, None);
        self.regions = vec!();
        self.size = 0;
        self.memory_type = 0;
    }
}