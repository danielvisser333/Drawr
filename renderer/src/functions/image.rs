use ash::{Device, vk::{ImageViewCreateInfo, Image, Format, ImageView, StructureType, ImageViewType, ComponentMapping, ImageSubresourceRange, ImageAspectFlags, ComponentSwizzle, ImageViewCreateFlags, ImageCreateInfo, Extent2D, ImageCreateFlags, Extent3D, ImageType, ImageLayout, SampleCountFlags, SharingMode, ImageTiling, ImageUsageFlags, MemoryPropertyFlags}};

use crate::allocator::{Allocator, ImageAndAllocation};

pub unsafe fn create_swapchain_image_views(device : &Device, images : &Vec<Image>, format : Format) -> Vec<ImageView>{
    let mut views = vec!();
    for &image in images.iter(){
        let image_view_create_info = ImageViewCreateInfo{
            s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : ImageViewCreateFlags::empty(),
            image,
            components : ComponentMapping{r : ComponentSwizzle::R, g : ComponentSwizzle::G, b : ComponentSwizzle::B, a : ComponentSwizzle::A},
            format,
            view_type : ImageViewType::TYPE_2D,
            subresource_range : ImageSubresourceRange{
                aspect_mask : ImageAspectFlags::COLOR,
                base_array_layer : 0,
                base_mip_level : 0,
                layer_count : 1,
                level_count : 1,
            }
        };
        views.push(device.create_image_view(&image_view_create_info, None).expect("Failed to create image view"));
    }
    return views;
}
pub unsafe fn create_depth_image(device : &Device, allocator : &mut Allocator, extent : Extent2D, format : Format) -> ImageAndView{
    let image_create_info = ImageCreateInfo{
        s_type : StructureType::IMAGE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : ImageCreateFlags::empty(),
        array_layers : 1,
        format,
        extent : Extent3D{width:extent.width,height:extent.height,depth:1},
        image_type : ImageType::TYPE_2D,
        initial_layout : ImageLayout::UNDEFINED,
        mip_levels : 1,
        p_queue_family_indices : std::ptr::null(),
        queue_family_index_count : 0,
        samples : SampleCountFlags::TYPE_1,
        sharing_mode : SharingMode::EXCLUSIVE,
        tiling : ImageTiling::OPTIMAL,
        usage : ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
    };
    let image = device.create_image(&image_create_info, None).expect("Failed to create depth image");
    let image = ImageAndAllocation::new(allocator, image, MemoryPropertyFlags::DEVICE_LOCAL);
    let image_view_create_info = ImageViewCreateInfo{
        s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : ImageViewCreateFlags::empty(),
        image:image.image,
        components : ComponentMapping{r : ComponentSwizzle::R, g : ComponentSwizzle::G, b : ComponentSwizzle::B, a : ComponentSwizzle::A},
        format,
        view_type : ImageViewType::TYPE_2D,
        subresource_range : ImageSubresourceRange{
            aspect_mask : ImageAspectFlags::DEPTH,
            base_array_layer : 0,
            base_mip_level : 0,
            layer_count : 1,
            level_count : 1,
        }
    };
    let view = device.create_image_view(&image_view_create_info, None).expect("Failed to create image view");
    return ImageAndView{image,view,}
}
pub struct ImageAndView{
    pub image : ImageAndAllocation,
    pub view : ImageView,
}
impl ImageAndView{
    pub unsafe fn destroy(&self, allocator : &mut Allocator){
        allocator.device.destroy_image_view(self.view, None);
        self.image.destroy(allocator);
    }
}