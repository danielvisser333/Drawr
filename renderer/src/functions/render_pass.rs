use ash::{Device, vk::{Format, RenderPass, AttachmentDescription, AttachmentDescriptionFlags, ImageLayout, AttachmentLoadOp, AttachmentStoreOp, SampleCountFlags, AttachmentReference, SubpassDescription, SubpassDescriptionFlags, PipelineBindPoint, SubpassDependency, DependencyFlags, SUBPASS_EXTERNAL, PipelineStageFlags, AccessFlags, RenderPassCreateInfo, StructureType, RenderPassCreateFlags}};

pub unsafe fn create_render_pass(device : &Device, format : Format, depth_format : Format) -> RenderPass{
    let render_pass_attachments = [
        AttachmentDescription{
            flags : AttachmentDescriptionFlags::empty(),
            format,
            initial_layout : ImageLayout::UNDEFINED,
            final_layout : ImageLayout::PRESENT_SRC_KHR,
            load_op : AttachmentLoadOp::CLEAR,
            store_op : AttachmentStoreOp::STORE,
            samples : SampleCountFlags::TYPE_1,
            stencil_load_op : AttachmentLoadOp::DONT_CARE,
            stencil_store_op : AttachmentStoreOp::DONT_CARE,
        },
        AttachmentDescription{
            flags : AttachmentDescriptionFlags::empty(),
            format : depth_format,
            initial_layout : ImageLayout::UNDEFINED,
            final_layout : ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            load_op : AttachmentLoadOp::CLEAR,
            store_op : AttachmentStoreOp::STORE,
            samples : SampleCountFlags::TYPE_1,
            stencil_load_op : AttachmentLoadOp::DONT_CARE,
            stencil_store_op : AttachmentStoreOp::DONT_CARE,
        },
    ];
    let color_attachment_references = [
        AttachmentReference{
            attachment : 0,
            layout : ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }
    ];
    let depth_attachment_reference = AttachmentReference{
        attachment : 1,
        layout : ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let subpasses = [
        SubpassDescription{
            flags : SubpassDescriptionFlags::empty(),
            color_attachment_count : color_attachment_references.len() as u32,
            p_color_attachments : color_attachment_references.as_ptr(),
            p_depth_stencil_attachment : &depth_attachment_reference,
            input_attachment_count : 0,
            p_input_attachments : std::ptr::null(),
            preserve_attachment_count : 0,
            p_preserve_attachments : std::ptr::null(),
            p_resolve_attachments : std::ptr::null(),
            pipeline_bind_point : PipelineBindPoint::GRAPHICS,
        }
    ];
    let dependencies = [
        SubpassDependency{
            dependency_flags : DependencyFlags::empty(),
            src_subpass : SUBPASS_EXTERNAL,
            dst_subpass : 0,
            src_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_access_mask : AccessFlags::empty(),
            dst_access_mask : AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        }
    ];
    let render_pass_create_info = RenderPassCreateInfo{
        s_type : StructureType::RENDER_PASS_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : RenderPassCreateFlags::empty(),
        attachment_count : render_pass_attachments.len() as u32,
        p_attachments : render_pass_attachments.as_ptr(),
        subpass_count : subpasses.len() as u32,
        p_subpasses : subpasses.as_ptr(),
        dependency_count : dependencies.len() as u32,
        p_dependencies : dependencies.as_ptr(),
    };
    return device.create_render_pass(&render_pass_create_info, None).expect("Failed to create render pass");
}