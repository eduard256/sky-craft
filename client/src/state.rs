// Application state and winit event handling with egui UI.

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId, WindowAttributes};
use tracing::{info, warn};

use crate::renderer::Renderer;
use crate::input::InputState;
use crate::world::ClientWorld;
use crate::session;
use crate::auth_client;
use crate::net_bridge::NetBridge;

/// Application state machine.
#[derive(Debug, Clone, PartialEq)]
pub enum AppScreen {
    MainMenu,
    Login,
    Connecting,
    Playing,
}

#[derive(Debug, Clone, PartialEq)]
enum LoginStep {
    EnterNickname,
    EnterCode,
}

/// UI-only state, separated to avoid borrow conflicts with egui.
struct UiState {
    screen: AppScreen,
    server_address: String,
    nickname: String,
    auth_code: String,
    status_message: String,
    saved_token: Option<String>,
    login_step: LoginStep,
}

/// Action returned from UI drawing (deferred execution after egui frame).
enum UiAction {
    None,
    RequestCode,
    VerifyCode,
    Connect,
    Quit,
    Logout,
    GoToLogin,
    GoToMainMenu,
    LoginBackToNickname,
}

pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    input: InputState,
    world: ClientWorld,

    egui_ctx: egui::Context,
    egui_state: Option<egui_winit::State>,
    egui_renderer: Option<egui_wgpu::Renderer>,

    ui: UiState,

    /// Network bridge (alive while connected to server).
    net: Option<NetBridge>,
}

