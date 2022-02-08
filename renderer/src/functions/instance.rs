use std::ffi::CString;

use ash::{Instance, vk::{ApplicationInfo, StructureType, InstanceCreateInfo, InstanceCreateFlags, API_VERSION_1_0}, Entry};
use winit::window::Window;

pub unsafe fn create_instance(entry : &Entry, window : &Window, debugging : bool) -> Instance{
    let name = CString::new("PWS").unwrap();
    let app_info = ApplicationInfo{
        s_type : StructureType::APPLICATION_INFO,
        p_next : std::ptr::null(),
        api_version : API_VERSION_1_0,
        application_version : 1,
        engine_version : 1,
        p_application_name : name.as_ptr(),
        p_engine_name : name.as_ptr(),
    };
    let window_extensions = ash_window::enumerate_required_extensions(window).expect("Failed to get window extensions");
    let enabled_extensions = window_extensions.iter().map(|ext|ext.as_ptr()).collect::<Vec<_>>();
    let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    let enabled_layers = if debugging {vec!(validation_layer.as_ptr())}else{vec!()};
    let instance_create_info = InstanceCreateInfo{
        s_type : StructureType::INSTANCE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : InstanceCreateFlags::empty(),
        p_application_info : &app_info,
        enabled_extension_count : enabled_extensions.len() as u32,
        pp_enabled_extension_names : enabled_extensions.as_ptr(),
        enabled_layer_count : enabled_layers.len() as u32,
        pp_enabled_layer_names : enabled_layers.as_ptr(),
    };
    return entry.create_instance(&instance_create_info, None).expect("Failed to create instance");
}