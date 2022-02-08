use ash::{Instance, vk::{PhysicalDevice, SurfaceKHR, SurfaceFormatKHR, Format, ColorSpaceKHR, FormatFeatureFlags, PresentModeKHR, Extent2D, SurfaceTransformFlagsKHR, SwapchainKHR, SwapchainCreateInfoKHR, StructureType, SwapchainCreateFlagsKHR, CompositeAlphaFlagsKHR, ImageUsageFlags, SharingMode}, extensions::khr::{Surface, Swapchain}};
use winit::dpi::PhysicalSize;

use super::device::QueueInfo;

const SURFACE_FORMATS : [SurfaceFormatKHR;2] = [
    SurfaceFormatKHR{format : Format::R8G8B8A8_SRGB, color_space : ColorSpaceKHR::SRGB_NONLINEAR},
    SurfaceFormatKHR{format : Format::B8G8R8A8_SRGB, color_space : ColorSpaceKHR::SRGB_NONLINEAR},
];
const DEPTH_FORMATS : [Format; 6] = [
    Format::D16_UNORM,Format::D16_UNORM_S8_UINT,Format::D32_SFLOAT,Format::D24_UNORM_S8_UINT,Format::D32_SFLOAT_S8_UINT,Format::X8_D24_UNORM_PACK32,
];

pub struct SwapchainInfo{
    pub surface_format : Format,
    pub depth_format : Format,
    pub color_space : ColorSpaceKHR,
    pub present_mode : PresentModeKHR,
    pub extent : Extent2D,
    pub transform : SurfaceTransformFlagsKHR,
    pub min_image_count : u32,
}
impl SwapchainInfo{
    pub fn new(instance : &Instance, physical_device : PhysicalDevice, surface_loader : &Surface, surface : SurfaceKHR, window_size : PhysicalSize<u32>) -> Self{
        let supported_surface_formats = unsafe{surface_loader.get_physical_device_surface_formats(physical_device, surface)}.expect("Failed to get swapchain formats");
        let supported_surface_present_modes = unsafe{surface_loader.get_physical_device_surface_present_modes(physical_device, surface)}.expect("Failed to get supported present modes");
        let surface_capabilities = unsafe{surface_loader.get_physical_device_surface_capabilities(physical_device, surface)}.expect("Failed to query swapchain capabilities");
        let mut surface_format = supported_surface_formats[0];
        for prefered_surface_format in SURFACE_FORMATS{
            if supported_surface_formats.contains(&prefered_surface_format){surface_format = prefered_surface_format;break}
        }
        let mut depth_format = Format::R8_SINT;
        for is_depth_format in DEPTH_FORMATS{
            if unsafe{instance.get_physical_device_format_properties(physical_device, is_depth_format).optimal_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT)}{
                depth_format = is_depth_format;break;
            }
        }
        if depth_format == Format::R8_SINT{panic!("No supported depth format")}
        let present_mode = if supported_surface_present_modes.contains(&PresentModeKHR::MAILBOX){PresentModeKHR::MAILBOX}else{PresentModeKHR::FIFO};
        let extent = if surface_capabilities.current_extent.width != u32::MAX{surface_capabilities.current_extent}else{Extent2D{width:window_size.width,height:window_size.height}};
        let transform = surface_capabilities.current_transform;
        let min_image_count = if surface_capabilities.min_image_count + 1 <= surface_capabilities.max_image_count || surface_capabilities.max_image_count == 0{surface_capabilities.min_image_count + 1}else{surface_capabilities.max_image_count};

        return Self{
            surface_format : surface_format.format,
            depth_format,
            color_space : surface_format.color_space,
            present_mode,
            extent,
            transform,
            min_image_count,
        }
    }
}
pub unsafe fn create_swapchain(swapchain_loader : &Swapchain, swapchain_info : &SwapchainInfo, queue_info : &QueueInfo, surface : SurfaceKHR) -> SwapchainKHR{
    let queue_families = [queue_info.graphics_family,queue_info.presentation_family];
    let exclusive = queue_families[0] == queue_families[1];
    let swapchain_create_info = SwapchainCreateInfoKHR{
        s_type : StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next : std::ptr::null(),
        flags : SwapchainCreateFlagsKHR::empty(),
        clipped : 1,
        composite_alpha : CompositeAlphaFlagsKHR::OPAQUE,
        image_array_layers : 1,
        image_usage : ImageUsageFlags::COLOR_ATTACHMENT,
        old_swapchain : SwapchainKHR::null(),
        surface,
        queue_family_index_count : if exclusive{0}else{2},
        p_queue_family_indices : if exclusive{std::ptr::null()}else{queue_families.as_ptr()},
        image_sharing_mode : if exclusive{SharingMode::EXCLUSIVE}else{SharingMode::CONCURRENT},
        image_color_space : swapchain_info.color_space,
        image_extent : swapchain_info.extent,
        image_format : swapchain_info.surface_format,
        min_image_count : swapchain_info.min_image_count,
        pre_transform : swapchain_info.transform,
        present_mode : swapchain_info.present_mode,
    };
    return swapchain_loader.create_swapchain(&swapchain_create_info, None).expect("Failed to create Vulkan swapchain")
}