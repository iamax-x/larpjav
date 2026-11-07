//! larp-jv-desktop — Desktop frontend for larp.jv

use larp_jv_core::{AuthManager, Downloader, InstanceManager, ModManager, JreManager, MoJsonManager};
use std::sync::Arc;
use winit::event_loop::{EventLoop, ControlFlow};

mod platform;
mod ui;
mod render;

fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Initialize core services
    let auth_manager = Arc::new(AuthManager::new());
    let downloader = Arc::new(larp_jv_core::Downloader::new()?);
    let instance_manager = Arc::new(InstanceManager::new("instances")?);
    let mod_manager = Arc::new(ModManager::new()?);
    let jre_manager = Arc::new(JreManager::new("jres")?);
    let mojson_manager = Arc::new(MoJsonManager::new("mojson")?);

    // Create event loop
    let event_loop = EventLoop::new()?;

    // Create window
    let window = winit::window::WindowBuilder::new()
        .with_title("larp.jv")
        .with_inner_size(winit::dpi::LogicalSize::new(1024, 768))
        .build(&event_loop)?;

    // Initialize wgpu
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = unsafe { instance.create_surface(&window) }?;

    // Initialize egui
    let egui_ctx = egui::Context::default();
    let mut egui_winit = egui_winit::State::new(
        egui::Context::default(),
        egui::ViewportId::ROOT,
        &event_loop,
        Some(window.scale_factor() as f32),
        None,
    );

    // Initialize wgpu renderer
    let renderer = pollster::block_on(async {
        larp_jv_android::render::GameRenderer::new(&instance, surface, 1024, 768).await
    })?;

    // UI state
    let mut ui_state = larp_jv_android::ui::LauncherUi::new(
        auth_manager,
        downloader,
        instance_manager,
        mod_manager,
        jre_manager,
        mojson_manager,
    );

    // Run event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        let _ = egui_winit.on_window_event(&window, &event);

        match event {
            winit::event::Event::RedrawRequested(_) => {
                let input = egui_winit.take_egui_input(&window);
                let output = egui_ctx.run(input, |ctx| {
                    ui_state.update(ctx);
                });
                egui_winit.handle_platform_output(&window, output.platform_output);

                let paint_jobs = egui_ctx.tessellate(output.shapes, output.pixels_per_point);
                let mut renderer = renderer.lock().unwrap();
                renderer.render(&paint_jobs, &output.textures_delta).unwrap();

                window.request_redraw();
            }
            winit::event::Event::WindowEvent { event: winit::event::WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    })?;
}