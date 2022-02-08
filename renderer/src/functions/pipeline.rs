use std::ffi::{c_void, CString};

use ash::{vk::{PipelineLayoutCreateFlags, DescriptorSetLayout, PipelineLayout, PipelineLayoutCreateInfo, StructureType, PipelineCache, PipelineCacheCreateInfo, PipelineCacheCreateFlags, Pipeline, PipelineCreateFlags, GraphicsPipelineCreateInfo, ShaderStageFlags, PipelineShaderStageCreateFlags, PipelineShaderStageCreateInfo, SampleCountFlags, PipelineMultisampleStateCreateFlags, PipelineMultisampleStateCreateInfo, StencilOpState, CompareOp, PipelineDepthStencilStateCreateFlags, PipelineDepthStencilStateCreateInfo, LogicOp, PipelineColorBlendStateCreateFlags, PipelineColorBlendStateCreateInfo, BlendFactor, ColorComponentFlags, BlendOp, PipelineColorBlendAttachmentState, PipelineViewportStateCreateFlags, PipelineViewportStateCreateInfo, Viewport, Offset2D, Rect2D, PolygonMode, FrontFace, CullModeFlags, PipelineRasterizationStateCreateFlags, PipelineRasterizationStateCreateInfo, PrimitiveTopology, PipelineInputAssemblyStateCreateFlags, PipelineInputAssemblyStateCreateInfo, PipelineVertexInputStateCreateFlags, PipelineVertexInputStateCreateInfo, RenderPass, ShaderModule, Extent2D}, Device};

use crate::math::{Vertex, InstanceVertex};

