use eframe::egui;
use egui::ColorImage;
use image::{GenericImageView, DynamicImage};

pub struct DiffApp {
    texture: Option<egui::TextureHandle>,
    image: DynamicImage,
    centers: Vec<(u32, u32)>,
    current_index: usize,
    zoom: f32,
    offset: egui::Vec2,
}

impl DiffApp {
    pub fn new(_cc: &eframe::CreationContext, image: DynamicImage, centers: Vec<(u32, u32)>) -> Self {
        Self {
            texture: None,
            image,
            centers,
            current_index: 0,
            zoom: 6.0,
            offset: egui::Vec2::ZERO,
        }
    }

    fn center_on(&mut self, panel_size: egui::Vec2, x: u32, y: u32) {
        self.offset = egui::Vec2::new(
            x as f32 - panel_size.x / (2.0 * self.zoom),
            y as f32 - panel_size.y / (2.0 * self.zoom),
        );
    }
}

// main gui stuff. Handles displaying the image, zooming, panning, and showing the red island centers.
impl eframe::App for DiffApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.texture.is_none() {
            let (w, h) = self.image.dimensions();
            let rgba = self.image.to_rgba8();
            let pixels: Vec<egui::Color32> = rgba
                .pixels()
                .map(|p| egui::Color32::from_rgba_unmultiplied(p.0[0], p.0[1], p.0[2], p.0[3]))
                .collect();
            let color_image = ColorImage {
                size: [w as usize, h as usize],
                pixels,
            };
            self.texture = Some(ctx.load_texture(
                "diff",
                color_image,
                egui::TextureOptions::NEAREST,
            ));
        }

        egui::TopBottomPanel::bottom("controls").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let total = self.centers.len();
                let label = if total == 0 {
                    "No islands found".to_string()
                } else {
                    format!("Island {} / {}", self.current_index + 1, total)
                };
                ui.label(label);

                ui.separator();

                let prev_clicked = ui.button("◀  Prev").clicked();
                let next_clicked = ui.button("Next  ▶").clicked();

                let left_key  = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
                let right_key = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));

                let panel_size = ctx.available_rect().size();

                if !self.centers.is_empty() {
                    if (prev_clicked || left_key) && self.current_index > 0 {
                        self.current_index -= 1;
                        let (cx, cy) = self.centers[self.current_index];
                        self.center_on(panel_size, cx, cy);
                    }
                    if (next_clicked || right_key) && self.current_index + 1 < self.centers.len() {
                        self.current_index += 1;
                        let (cx, cy) = self.centers[self.current_index];
                        self.center_on(panel_size, cx, cy);
                    }
                }

                ui.separator();
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 1.0..=20.0).fixed_decimals(1));

                if self.centers.is_empty() {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(80, 200, 80), "Images are identical!");
                }
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture {
                let panel = ui.available_size();
                let (img_w, img_h) = self.image.dimensions();

                let (drag_rect, drag_response) =
                    ui.allocate_exact_size(panel, egui::Sense::drag());

                if drag_response.dragged() {
                    let delta = drag_response.drag_delta();
                    self.offset.x -= delta.x / self.zoom;
                    self.offset.y -= delta.y / self.zoom;
                }

                if drag_response.hovered() {
                    ctx.set_cursor_icon(if drag_response.dragged() {
                        egui::CursorIcon::Grabbing
                    } else {
                        egui::CursorIcon::Grab
                    });
                }

                let max_offset_x = (img_w as f32 - panel.x / self.zoom).max(0.0);
                let max_offset_y = (img_h as f32 - panel.y / self.zoom).max(0.0);
                self.offset.x = self.offset.x.clamp(0.0, max_offset_x);
                self.offset.y = self.offset.y.clamp(0.0, max_offset_y);

                let uv_min = egui::pos2(
                    self.offset.x / img_w as f32,
                    self.offset.y / img_h as f32,
                );
                let uv_max = egui::pos2(
                    ((self.offset.x + panel.x / self.zoom) / img_w as f32).min(1.0),
                    ((self.offset.y + panel.y / self.zoom) / img_h as f32).min(1.0),
                );
                let uv = egui::Rect::from_min_max(uv_min, uv_max);


                let mut mesh = egui::Mesh::with_texture(tex.id());
                mesh.add_rect_with_uv(drag_rect, uv, egui::Color32::WHITE);
                ui.painter().add(egui::Shape::mesh(mesh));

                if let Some(&(cx, cy)) = self.centers.get(self.current_index) {
                    let screen_x = drag_rect.min.x + (cx as f32 - self.offset.x) * self.zoom;
                    let screen_y = drag_rect.min.y + (cy as f32 - self.offset.y) * self.zoom;
                    let painter = ui.painter();
                    let center = egui::pos2(screen_x, screen_y);
                    let arm = 10.0;
                    let stroke = egui::Stroke::new(2.0, egui::Color32::YELLOW);
                    painter.line_segment(
                        [center - egui::vec2(arm, 0.0), center + egui::vec2(arm, 0.0)],
                        stroke,
                    );
                    painter.line_segment(
                        [center - egui::vec2(0.0, arm), center + egui::vec2(0.0, arm)],
                        stroke,
                    );
                    painter.circle_stroke(center, arm * 1.4, stroke);
                }
            }
        });
    }
}

pub fn launch(image: DynamicImage, centers: Vec<(u32, u32)>) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Image Diff Viewer"),
        ..Default::default()
    };

    eframe::run_native(
        "Image Diff Viewer",
        options,
        Box::new(move |cc| Ok(Box::new(DiffApp::new(cc, image, centers)))),
    )
}