impl App {
    pub fn new() -> Self {
        let (saved_token, nickname) = match session::load_session() {
            Some(s) => {
                info!("Loaded saved session for {}", s.nickname);
                (Some(s.token), s.nickname)
            }
            None => (None, String::new()),
        };

        Self {
            window: None,
            renderer: None,
            input: InputState::new(),
            world: ClientWorld::new(),
            egui_ctx: egui::Context::default(),
            egui_state: None,
            egui_renderer: None,
            net: None,
            ui: UiState {
                screen: AppScreen::MainMenu,
                server_address: "127.0.0.1".to_string(),
                nickname,
                auth_code: String::new(),
                status_message: String::new(),
                saved_token,
                login_step: LoginStep::EnterNickname,
            },
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        info!("Creating window...");
        let attrs = WindowAttributes::default()
            .with_title("Sky Craft")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

        match event_loop.create_window(attrs) {
            Ok(window) => {
                let window = Arc::new(window);

                let renderer = pollster::block_on(Renderer::new(window.clone()));
                match renderer {
                    Ok(r) => {
                        let egui_state = egui_winit::State::new(
                            self.egui_ctx.clone(),
                            egui::ViewportId::ROOT,
                            &window,
                            Some(window.scale_factor() as f32),
                            None,
                            None,
                        );

                        let egui_renderer = egui_wgpu::Renderer::new(
                            r.device(),
                            r.surface_format(),
                            None,
                            1,
                            false,
                        );

                        self.egui_state = Some(egui_state);
                        self.egui_renderer = Some(egui_renderer);
                        self.renderer = Some(r);
                        info!("Renderer + egui initialized");
                    }
                    Err(e) => {
                        warn!("Failed to init renderer: {}", e);
                        event_loop.exit();
                        return;
                    }
                }
                self.window = Some(window);
            }
            Err(e) => {
                warn!("Failed to create window: {}", e);
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(egui_state) = &mut self.egui_state {
            let response = egui_state.on_window_event(self.window.as_ref().unwrap(), &event);
            if response.consumed {
                return;
            }
        }

        self.input.handle_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                info!("Window close requested");
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(new_size);
                }
            }
            WindowEvent::RedrawRequested => {
                self.do_frame();
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn do_frame(&mut self) {
        let Some(window) = &self.window else { return };
        let Some(egui_state) = &mut self.egui_state else { return };
        let Some(renderer) = &mut self.renderer else { return };
        let Some(egui_renderer) = &mut self.egui_renderer else { return };

        // Drain network packets into world (if connected)
        if let Some(ref net) = self.net {
            for packet in net.drain_packets() {
                self.world.handle_server_packet(packet);
            }
        }

        // Begin egui frame
        let raw_input = egui_state.take_egui_input(window);
        self.egui_ctx.begin_pass(raw_input);

        // Draw UI -- only reads/writes ui state, returns deferred action
        let action = draw_ui(&self.egui_ctx, &mut self.ui, &self.world);

        // End egui frame
        let full_output = self.egui_ctx.end_pass();

        egui_state.handle_platform_output(window, full_output.platform_output);

        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, full_output.pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [renderer.size().width, renderer.size().height],
            pixels_per_point: full_output.pixels_per_point,
        };

        for (id, delta) in &full_output.textures_delta.set {
            egui_renderer.update_texture(renderer.device(), renderer.queue(), *id, delta);
        }

        let mut encoder = renderer.create_encoder();
        egui_renderer.update_buffers(
            renderer.device(),
            renderer.queue(),
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        renderer.queue().submit(std::iter::once(encoder.finish()));

        // Render
        match renderer.render_with_egui(egui_renderer, &paint_jobs, &screen_descriptor, &self.ui.screen) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => {
                renderer.resize(window.inner_size());
            }
            Err(wgpu::SurfaceError::OutOfMemory) => {
                warn!("Out of GPU memory");
            }
            Err(e) => {
                warn!("Render error: {:?}", e);
            }
        }

        for id in &full_output.textures_delta.free {
            egui_renderer.free_texture(id);
        }

        // Execute deferred action (after egui frame is done)
        self.execute_action(action);
    }

    fn execute_action(&mut self, action: UiAction) {
        match action {
            UiAction::None => {}
            UiAction::Quit => std::process::exit(0),
            UiAction::Logout => {
                session::delete_session();
                self.ui.saved_token = None;
                self.ui.nickname.clear();
                self.ui.status_message = "Logged out".to_string();
            }
            UiAction::GoToLogin => {
                self.ui.screen = AppScreen::Login;
                self.ui.login_step = LoginStep::EnterNickname;
                self.ui.status_message.clear();
            }
            UiAction::GoToMainMenu => {
                self.ui.screen = AppScreen::MainMenu;
                self.ui.status_message.clear();
            }
            UiAction::LoginBackToNickname => {
                self.ui.login_step = LoginStep::EnterNickname;
                self.ui.auth_code.clear();
                self.ui.status_message.clear();
            }
            UiAction::RequestCode => {
                if self.ui.nickname.len() < 3 {
                    self.ui.status_message = "Nickname must be at least 3 characters".to_string();
                    return;
                }
                self.ui.status_message = "Requesting code...".to_string();
                match auth_client::request_code(&self.ui.nickname) {
                    Ok(msg) => {
                        self.ui.status_message = msg;
                        self.ui.login_step = LoginStep::EnterCode;
                    }
                    Err(e) => {
                        self.ui.status_message = e;
                    }
                }
            }
            UiAction::VerifyCode => {
                if self.ui.auth_code.len() != 6 {
                    self.ui.status_message = "Code must be 6 digits".to_string();
                    return;
                }
                self.ui.status_message = "Verifying...".to_string();
                match auth_client::verify_code(&self.ui.nickname, &self.ui.auth_code) {
                    Ok(token) => {
                        let sess = session::Session {
                            nickname: self.ui.nickname.clone(),
                            token: token.clone(),
                        };
                        let _ = session::save_session(&sess);
                        self.ui.saved_token = Some(token);
                        self.ui.status_message = "Login successful!".to_string();
                        info!("Logged in as {}", self.ui.nickname);
                        self.ui.screen = AppScreen::Connecting;
                        self.try_connect();
                    }
                    Err(e) => {
                        self.ui.status_message = e;
                    }
                }
            }
            UiAction::Connect => {
                if self.ui.saved_token.is_some() {
                    self.ui.screen = AppScreen::Connecting;
                    self.try_connect();
                } else {
                    self.ui.screen = AppScreen::Login;
                    self.ui.login_step = LoginStep::EnterNickname;
                }
            }
        }
    }

    fn try_connect(&mut self) {
        let token = match &self.ui.saved_token {
            Some(t) => t.clone(),
            None => {
                self.ui.status_message = "No auth token. Please login first.".to_string();
                self.ui.screen = AppScreen::Login;
                return;
            }
        };

        let addr = if self.ui.server_address.contains(':') {
            self.ui.server_address.clone()
        } else {
            format!("{}:{}", self.ui.server_address, skycraft_protocol::DEFAULT_PORT)
        };

        self.ui.status_message = format!("Connecting to {}...", addr);

        match NetBridge::connect(addr, token) {
            Ok((bridge, login_success)) => {
                info!("Connected! Player: {} UUID: {}", login_success.nickname, login_success.player_uuid);
                self.ui.nickname = login_success.nickname;
                self.ui.status_message.clear();
                self.ui.screen = AppScreen::Playing;

                // Store the bridge -- keeps connection alive, packets flow in background
                self.net = Some(bridge);
            }
            Err(e) => {
                let err_msg = e.to_string();
                warn!("Connection failed: {}", err_msg);

                if err_msg.contains("Invalid session") || err_msg.contains("401") {
                    session::delete_session();
                    self.ui.saved_token = None;
                    self.ui.status_message = "Session expired. Please login again.".to_string();
                    self.ui.screen = AppScreen::Login;
                } else {
                    self.ui.status_message = format!("Failed: {}", err_msg);
                    self.ui.screen = AppScreen::MainMenu;
                }
            }
        }
    }
}

// ─── UI Drawing (pure functions, no &mut self) ──────────────────────────────

fn draw_ui(ctx: &egui::Context, ui: &mut UiState, world: &ClientWorld) -> UiAction {
    match ui.screen {
        AppScreen::MainMenu => draw_main_menu(ctx, ui),
        AppScreen::Login => draw_login(ctx, ui),
        AppScreen::Connecting => draw_connecting(ctx, ui),
        AppScreen::Playing => draw_playing(ctx, ui, world),
    }
}

fn draw_main_menu(ctx: &egui::Context, ui: &mut UiState) -> UiAction {
    let mut action = UiAction::None;

    egui::CentralPanel::default().show(ctx, |panel| {
        panel.vertical_centered(|panel| {
            panel.add_space(100.0);
            panel.heading(egui::RichText::new("SKY CRAFT").size(48.0).strong());
            panel.add_space(40.0);

            panel.label("Server address:");
            panel.add_space(4.0);
            panel.add(
                egui::TextEdit::singleline(&mut ui.server_address)
                    .hint_text("ip or ip:port")
                    .desired_width(300.0),
            );
            panel.add_space(16.0);

            if panel.add_sized([200.0, 40.0], egui::Button::new("Connect")).clicked() {
                action = UiAction::Connect;
            }

            panel.add_space(8.0);
            if panel.add_sized([200.0, 30.0], egui::Button::new("Quit")).clicked() {
                action = UiAction::Quit;
            }

            if !ui.status_message.is_empty() {
                panel.add_space(20.0);
                panel.colored_label(egui::Color32::YELLOW, &ui.status_message);
            }

            if ui.saved_token.is_some() {
                panel.add_space(20.0);
                panel.colored_label(egui::Color32::GREEN, format!("Logged in as: {}", ui.nickname));
                if panel.small_button("Logout").clicked() {
                    action = UiAction::Logout;
                }
            }

            panel.add_space(40.0);
            panel.colored_label(egui::Color32::DARK_GRAY, format!("v{}", env!("CARGO_PKG_VERSION")));
        });
    });

    action
}

fn draw_login(ctx: &egui::Context, ui: &mut UiState) -> UiAction {
    let mut action = UiAction::None;

    egui::CentralPanel::default().show(ctx, |panel| {
        panel.vertical_centered(|panel| {
            panel.add_space(100.0);
            panel.heading(egui::RichText::new("Login").size(36.0));
            panel.add_space(8.0);
            panel.label("Register via Telegram bot: @skycraftauth_bot");
            panel.add_space(30.0);

            match ui.login_step {
                LoginStep::EnterNickname => {
                    panel.label("Nickname:");
                    panel.add_space(4.0);
                    panel.add(
                        egui::TextEdit::singleline(&mut ui.nickname)
                            .hint_text("your nickname")
                            .desired_width(300.0),
                    );
                    panel.add_space(16.0);

                    if panel.add_sized([200.0, 40.0], egui::Button::new("Request Code")).clicked() {
                        action = UiAction::RequestCode;
                    }
                }
                LoginStep::EnterCode => {
                    panel.label(format!("Code sent to Telegram for: {}", ui.nickname));
                    panel.add_space(12.0);
                    panel.label("Enter 6-digit code:");
                    panel.add_space(4.0);
                    panel.add(
                        egui::TextEdit::singleline(&mut ui.auth_code)
                            .hint_text("000000")
                            .desired_width(200.0),
                    );
                    panel.add_space(16.0);

                    if panel.add_sized([200.0, 40.0], egui::Button::new("Verify")).clicked() {
                        action = UiAction::VerifyCode;
                    }

                    if panel.small_button("Back").clicked() {
                        action = UiAction::LoginBackToNickname;
                    }
                }
            }

            if !ui.status_message.is_empty() {
                panel.add_space(20.0);
                panel.colored_label(egui::Color32::YELLOW, &ui.status_message);
            }

            panel.add_space(20.0);
            if panel.small_button("Back to menu").clicked() {
                action = UiAction::GoToMainMenu;
            }
        });
    });

    action
}

fn draw_connecting(ctx: &egui::Context, ui: &mut UiState) -> UiAction {
    let mut action = UiAction::None;

    egui::CentralPanel::default().show(ctx, |panel| {
        panel.vertical_centered(|panel| {
            panel.add_space(200.0);
            panel.heading("Connecting...");
            panel.add_space(16.0);
            panel.spinner();
            panel.add_space(16.0);

            if !ui.status_message.is_empty() {
                panel.colored_label(egui::Color32::YELLOW, &ui.status_message);
            }

            panel.add_space(20.0);
            if panel.button("Cancel").clicked() {
                action = UiAction::GoToMainMenu;
            }
        });
    });

    action
}

fn draw_playing(ctx: &egui::Context, ui: &mut UiState, world: &ClientWorld) -> UiAction {
    egui::TopBottomPanel::top("top_hud").show(ctx, |panel| {
        panel.horizontal(|panel| {
            panel.label(egui::RichText::new("Sky Craft").strong());
            panel.separator();
            panel.label(format!("Player: {}", ui.nickname));
            panel.separator();
            panel.label(format!(
                "Chunks: {} | HP: {:.0} | Food: {}",
                world.loaded_chunk_count(),
                world.player.health,
                world.player.food,
            ));
            panel.separator();
            panel.label(format!(
                "Pos: {:.1} {:.1} {:.1}",
                world.player.position.x,
                world.player.position.y,
                world.player.position.z,
            ));
            panel.separator();
            panel.label(format!("Ring: {}", world.current_ring));
        });
    });

    UiAction::None
}
