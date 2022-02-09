pub mod functions;
pub mod allocator;
pub mod math;

use std::sync::mpsc::{Sender, Receiver};

use allocator::{Allocator, BufferAndAllocation};
use ash::{Entry, Instance, extensions::khr::{Surface, Swapchain}, vk::{SurfaceKHR, SwapchainKHR, ImageView, PhysicalDevice, RenderPass, ShaderModule, Framebuffer, DescriptorSetLayout, PipelineLayout, PipelineCache, DescriptorPool, DescriptorSet, Pipeline, Fence, CommandPool, Queue, CommandBuffer, PipelineStageFlags, SubmitInfo, StructureType, PresentInfoKHR}, Device};
use functions::{image::ImageAndView, device::QueueInfo, synchronization::Synchronizer, buffer::UniformBufferObject};
use math::{UniformBuffer, camera::Camera, ModelMatrix};
use rayon::ThreadPoolBuilder;
use winit::{event_loop::{EventLoop, ControlFlow}, window::Window, event::{Event, WindowEvent, StartCause, VirtualKeyCode, DeviceEvent, MouseScrollDelta, MouseButton, ElementState}, dpi::PhysicalSize, platform::windows::EventLoopExtWindows};
#[cfg(target_os="unix")]
use winit::platform::{unix::EventLoopExtUnix, run_return::EventLoopExtRunReturn};
#[cfg(target_os="windows")]
use winit::platform::run_return::EventLoopExtRunReturn;
const MAX_FRAMES_IN_FLIGHT : usize = 2;

