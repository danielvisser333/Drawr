use ash::{vk::{Framebuffer, ImageView, FramebufferCreateInfo, StructureType, FramebufferCreateFlags, RenderPass, Extent2D}, Device};

pub unsafe fn create_framebuffers(device : &Device, swapchain_views : &Vec<ImageView>, depth_image : ImageView, render_pass : RenderPass, extent : Extent2D) -> Vec<Framebuffer>{
    let mut framebuffers = vec!();
    for &image in swapchain_views.iter(){
        let attachments = [image,depth_image];
        let framebuffer_create_info = FramebufferCreateInfo{
            s_type : StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : FramebufferCreateFlags::empty(),
            attachment_count : attachments.len() as u32,
            p_attachments : attachments.as_ptr(),
            render_pass : render_pass,
            width : extent.width,
            height : extent.height,
            layers : 1,
        };
        framebuffers.push(device.create_framebuffer(&framebuffer_create_info, None).expect("Failed to create framebuffer"));
    }
    return framebuffers;
}