pub unsafe fn create_pipeline_layout(device : &Device, descriptor_set_layout : &DescriptorSetLayout) -> PipelineLayout{
    let pipeline_layout_create_info = PipelineLayoutCreateInfo{
        s_type : StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineLayoutCreateFlags::empty(),
        p_push_constant_ranges : std::ptr::null(),
        push_constant_range_count : 0,
        p_set_layouts : descriptor_set_layout,
        set_layout_count : 1,
    };
    return device.create_pipeline_layout(&pipeline_layout_create_info, None).expect("Failed to create pipeline layout");
}
pub unsafe fn create_pipeline_cache(device : &Device) -> PipelineCache{
    let mut file = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    file.push("./pipeline.cache");
    let pipeline_cache_create_info;
    if !file.exists(){
        pipeline_cache_create_info = PipelineCacheCreateInfo{
            s_type : StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineCacheCreateFlags::empty(),
            initial_data_size : 0,
            p_initial_data : std::ptr::null(),
        };
    }
    else{
        let file_contents = std::fs::read(file).expect("Failed to read pipeline cache");
        pipeline_cache_create_info = PipelineCacheCreateInfo{
            s_type : StructureType::PIPELINE_CACHE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineCacheCreateFlags::empty(),
            p_initial_data : file_contents.as_ptr() as *const c_void,
            initial_data_size : file_contents.len(),
        };
    }
    return device.create_pipeline_cache(&pipeline_cache_create_info, None).expect("Failed to create pipeline cache");
}
pub unsafe fn save_pipeline_cache(device : &Device, cache : PipelineCache){
    let cache_contents = device.get_pipeline_cache_data(cache).expect("Failed to get pipeline cache contents");
    let mut file = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    file.push("./pipeline.cache");
    if !file.parent().unwrap().exists(){std::fs::create_dir_all(file.clone()).expect("Failed to create cache directory")}
    std::fs::write(file, cache_contents).expect("Failed to save pipeline cache");
}
pub unsafe fn create_pipelines(device : &Device, cache : PipelineCache, layout : PipelineLayout, render_pass : RenderPass, modules : &Vec<ShaderModule>, extent : Extent2D) -> Vec<Pipeline>{
    let vertex_attributes = InstanceVertex::get_attributes();
    let vertex_bindings = InstanceVertex::get_bindings();
    let vertex_input_state = PipelineVertexInputStateCreateInfo{
        s_type : StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineVertexInputStateCreateFlags::empty(),
        p_vertex_attribute_descriptions : vertex_attributes.as_ptr(),
        vertex_attribute_description_count : vertex_attributes.len() as u32,
        p_vertex_binding_descriptions : vertex_bindings.as_ptr(),
        vertex_binding_description_count : vertex_bindings.len() as u32,
    };
    let grid_vertex_attributes = Vertex::get_attributes();
    let grid_vertex_bindings = Vertex::get_bindings();
    let vertex_input_grid_state = PipelineVertexInputStateCreateInfo{
        s_type : StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineVertexInputStateCreateFlags::empty(),
        p_vertex_attribute_descriptions : grid_vertex_attributes.as_ptr(),
        vertex_attribute_description_count : grid_vertex_attributes.len() as u32,
        p_vertex_binding_descriptions : grid_vertex_bindings.as_ptr(),
        vertex_binding_description_count : grid_vertex_bindings.len() as u32,
    };
    let input_assembly_state = PipelineInputAssemblyStateCreateInfo{
        s_type : StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineInputAssemblyStateCreateFlags::empty(),
        primitive_restart_enable : 0,
        topology : PrimitiveTopology::TRIANGLE_LIST,
    };
    let input_assembly_grid_state = PipelineInputAssemblyStateCreateInfo{
        s_type : StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineInputAssemblyStateCreateFlags::empty(),
        primitive_restart_enable : 0,
        topology : PrimitiveTopology::LINE_LIST,
    };
    let rasterization_state = PipelineRasterizationStateCreateInfo{
        s_type : StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineRasterizationStateCreateFlags::empty(),
        cull_mode : CullModeFlags::BACK,
        front_face : FrontFace::COUNTER_CLOCKWISE,
        depth_bias_clamp : 0.0,
        depth_bias_constant_factor : 0.0,
        depth_bias_enable : 0,
        depth_bias_slope_factor : 0.0,
        depth_clamp_enable : 0,
        line_width : 1.0,
        rasterizer_discard_enable : 0,
        polygon_mode : PolygonMode::FILL, 
    };
    let rasterization_grid_state = PipelineRasterizationStateCreateInfo{
        s_type : StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineRasterizationStateCreateFlags::empty(),
        cull_mode : CullModeFlags::BACK,
        front_face : FrontFace::CLOCKWISE,
        depth_bias_clamp : 0.0,
        depth_bias_constant_factor : 0.0,
        depth_bias_enable : 0,
        depth_bias_slope_factor : 0.0,
        depth_clamp_enable : 0,
        line_width : 2.0,
        rasterizer_discard_enable : 0,
        polygon_mode : PolygonMode::FILL, 
    };
    let scissors = [
        Rect2D{
            extent,
            offset : Offset2D{x : 0,y : 0,}
        }
    ];
    let viewports = [
        Viewport{
            height : extent.height as f32,
            width : extent.width as f32,
            max_depth : 1.0,
            min_depth : 0.0,
            x : 0.0,
            y : 0.0,
        }
    ];
    let viewport_state = PipelineViewportStateCreateInfo{
        s_type : StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineViewportStateCreateFlags::empty(),
        p_scissors : scissors.as_ptr(),
        scissor_count : scissors.len() as u32,
        p_viewports : viewports.as_ptr(),
        viewport_count : viewports.len() as u32,
    };
    let color_blend_attachments = [
        PipelineColorBlendAttachmentState{
            alpha_blend_op : BlendOp::ADD,
            color_blend_op : BlendOp::ADD,
            blend_enable : 0,
            color_write_mask : ColorComponentFlags::R | ColorComponentFlags::G | ColorComponentFlags::B | ColorComponentFlags::A,
            dst_alpha_blend_factor : BlendFactor::ZERO,
            dst_color_blend_factor : BlendFactor::ZERO,
            src_alpha_blend_factor : BlendFactor::ONE,
            src_color_blend_factor : BlendFactor::ONE,
        }
    ];
    let color_blend_state = PipelineColorBlendStateCreateInfo{
        s_type : StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineColorBlendStateCreateFlags::empty(),
        attachment_count: color_blend_attachments.len() as u32,
        p_attachments : color_blend_attachments.as_ptr(),
        blend_constants : [0.0,0.0,0.0,0.0],
        logic_op : LogicOp::COPY,
        logic_op_enable : 0,
    };
    let depth_stencil_state = PipelineDepthStencilStateCreateInfo{
        s_type : StructureType::PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineDepthStencilStateCreateFlags::empty(),
        depth_bounds_test_enable : 0,
        depth_compare_op : CompareOp::LESS,
        depth_test_enable : 1,
        depth_write_enable : 1,
        back : StencilOpState::default(),
        front : StencilOpState::default(),
        max_depth_bounds : 1.0,
        min_depth_bounds : 0.0,
        stencil_test_enable : 0
    };
    let multisample_state = PipelineMultisampleStateCreateInfo{
        s_type : StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : PipelineMultisampleStateCreateFlags::empty(),
        alpha_to_coverage_enable : 0,
        alpha_to_one_enable : 0,
        min_sample_shading : 1.0,
        p_sample_mask : std::ptr::null(),
        rasterization_samples : SampleCountFlags::TYPE_1,
        sample_shading_enable : 0,
    };
    let name = CString::new("main").unwrap();
    let stages = [
        PipelineShaderStageCreateInfo{
            s_type : StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineShaderStageCreateFlags::empty(),
            module : modules[0],
            p_specialization_info : std::ptr::null(),
            stage : ShaderStageFlags::VERTEX,
            p_name : name.as_ptr(),
        },
        PipelineShaderStageCreateInfo{
            s_type : StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineShaderStageCreateFlags::empty(),
            module : modules[1],
            p_specialization_info : std::ptr::null(),
            stage : ShaderStageFlags::FRAGMENT,
            p_name : name.as_ptr(),
        },
    ];
    let grid_stages = [
        PipelineShaderStageCreateInfo{
            s_type : StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineShaderStageCreateFlags::empty(),
            module : modules[2],
            p_specialization_info : std::ptr::null(),
            stage : ShaderStageFlags::VERTEX,
            p_name : name.as_ptr(),
        },
        PipelineShaderStageCreateInfo{
            s_type : StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineShaderStageCreateFlags::empty(),
            module : modules[3],
            p_specialization_info : std::ptr::null(),
            stage : ShaderStageFlags::FRAGMENT,
            p_name : name.as_ptr(),
        },
    ];
    let pipeline_create_infos = [
        GraphicsPipelineCreateInfo{
            s_type : StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineCreateFlags::empty(),
            base_pipeline_handle : Pipeline::null(),
            base_pipeline_index : -1,
            layout : layout,
            render_pass : render_pass,
            subpass : 0,
            p_color_blend_state : &color_blend_state,
            p_depth_stencil_state : &depth_stencil_state,
            p_dynamic_state : std::ptr::null(),
            p_input_assembly_state : &input_assembly_state,
            p_rasterization_state : &rasterization_state,
            p_tessellation_state : std::ptr::null(),
            p_vertex_input_state : &vertex_input_state,
            p_viewport_state: &viewport_state,
            p_multisample_state : &multisample_state,
            p_stages : stages.as_ptr(),
            stage_count : stages.len() as u32,
        },
        GraphicsPipelineCreateInfo{
            s_type : StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : PipelineCreateFlags::empty(),
            base_pipeline_handle : Pipeline::null(),
            base_pipeline_index : -1,
            layout : layout,
            render_pass : render_pass,
            subpass : 0,
            p_color_blend_state : &color_blend_state,
            p_depth_stencil_state : &depth_stencil_state,
            p_dynamic_state : std::ptr::null(),
            p_input_assembly_state : &input_assembly_grid_state,
            p_rasterization_state : &rasterization_grid_state,
            p_tessellation_state : std::ptr::null(),
            p_vertex_input_state : &vertex_input_grid_state,
            p_viewport_state: &viewport_state,
            p_multisample_state : &multisample_state,
            p_stages : grid_stages.as_ptr(),
            stage_count : grid_stages.len() as u32,
        }
    ];
    return device.create_graphics_pipelines(cache, &pipeline_create_infos, None).expect("Failed to create pipelines");
}