use ash::{vk::{ShaderModule, ShaderModuleCreateInfo, StructureType, ShaderModuleCreateFlags}, Device};

const SHADERS : [&str;4] = [
    "main.vert","main.frag","grid.vert", "grid.frag"
];

pub unsafe fn load_shaders(device: &Device) -> Vec<ShaderModule>{
    let mut shader_dir = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
    shader_dir.push("./shaders");
    let mut shaders = vec!();
    for shader in SHADERS{
        let path = shader_dir.join(format!("./{}.spv",shader));
        let file = std::fs::read(path).expect("Failed to load shader");
        let shader_create_info = ShaderModuleCreateInfo{
            s_type : StructureType::SHADER_MODULE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : ShaderModuleCreateFlags::empty(),
            code_size : file.len(),
            p_code : file.as_ptr() as *const u32,
        };
        shaders.push(device.create_shader_module(&shader_create_info, None).expect("A shader got corrupted"));
    }
    return shaders;
}