#[macro_use]
extern crate lazy_static;

use cgmath::{prelude::*, Point2, Vector3};
use entity::{Collider, ECS};
use graphics::{Camera, MeshManager, Renderer};
use specs::RunNow;
use std::collections::HashSet;
use winit::event;

pub const WIREFRAME_MODE: bool = false;

mod app;
mod block;
mod entity;
mod floor;
mod graphics;

struct AppState<'a: 'static> {
    renderer: Renderer,
    camera: Camera,
    ecs: entity::ECS<'a>,
    keys: Keys,
    window_size: Point2<f32>,
}

impl AppState<'_> {
    fn update_camera(&mut self) {
        let rotate_speed = 0.02;
        let move_speed = 0.16;

        if self.keys.is_key_down(event::VirtualKeyCode::Q) {
            self.camera.yaw += rotate_speed;
        } else if self.keys.is_key_down(event::VirtualKeyCode::E) {
            self.camera.yaw -= rotate_speed;
        }

        let forward_power = if self.keys.is_key_down(event::VirtualKeyCode::W) {
            1.0
        } else if self.keys.is_key_down(event::VirtualKeyCode::S) {
            -1.0
        } else {
            0.0
        };
        let side_power = if self.keys.is_key_down(event::VirtualKeyCode::D) {
            -1.0
        } else if self.keys.is_key_down(event::VirtualKeyCode::A) {
            1.0
        } else {
            0.0
        };

        let (yaw_sin, yaw_cos) = self.camera.yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, yaw_sin, 0.0).normalize() * forward_power * move_speed;
        let side = Vector3::new(-yaw_sin, yaw_cos, 0.0).normalize() * side_power * move_speed;
        self.camera.position += forward + side;
    }
}

impl<'a> app::Application for AppState<'a> {
    fn init(swapchain: &wgpu::SwapChainDescriptor, device: &wgpu::Device, _: &wgpu::Queue) -> Self {
        let mut mesh_manager = MeshManager::new();
        let renderer = Renderer::new(device, &swapchain);
        let blocks = block::load_blocks(device, &mut mesh_manager);
        let floors = floor::load_floors(device, &mut mesh_manager);
        let camera = Camera {
            position: (-12.0, 0.0, 12.0).into(),
            yaw: 0.0,
            pitch: -1.0,
            aspect: swapchain.width as f32 / swapchain.height as f32,
            fov: 45.0,
            near: 0.1,
            far: 100.0,
        };

        let ecs = ECS::new(device, mesh_manager, blocks, floors);
        let keys = Keys(HashSet::new());
        let window_size = Point2::new(swapchain.width as f32, swapchain.height as f32);

        AppState {
            renderer,
            camera,
            ecs,
            keys,
            window_size,
        }
    }

    fn resize(
        &mut self,
        swapchain: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
        _: &wgpu::Queue,
    ) {
        self.camera.resize(swapchain);
        self.renderer.resize(device, swapchain);
        self.window_size = Point2::new(swapchain.width as f32, swapchain.height as f32);
    }

    fn key_event(&mut self, key: event::VirtualKeyCode, state: event::ElementState) {
        match state {
            event::ElementState::Pressed => self.keys.0.insert(key),
            event::ElementState::Released => self.keys.0.remove(&key),
        };
    }

    fn scroll_event(&mut self, _: f32) {}

    fn mouse_moved(&mut self, _: Point2<f32>) {}

    fn click_event(&mut self, _: event::MouseButton, state: event::ElementState, pt: Point2<f32>) {
        if state != event::ElementState::Pressed {
            return;
        }

        let near = self
            .camera
            .unproject(Vector3::new(pt.x, pt.y, 0.0), self.window_size);
        let far = self
            .camera
            .unproject(Vector3::new(pt.x, pt.y, 1.0), self.window_size);

        let mut raycast_system = entity::physics::RaycastSystem::new();
        raycast_system.run_now(&mut self.ecs.world);

        if let Some(asteroid) = raycast_system.raycast(vec![Collider::ASTEROID], near, far) {
            self.ecs.mark_for_removal(asteroid);
            entity::objects::create_asteroid(&mut self.ecs.world);
            self.ecs.maintain();
        }
    }

    fn fixed_update(&mut self, _: &wgpu::Device, _: &wgpu::Queue) {
        self.update_camera();
        self.ecs.update();
    }

    fn render(
        &mut self,
        texture: &wgpu::SwapChainTexture,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mut mesh_manager = self.ecs.world.fetch_mut::<MeshManager>();
        self.renderer
            .render(device, queue, texture, &self.camera, &mut mesh_manager)
    }
}

fn main() {
    app::run::<AppState>("Spaceship Alpha");
}

struct Keys(HashSet<event::VirtualKeyCode>);

impl Keys {
    fn is_key_down(&self, key: event::VirtualKeyCode) -> bool {
        self.0.contains(&key)
    }
}

#[allow(dead_code)]
pub fn print_time(title: &str) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        % 1000;

    println!("{}: {}", title, time_ms);
}
