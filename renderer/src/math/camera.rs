use ash::vk::Extent2D;
use cgmath::{Matrix4, SquareMatrix, Deg, Point3, Vector3, Quaternion, InnerSpace};

use super::UniformBuffer;

pub struct Camera{
    model : Matrix4<f32>,
    view : Matrix4<f32>,
    projection : Matrix4<f32>,
    near : f32,
    far : f32,
    fov : f32,
    aspect : f32,
    eye : Point3<f32>,
    center : Point3<f32>,
    up : Vector3<f32>,
    pub matrix : UniformBuffer,
    pub left_mouse_button_pressed : bool,
}
impl Camera{
    pub fn new(extent : Extent2D) -> Self{
        let near = 0.1;
        let far = 10.0;
        let fov = 70.0;
        let aspect = extent.width as f32 / extent.height as f32;
        let projection = cgmath::perspective(
            Deg(fov), 
            aspect, 
            near, 
            far,
        );
        let eye = Point3::new(2.0, 2.0, 2.0);
        let center = Point3::new(0.0, 0.0, 0.0);
        let up = Vector3::new(0.0, 0.0, 1.0);
        let view = Matrix4::look_at_rh(eye,center, up);
        let model = Matrix4::identity();
        return Self{
            matrix : UniformBuffer{matrix:projection*view*model},
            projection,
            view,
            model,
            near,
            far,
            fov,
            aspect,
            eye,
            center,
            up,
            left_mouse_button_pressed : false,
        }
    }
    pub fn correct_perspective(&mut self, extent : Extent2D){
        self.aspect = extent.width as f32 / extent.height as f32;
        self.projection = cgmath::perspective(Deg(self.fov), self.aspect, self.near, self.far);
    }
    pub fn update(&mut self){
        self.matrix.matrix=self.projection*self.view*self.model;
    }
    pub fn mouse_movement(&mut self, delta : (f64,f64)){
        if self.left_mouse_button_pressed{
            let alpha = Quaternion::new(0.0, self.eye.x-self.center.x, self.eye.y-self.center.y,self.eye.z-self.center.z).normalize();
            let sin_value = (delta.0 as f32)*0.2;
            let cos_value = (1.0-sin_value.powf(2.0)).sqrt();
            let mut q = sin_value * alpha;
            q.s += cos_value;
            let beta = Quaternion::new(0.0, self.eye.x, self.eye.y, self.eye.z);
            let eye = (q*beta*q.conjugate()).v;
            println!("{:?}",eye);
            self.eye = Point3::new(eye.x, eye.y, eye.z);
            self.view = Matrix4::look_at_rh(self.eye,self.center, self.up);
        }
    }
    pub fn mouse_zoom(&mut self, delta : f32){
        if self.fov + delta/100.0 < 180.0 && self.fov + delta/100.0 > 0.0{
            self.fov += delta/100.0;
        };
        self.projection = cgmath::perspective(Deg(self.fov), self.aspect, self.near, self.far);
    }
}