pub struct Renderer{
    sender : Sender<RenderTask>,
    receiver : Receiver<RenderResult>,
}
impl Renderer{
    pub fn new(debug : bool) -> Self{
        let thread_pool = ThreadPoolBuilder::new().build().expect("Failed to create threadpool");
        let (sender, receiver_render_thread) = std::sync::mpsc::channel();
        let (sender_render_thread, receiver) = std::sync::mpsc::channel();
        thread_pool.spawn(move ||{
            println!("Created render thread");
            let mut event_loop : EventLoop<()> = EventLoop::new_any_thread();
            let window = Window::new(&event_loop).expect("Failed to create render window");
            let mut renderer = RenderOnThread::new(&window, debug);
            event_loop.run_return(|event,_,control_flow|{
                match event{
                    Event::WindowEvent{event,window_id:_}=>{
                        match event{
                            WindowEvent::CloseRequested=>{*control_flow=ControlFlow::Exit}
                            WindowEvent::KeyboardInput{device_id:_, is_synthetic:_, input}=>{
                                if input.virtual_keycode == Some(VirtualKeyCode::F10){
                                    renderer.allocator.dump_contents();
                                }
                            }
                            WindowEvent::MouseWheel{delta, .. }=>{
                                match delta{
                                    MouseScrollDelta::LineDelta(_,y) =>{
                                        renderer.camera.mouse_zoom(y);
                                    }
                                    MouseScrollDelta::PixelDelta(delta)=>{
                                        renderer.camera.mouse_zoom(delta.y as f32);
                                    }
                                }
                            }
                            WindowEvent::MouseInput{button,state,..} => {
                                match button{
                                    MouseButton::Left =>{
                                        match state{
                                            ElementState::Pressed=>{renderer.camera.left_mouse_button_pressed = true}
                                            ElementState::Released=>{renderer.camera.left_mouse_button_pressed = false}
                                        }
                                    }
                                    _=>{}
                                }
                            }
                            _=>{}
                        }
                    }
                    Event::RedrawRequested(_) => {
                        let resized = renderer.draw();
                        if resized{renderer.recreate_swapchain(window.inner_size());}
                    }
                    Event::MainEventsCleared => {
                        window.request_redraw();
                    }
                    Event::NewEvents(start) =>{
                        match start{
                            StartCause::Init=>{*control_flow=ControlFlow::Poll}
                            _=>{}
                        }
                    }
                    Event::DeviceEvent{device_id:_,event}=>{
                        match event{
                            DeviceEvent::MouseMotion{delta}=>{
                                renderer.camera.mouse_movement(delta);
                            }
                            DeviceEvent::MouseWheel{delta} => {
                                match delta{
                                    MouseScrollDelta::LineDelta(_,y) =>{
                                        renderer.camera.mouse_zoom(y);
                                    }
                                    MouseScrollDelta::PixelDelta(delta)=>{
                                        renderer.camera.mouse_zoom(delta.y as f32);
                                    }
                                }
                            }
                            _=>{}
                        }
                    }
                    _=>{}
                }
            });
            drop(renderer);
            sender_render_thread.send(RenderResult::Success).unwrap();
            println!("Destroying render thread");
        });
        return Self{
            sender,receiver,
        }
    }
    pub fn await_close_request(self){
        self.receiver.recv().expect("Failed to block on render complete");
    }
}
pub enum RenderTask{
    Draw,
}
pub enum RenderResult{
    Success
}
struct RenderOnThread{
    _entry : Entry,
    instance : Instance,
    surface_loader : Surface,
    surface : SurfaceKHR,
    physical_device : PhysicalDevice,
    queue_info : QueueInfo,
    device : Device,
    swapchain_loader : Swapchain,
    swapchain : SwapchainKHR,
    swapchain_image_views : Vec<ImageView>,
    allocator : Allocator,
    depth_image : ImageAndView,
    render_pass : RenderPass,
    framebuffers : Vec<Framebuffer>,
    uniform_buffer : UniformBufferObject,
    descriptor_set_layout : DescriptorSetLayout,
    descriptor_pool : DescriptorPool,
    descriptor_sets : Vec<DescriptorSet>,
    pipeline_layout : PipelineLayout,
    pipeline_cache : PipelineCache,
    shaders : Vec<ShaderModule>,
    pipelines : Vec<Pipeline>,
    graphics_command_pool : CommandPool,
    graphics_queue : Queue,
    vertex_buffers : Vec<(u32,BufferAndAllocation)>,
    drawing_command_buffers : Vec<CommandBuffer>,
    synchronizer : Synchronizer,
    camera : Camera,
}
impl RenderOnThread{
    pub fn new(window : &Window, debug : bool) -> Self{
        let entry = unsafe{Entry::load()}.expect("Failed to load Vulkan drivers");
        let instance = unsafe{functions::instance::create_instance(&entry, window, debug)};
        let surface_loader = Surface::new(&entry, &instance);
        let surface = unsafe{ash_window::create_surface(&entry, &instance, window, None)}.expect("Failed to create window");
        let physical_device = functions::device::get_device_handle(&instance, &surface_loader, &surface);
        let device_limits = unsafe{instance.get_physical_device_properties(physical_device)}.limits;
        let queue_info = functions::device::QueueInfo::new(&instance, &surface_loader, &surface, physical_device);
        let device = unsafe{functions::device::create_device(&instance, physical_device, &queue_info)};
        let swapchain_loader = Swapchain::new(&instance, &device);
        let swapchain_info = functions::swapchain::SwapchainInfo::new(&instance, physical_device, &surface_loader, surface, window.inner_size());
        let swapchain = unsafe{functions::swapchain::create_swapchain(&swapchain_loader, &swapchain_info, &queue_info, surface)};
        let swapchain_images = unsafe{swapchain_loader.get_swapchain_images(swapchain)}.expect("Failed to get swapchain images");
        let swapchain_image_views = unsafe{functions::image::create_swapchain_image_views(&device, &swapchain_images, swapchain_info.surface_format)};
        let mut allocator = unsafe{allocator::Allocator::new(&instance, physical_device, device.clone())};
        let depth_image = unsafe{functions::image::create_depth_image(&device, &mut allocator, swapchain_info.extent, swapchain_info.depth_format)};
        let render_pass = unsafe{functions::render_pass::create_render_pass(&device, swapchain_info.surface_format, swapchain_info.depth_format)};
        let framebuffers = unsafe{functions::framebuffer::create_framebuffers(&device, &swapchain_image_views, depth_image.view, render_pass, swapchain_info.extent)};
        let uniform_buffer = unsafe{functions::buffer::create_uniform_buffers(&device, &mut allocator, swapchain_image_views.len() as u32, &device_limits)};
        let descriptor_set_layout = unsafe{functions::descriptor::create_descriptor_set_layout(&device)};
        let pipeline_layout = unsafe{functions::pipeline::create_pipeline_layout(&device, &descriptor_set_layout)};
        let pipeline_cache = unsafe{functions::pipeline::create_pipeline_cache(&device)};
        let descriptor_pool = unsafe{functions::descriptor::create_descriptor_pool(&device, swapchain_image_views.len() as u32)};
        let descriptor_sets = unsafe{functions::descriptor::create_descriptor_sets(&device, descriptor_set_layout, descriptor_pool, swapchain_image_views.len() as u32, uniform_buffer.buffer.buffer, &device_limits)};
        let synchronizer = unsafe{Synchronizer::new(&device, swapchain_image_views.len() as u32)};
        let shaders = unsafe{functions::shader::load_shaders(&device)};
        let pipelines = unsafe{functions::pipeline::create_pipelines(&device, pipeline_cache, pipeline_layout, render_pass, &shaders, swapchain_info.extent)};
        let graphics_command_pool = unsafe{functions::command::create_command_pool(&device, queue_info.graphics_family)};
        let graphics_queue = unsafe{device.get_device_queue(queue_info.graphics_family, 0)};
        let instance_positions = ModelMatrix::get_default();
        let vertex_buffers = unsafe{functions::vertex::create_vertex_buffers(&device, &mut allocator, graphics_command_pool, graphics_queue, instance_positions)};
        let drawing_command_buffers = unsafe{functions::command::create_drawing_command_buffers(&device, graphics_command_pool, pipeline_layout, &pipelines, render_pass, &framebuffers, &descriptor_sets, &vertex_buffers, swapchain_info.extent)};
        let camera = Camera::new(swapchain_info.extent);
        return Self{
            _entry:entry,instance,surface_loader,surface,physical_device,queue_info,device,swapchain_loader,swapchain,swapchain_image_views,allocator,depth_image,
            render_pass,shaders,framebuffers,uniform_buffer,descriptor_set_layout,pipeline_layout,pipeline_cache,descriptor_pool,descriptor_sets,pipelines,synchronizer,
            graphics_queue,graphics_command_pool,vertex_buffers,drawing_command_buffers,camera,
        }
    }
    pub fn draw(&mut self) -> bool{ 
        let wait_fences = [self.synchronizer.in_flight_fences[self.synchronizer.current_frame]];
        unsafe{
            self.device.wait_for_fences(&wait_fences, true, u64::MAX).expect("Failed to wait for fences");
        }
        let (image_index, suboptimal) = match unsafe{self.swapchain_loader.acquire_next_image(self.swapchain, u64::MAX, self.synchronizer.image_available_semaphores[self.synchronizer.current_frame], Fence::null())}{
            Ok(tuple)=>{tuple}
            Err(error)=>{
                match error{
                    ash::vk::Result::ERROR_OUT_OF_DATE_KHR=>{return true}
                    _=>{panic!("Failed to draw frame");}
                }
            }
        };
        self.camera.update();
        unsafe{self.update_uniform_buffer(image_index, self.camera.matrix)};
        let wait_semaphores = [self.synchronizer.image_available_semaphores[self.synchronizer.current_frame]];
        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let signal_semaphores = [self.synchronizer.render_finished_semaphores[self.synchronizer.current_frame]];
        let submit_infos = [SubmitInfo {
            s_type: StructureType::SUBMIT_INFO,
            p_next: std::ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.drawing_command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];
        unsafe{self.device.reset_fences(&wait_fences)}.expect("Failed to reset fences");
        unsafe{self.device.queue_submit(self.graphics_queue, &submit_infos, self.synchronizer.in_flight_fences[self.synchronizer.current_frame])}.expect("Failed to submit queue");
        let swapchains = [self.swapchain];
        let present_info = PresentInfoKHR {
            s_type: StructureType::PRESENT_INFO_KHR,
            p_next: std::ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: std::ptr::null_mut(),
        };
        self.synchronizer.current_frame = (self.synchronizer.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        match unsafe{self.swapchain_loader.queue_present(self.graphics_queue, &present_info)}{
            Ok(sub) => {return sub||suboptimal}
            Err(error) => {
                match error{
                    ash::vk::Result::ERROR_OUT_OF_DATE_KHR => {return true}
                    _=>{panic!("Queue present failed")}
                }
            }
        };
    }
    pub fn recreate_swapchain(&mut self, window_size : PhysicalSize<u32>){
        unsafe{self.destroy_swapchain()}
        let swapchain_info = functions::swapchain::SwapchainInfo::new(&self.instance, self.physical_device, &self.surface_loader, self.surface, window_size);
        self.swapchain = unsafe{functions::swapchain::create_swapchain(&self.swapchain_loader, &swapchain_info, &self.queue_info, self.surface)};
        let swapchain_images = unsafe{self.swapchain_loader.get_swapchain_images(self.swapchain).expect("Failed to get swapchain images")};
        self.swapchain_image_views = unsafe{functions::image::create_swapchain_image_views(&self.device, &swapchain_images, swapchain_info.surface_format)};
        self.depth_image = unsafe{functions::image::create_depth_image(&self.device, &mut self.allocator, swapchain_info.extent, swapchain_info.depth_format)};
        self.framebuffers = unsafe{functions::framebuffer::create_framebuffers(&self.device, &self.swapchain_image_views, self.depth_image.view, self.render_pass, swapchain_info.extent)};
        self.pipelines = unsafe{functions::pipeline::create_pipelines(&self.device, self.pipeline_cache, self.pipeline_layout, self.render_pass, &self.shaders, swapchain_info.extent)};
        self.drawing_command_buffers = unsafe{functions::command::create_drawing_command_buffers(&self.device, self.graphics_command_pool, self.pipeline_layout, &self.pipelines, self.render_pass, &self.framebuffers, &self.descriptor_sets, &self.vertex_buffers, swapchain_info.extent)};
        self.camera.correct_perspective(swapchain_info.extent);
    }
    unsafe fn update_uniform_buffer(&self, current_frame : u32, object : UniformBuffer){
        self.uniform_buffer.update_uniform_buffer(object, current_frame, &self.device);
    }
    unsafe fn destroy_swapchain(&mut self){
        self.device.device_wait_idle().expect("Failed to wait for device");
        self.device.free_command_buffers(self.graphics_command_pool, &self.drawing_command_buffers);
        for &pipeline in self.pipelines.iter(){
            self.device.destroy_pipeline(pipeline, None);
        }
        for &framebuffer in self.framebuffers.iter(){
            self.device.destroy_framebuffer(framebuffer, None);
        }
        self.depth_image.destroy(&mut self.allocator);
        for &image_view in self.swapchain_image_views.iter(){
            self.device.destroy_image_view(image_view, None);
        }
        self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }
}
impl Drop for RenderOnThread{
    fn drop(&mut self) {
        unsafe{
            self.device.device_wait_idle().expect("Failed to wait for device handle to finish");

            self.destroy_swapchain();
            for &shader in self.shaders.iter(){
                self.device.destroy_shader_module(shader, None);
            }
            functions::pipeline::save_pipeline_cache(&self.device, self.pipeline_cache);
            for buffer in self.vertex_buffers.iter(){
                buffer.1.destroy(&mut self.allocator);
            }
            self.synchronizer.destroy(&self.device);
            self.device.destroy_command_pool(self.graphics_command_pool, None);
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_pipeline_cache(self.pipeline_cache, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            self.uniform_buffer.buffer.destroy(&mut self.allocator);
            self.device.destroy_render_pass(self.render_pass, None);
            self.allocator.destroy();
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}