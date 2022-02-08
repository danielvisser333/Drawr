use ash::{Device, vk::{Semaphore, SemaphoreCreateInfo, StructureType, SemaphoreCreateFlags, Fence, FenceCreateInfo, FenceCreateFlags}};

pub unsafe fn create_semaphores(device : &Device, count : u32) -> Vec<Semaphore>{
    let mut semaphores = vec!();
    let semaphore_create_info = SemaphoreCreateInfo{
        s_type : StructureType::SEMAPHORE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : SemaphoreCreateFlags::empty(),
    };
    for _ in 0..count{
        semaphores.push(device.create_semaphore(&semaphore_create_info, None).expect("Failed to create semaphore"));
    }
    return semaphores;
}
pub unsafe fn create_fences(device : &Device, signaled : bool, count : u32) -> Vec<Fence>{
    let mut fences = vec!();
    let fence_create_info = FenceCreateInfo{
        s_type : StructureType::FENCE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : if signaled{FenceCreateFlags::SIGNALED}else{FenceCreateFlags::empty()},
    };
    for _ in 0..count{
        fences.push(device.create_fence(&fence_create_info, None).expect("Failed to create fence"))
    }
    return fences;
}
pub struct Synchronizer{
    pub in_flight_fences : Vec<Fence>,
    pub image_available_semaphores : Vec<Semaphore>,
    pub render_finished_semaphores : Vec<Semaphore>,
    pub current_frame : usize,
}
impl Synchronizer{
    pub unsafe fn new(device : &Device, count : u32) -> Self{
        return Self{
            image_available_semaphores : create_semaphores(device, count),
            render_finished_semaphores : create_semaphores(device, count),
            in_flight_fences : create_fences(device, true, count),
            current_frame : 0,
        }
    }
    pub unsafe fn destroy(&self, device : &Device){
        for &semaphore in self.image_available_semaphores.iter(){
            device.destroy_semaphore(semaphore, None);
        }
        for &semaphore in self.render_finished_semaphores.iter(){
            device.destroy_semaphore(semaphore, None);
        }
        for &fence in self.in_flight_fences.iter(){
            device.destroy_fence(fence, None);
        }
    }
}