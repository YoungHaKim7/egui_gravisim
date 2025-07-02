use eframe::{App, Frame, egui};
use egui::{Color32, Pos2, Vec2};
use nalgebra::Vector2;
use rand::Rng;

const G: f32 = 0.0005;

#[derive(Clone)]
struct Body {
    pos: Vector2<f32>,
    vel: Vector2<f32>,
    mass: f32,
    radius: f32,
    color: Color32,
}

impl Body {
    fn new(pos: Vector2<f32>, vel: Vector2<f32>, density: f32, size: f32) -> Self {
        let radius = size;
        let mass = density * radius * radius;
        Self {
            pos,
            vel,
            mass,
            radius,
            color: Color32::from_rgb(200, 200, 255),
        }
    }

    fn apply_gravity(&mut self, other: &Body) {
        let dir = other.pos - self.pos;
        let dist_sq = dir.norm_squared();
        if dist_sq < 1.0 {
            return;
        }
        let force_mag = G * self.mass * other.mass / dist_sq;
        let force = dir.normalize() * force_mag;
        self.vel += force / self.mass;
    }

    fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
    }
}

struct GravisimApp {
    bodies: Vec<Body>,
    camera_pos: Vector2<f32>,
    zoom: f32,
    selected_size: f32,
    selected_density: f32,
    selected_pos: Option<Vector2<f32>>,
    show_hud: bool,
    elastic: bool,
}

impl Default for GravisimApp {
    fn default() -> Self {
        Self {
            bodies: Vec::new(),
            camera_pos: Vector2::new(0.0, 0.0),
            zoom: 1.0,
            selected_size: 50.0,
            selected_density: 1.0,
            selected_pos: None,
            show_hud: true,
            elastic: false,
        }
    }
}

fn nalgebra_from_vec2(v: Vec2) -> Vector2<f32> {
    Vector2::new(v.x, v.y)
}

impl App for GravisimApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        let input = ctx.input(|i| i.clone());
        let dt = input.stable_dt;

        // Handle input
        if input.key_pressed(egui::Key::R) {
            self.bodies.clear();
            self.camera_pos = Vector2::new(0.0, 0.0);
            self.zoom = 1.0;
        }
        if input.key_pressed(egui::Key::H) {
            self.show_hud = !self.show_hud;
        }
        if input.key_pressed(egui::Key::E) {
            self.elastic = !self.elastic;
        }

        // Pan
        let pan_speed = 300.0 * dt / self.zoom;
        if input.key_down(egui::Key::W) {
            self.camera_pos.y -= pan_speed;
        }
        if input.key_down(egui::Key::S) {
            self.camera_pos.y += pan_speed;
        }
        if input.key_down(egui::Key::A) {
            self.camera_pos.x -= pan_speed;
        }
        if input.key_down(egui::Key::D) {
            self.camera_pos.x += pan_speed;
        }

        // Zoom
        self.zoom *= (1.0 + input.raw_scroll_delta.y * 0.1).clamp(0.1, 10.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
            let painter = ui.painter();
            painter.rect_filled(rect, 0.0, Color32::BLACK);

            let center = rect.center();
            let center_vec = nalgebra_from_vec2(center.to_vec2());

            // Gravity
            for i in 0..self.bodies.len() {
                let (left, right) = self.bodies.split_at_mut(i + 1);
                let (this, others) = left.split_last_mut().unwrap();
                for other in right {
                    this.apply_gravity(other);
                }
            }
            // for i in 0..self.bodies.len() {
            //     for j in 0..self.bodies.len() {
            //         if i == j {
            //             continue;
            //         }
            //         let other = &self.bodies[j];
            //         self.bodies[i].apply_gravity(other);
            //     }
            // }

            // Update
            for body in &mut self.bodies {
                body.update(dt);
            }

            // Mouse world pos
            let mouse_pos = input.pointer.hover_pos().unwrap_or(center);
            let mouse_vec = nalgebra_from_vec2(mouse_pos.to_vec2());
            let world_mouse = (mouse_vec - center_vec) / self.zoom + self.camera_pos;

            // Handle placing body
            if input.pointer.any_pressed() && self.selected_pos.is_none() {
                self.selected_pos = Some(world_mouse);
            }

            if input.pointer.any_released() {
                if let Some(start) = self.selected_pos.take() {
                    let end = world_mouse;
                    let vel = (end - start) / 20.0;
                    self.bodies.push(Body::new(
                        start,
                        vel,
                        self.selected_density,
                        self.selected_size,
                    ));
                }
            }

            // Render bodies
            for body in &self.bodies {
                let screen_vec = (body.pos - self.camera_pos) * self.zoom + center_vec;
                let screen_pos = Pos2::new(screen_vec.x, screen_vec.y);
                painter.circle_filled(screen_pos, body.radius * self.zoom, body.color);
            }

            // Render selected circle
            if let Some(_) = self.selected_pos {
                let screen_vec = (world_mouse - self.camera_pos) * self.zoom + center_vec;
                let screen_pos = Pos2::new(screen_vec.x, screen_vec.y);
                painter.circle_stroke(
                    screen_pos,
                    self.selected_size * self.zoom,
                    (1.0, Color32::LIGHT_GREEN),
                );
            }

            if self.show_hud {
                egui::Window::new("HUD").show(ctx, |ui| {
                    ui.label(format!("Bodies: {}", self.bodies.len()));
                    ui.label(format!("Zoom: {:.2}", self.zoom));
                    ui.label(format!("Elastic Collisions: {}", self.elastic));
                    ui.label(
                        "Controls:\n\
                        R: Reset\n\
                        H: Toggle HUD\n\
                        E: Toggle Elastic\n\
                        WASD: Pan\n\
                        Scroll: Zoom\n\
                        Click-Drag: Spawn",
                    );
                });
            }

            ctx.request_repaint();
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Gravisim (Egui)",
        options,
        Box::new(|_cc| Ok(Box::new(GravisimApp::default()))),
    )
}
