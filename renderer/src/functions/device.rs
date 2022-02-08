use ash::{Instance, extensions::khr::{Surface, Swapchain}, vk::{SurfaceKHR, PhysicalDevice, QueueFlags, PhysicalDeviceType, DeviceQueueCreateInfo, StructureType, DeviceQueueCreateFlags, PhysicalDeviceFeatures, DeviceCreateInfo, DeviceCreateFlags}, Device};

pub fn get_device_handle(instance : &Instance, surface_loader : &Surface, surface : &SurfaceKHR) -> PhysicalDevice{
    let mut fallback_device = None;
    for existing_device in unsafe{instance.enumerate_physical_devices()}.expect("Failed to get Vulkan devices"){
        let mut supports_graphics = false;
        let mut supports_compute = false;
        let mut supports_presentation = false;
        for (i, queue_family) in unsafe{instance.get_physical_device_queue_family_properties(existing_device)}.iter().enumerate(){
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS){supports_graphics=true}
            if queue_family.queue_flags.contains(QueueFlags::COMPUTE){supports_compute=true}
            if unsafe{surface_loader.get_physical_device_surface_support(existing_device, i as u32, *surface)}.expect("Failed to check device surface support"){supports_presentation=true};
        } 
        if supports_compute && supports_graphics && supports_presentation{
            if unsafe{instance.get_physical_device_properties(existing_device)}.device_type == PhysicalDeviceType::DISCRETE_GPU{return existing_device}
            if fallback_device.is_none(){fallback_device = Some(existing_device)};
        }
    }
    return fallback_device.expect("No Vulkan compatible device found");
}
pub struct QueueInfo{
    pub graphics_family : u32,
    pub compute_family : u32,
    pub presentation_family : u32,
    pub transfer_family : u32,
}
impl QueueInfo{
    pub fn new(instance : &Instance, surface_loader : &Surface, surface : &SurfaceKHR, physical_device : PhysicalDevice) -> Self{
        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;
        let mut presentation_family = None;
        for (i, queue_flags) in unsafe{instance.get_physical_device_queue_family_properties(physical_device)}.iter().enumerate(){
            if graphics_family.is_none() && queue_flags.queue_flags.contains(QueueFlags::GRAPHICS){graphics_family = Some(i as u32)}
            let surface_support = unsafe{surface_loader.get_physical_device_surface_support(physical_device, i as u32, *surface).expect("Failed to get device surface queue support")};
            if graphics_family.is_some() && graphics_family.unwrap() == i as u32 && surface_support{
                presentation_family = Some(i as u32)
            }
            else if presentation_family.is_none() && surface_support{presentation_family = Some(i as u32)}
            if compute_family.is_none() && queue_flags.queue_flags.contains(QueueFlags::COMPUTE){compute_family = Some(i as u32)}
            else if queue_flags.queue_flags.contains(QueueFlags::COMPUTE) && !queue_flags.queue_flags.contains(QueueFlags::GRAPHICS){compute_family = Some(i as u32)}
            if transfer_family.is_none() && queue_flags.queue_flags.contains(QueueFlags::TRANSFER){transfer_family = Some(i as u32)}
            else if queue_flags.queue_flags.contains(QueueFlags::TRANSFER) && !queue_flags.queue_flags.contains(QueueFlags::COMPUTE) && !queue_flags.queue_flags.contains(QueueFlags::GRAPHICS){transfer_family = Some(i as u32)}
        }
        return Self{
            graphics_family : graphics_family.unwrap(),
            compute_family : compute_family.unwrap(),
            presentation_family : presentation_family.unwrap(),
            transfer_family : transfer_family.unwrap(),
        }
    }
}
pub unsafe fn create_device(instance : &Instance, physical_device : PhysicalDevice, queue_info : &QueueInfo) -> Device{
    let priorities = [1.0];
    let mut device_queue_create_infos = vec!(
        DeviceQueueCreateInfo{
            s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DeviceQueueCreateFlags::empty(),
            p_queue_priorities : priorities.as_ptr(),
            queue_family_index : queue_info.graphics_family,
            queue_count : 1,
        }
    );
    if queue_info.graphics_family != queue_info.compute_family{
        device_queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                p_queue_priorities : priorities.as_ptr(),
                queue_family_index : queue_info.compute_family,
                queue_count : 1,
            }
        );
    }
    if queue_info.transfer_family != queue_info.graphics_family && queue_info.transfer_family != queue_info.compute_family{
        device_queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                p_queue_priorities : priorities.as_ptr(),
                queue_family_index : queue_info.transfer_family,
                queue_count : 1,
            }
        );
    }
    if queue_info.presentation_family != queue_info.graphics_family && queue_info.presentation_family != queue_info.compute_family && queue_info.presentation_family != queue_info.transfer_family{
        device_queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                p_queue_priorities : priorities.as_ptr(),
                queue_family_index : queue_info.presentation_family,
                queue_count : 1,
            }
        );
    }
    let device_features = PhysicalDeviceFeatures{
        wide_lines:1,
        ..Default::default()
    };
    let enabled_extensions = [Swapchain::name().as_ptr()];
    let device_create_info = DeviceCreateInfo{
        s_type : StructureType::DEVICE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DeviceCreateFlags::empty(),
        enabled_extension_count : enabled_extensions.len() as u32,
        pp_enabled_extension_names : enabled_extensions.as_ptr(),
        enabled_layer_count : 0,
        pp_enabled_layer_names : std::ptr::null(),
        p_enabled_features : &device_features,
        queue_create_info_count : device_queue_create_infos.len() as u32,
        p_queue_create_infos : device_queue_create_infos.as_ptr(),
    };
    return instance.create_device(physical_device, &device_create_info, None).expect("Failed to create Vulkan device");
}