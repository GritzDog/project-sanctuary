use winit::{
    event::{ElementState, MouseButton},
    keyboard::KeyCode,
};
use cgmath::{Matrix4, Deg, perspective, Point3, Vector3, InnerSpace};
use std::collections::HashSet;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view_pos: [f32; 3],
    pub _padding: f32,
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
            view_pos: [0.0, 0.0, 0.0],
            _padding: 0.0,
        }
    }

    pub fn update_view_proj(&mut self, camera_controller: &CameraController) {
        self.view_proj = camera_controller.build_view_projection_matrix().into();
        self.view_pos = [camera_controller.eye.x, camera_controller.eye.y, camera_controller.eye.z];
    }
}

#[derive(Debug)]
pub struct CameraController {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fov: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    
    // Controls
    forward_speed: f32,
    rotation_speed: f32,
    zoom_speed: f32,
    
    // Input state
    keys_pressed: HashSet<KeyCode>,
    mouse_pressed: bool,
    shift_pressed: bool,
    pub last_mouse_pos: (f64, f64),
}

impl CameraController {
    pub fn new(aspect: f32) -> Self {
        Self {
            eye: Point3::new(-10.0, 60.0, 30.0),
            target: Point3::new(15.0, 0.0, 0.0),
            up: Vector3::unit_y(),
            fov: 45.0,
            aspect,
            znear: 0.1,
            zfar: 2000.0,  // Increased far plane for better visibility
            
            forward_speed: 5.0,
            rotation_speed: 0.015,
            zoom_speed: 2.0,
            
            keys_pressed: HashSet::new(),
            mouse_pressed: false,
            shift_pressed: false,
            last_mouse_pos: (0.0, 0.0),
        }
    }
    
    pub fn update_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }
    
    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                self.keys_pressed.insert(key);
                if key == KeyCode::ShiftLeft || key == KeyCode::ShiftRight {
                    self.shift_pressed = true;
                }
            }
            ElementState::Released => {
                self.keys_pressed.remove(&key);
                if key == KeyCode::ShiftLeft || key == KeyCode::ShiftRight {
                    self.shift_pressed = false;
                }
            }
        }
    }
    
    pub fn process_mouse_motion(&mut self, delta_x: f64, delta_y: f64) {
        if self.mouse_pressed {
            let dx = delta_x as f32;
            let dy = delta_y as f32;
            
            if self.shift_pressed {
                // Panning mode
                let pan_speed = 0.05;
                let forward = (self.target - self.eye).normalize();
                let right = forward.cross(self.up).normalize();
                let up = right.cross(forward).normalize();
                
                let pan_movement = right * (-dx * pan_speed) + up * (dy * pan_speed);
                
                self.eye += pan_movement;
                self.target += pan_movement;
            } else {
                // Rotation mode
                let dx = dx * self.rotation_speed;
                let dy = dy * self.rotation_speed;
                
                let radius = (self.eye - self.target).magnitude();
                let theta = dx;
                let phi = dy;
                
                let forward = (self.target - self.eye).normalize();
                let right = forward.cross(self.up).normalize();
                let up = right.cross(forward).normalize();
                
                let new_forward = Matrix4::from_axis_angle(up, Deg(-theta)) * 
                                 Matrix4::from_axis_angle(right, Deg(-phi)) * 
                                 forward.extend(0.0);

                self.eye = self.target - Vector3::new(new_forward.x, new_forward.y, new_forward.z) * radius;
            }
        }
    }
    
    pub fn process_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        if button == MouseButton::Left {
            self.mouse_pressed = state == ElementState::Pressed;
        }
    }
    
    pub fn process_scroll(&mut self, delta: f32) {
        let forward = (self.target - self.eye).normalize();
        self.eye += forward * delta * self.zoom_speed;
        
        // Prevent getting too close
        let distance = (self.eye - self.target).magnitude();
        if distance < 5.0 {
            self.eye = self.target - forward * 5.0;
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        let up = right.cross(forward).normalize();
        
        let mut movement = Vector3::new(0.0, 0.0, 0.0);
        
        // WASD movement
        if self.keys_pressed.contains(&KeyCode::KeyW) {
            movement += forward;
        }
        if self.keys_pressed.contains(&KeyCode::KeyS) {
            movement -= forward;
        }
        if self.keys_pressed.contains(&KeyCode::KeyA) {
            movement -= right;
        }
        if self.keys_pressed.contains(&KeyCode::KeyD) {
            movement += right;
        }
        if self.keys_pressed.contains(&KeyCode::KeyQ) {
            movement += up;
        }
        if self.keys_pressed.contains(&KeyCode::KeyE) {
            movement -= up;
        }
        
        // Apply movement
        if movement.magnitude() > 0.0 {
            movement = movement.normalize() * self.forward_speed * dt;
            self.eye += movement;
            self.target += movement;
        }
    }
    
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fov), self.aspect, self.znear, self.zfar);
        proj * view
    }
    
    pub fn reset(&mut self) {
        *self = Self::new(self.aspect);
    }
}