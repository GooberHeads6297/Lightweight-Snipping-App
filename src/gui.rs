use eframe::egui;
use eframe::egui::ColorImage;
use image::{DynamicImage, GenericImageView};
use std::path::Path;

pub struct SnippingApp {
    pub image_path: Option<String>,
    pub texture: Option<egui::TextureHandle>,
    pub current_image: Option<DynamicImage>,
    pub drag_start: Option<egui::Pos2>,
    pub drag_end: Option<egui::Pos2>,
    pub target_width: u32,
    pub target_height: u32,
    pub keep_aspect: bool,
    pub save_as_path: String,
    pub zoom: f32,
    pub image_offset: egui::Vec2,
}

impl Default for SnippingApp {
    fn default() -> Self {
        Self {
            image_path: None,
            texture: None,
            current_image: None,
            drag_start: None,
            drag_end: None,
            target_width: 256,
            target_height: 256,
            keep_aspect: true,
            save_as_path: "output.png".to_string(),
            zoom: 1.0,
            image_offset: egui::vec2(0.0, 0.0),
        }
    }
}

impl SnippingApp {
    fn load_image(&mut self, ctx: &egui::Context, path: &str) {
        let img = image::open(path).expect("Failed to open image");
        self.set_texture_from_image(ctx, &img);
        self.target_width = img.width();
        self.target_height = img.height();
        self.current_image = Some(img);
        self.image_path = Some(path.to_string());
        self.image_offset = egui::vec2(0.0, 0.0);
    }

    fn load_image_dialog(&mut self, ctx: &egui::Context) {
        let default_dir = dirs::picture_dir().unwrap_or(dirs::home_dir().unwrap());

        if let Some(path) = rfd::FileDialog::new()
            .set_directory(default_dir)
            .add_filter("Image Files", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        {
            self.load_image(ctx, path.to_string_lossy().as_ref());
        }
    }

    fn reload_original(&mut self, ctx: &egui::Context) {
        if let Some(path) = &self.image_path {
            let path_clone = path.clone();
            if Path::new(&path_clone).exists() {
                self.load_image(ctx, &path_clone);
            }
        }
    }

    fn set_texture_from_image(&mut self, ctx: &egui::Context, img: &DynamicImage) {
        let size = [img.width() as usize, img.height() as usize];
        let rgba = img.to_rgba8();
        let pixels = rgba.into_vec();
        let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
        self.texture = Some(ctx.load_texture(
            "loaded_image",
            color_image,
            egui::TextureOptions::default(),
        ));
    }

    fn crop_current_image(&mut self, ctx: &egui::Context, rect: egui::Rect) {
        if let Some(img) = &self.current_image {
            let (img_w, img_h) = img.dimensions();
            let crop_x = rect.min.x.max(0.0) as u32;
            let crop_y = rect.min.y.max(0.0) as u32;
            let crop_w = (rect.width() as u32).min(img_w.saturating_sub(crop_x));
            let crop_h = (rect.height() as u32).min(img_h.saturating_sub(crop_y));

            if crop_w > 0 && crop_h > 0 {
                let cropped = img.crop_imm(crop_x, crop_y, crop_w, crop_h);
                self.set_texture_from_image(ctx, &cropped);
                self.current_image = Some(cropped);
                self.target_width = crop_w;
                self.target_height = crop_h;
                self.image_offset = egui::vec2(0.0, 0.0);
            }
        }
    }

    fn resize_current_image(&mut self, ctx: &egui::Context) {
        if let Some(img) = &self.current_image {
            let resized = if self.keep_aspect {
                img.resize(
                    self.target_width,
                    self.target_height,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                img.resize_exact(
                    self.target_width,
                    self.target_height,
                    image::imageops::FilterType::Lanczos3,
                )
            };
            self.set_texture_from_image(ctx, &resized);
            self.current_image = Some(resized);
            self.image_offset = egui::vec2(0.0, 0.0);
        }
    }

    fn save_as(&mut self) {
        if let Some(img) = &self.current_image {
            let default_dir = dirs::picture_dir().unwrap_or(dirs::home_dir().unwrap());

            if let Some(path) = rfd::FileDialog::new()
                .set_directory(default_dir)
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .add_filter("WebP Image", &["webp"])
                .set_file_name("output.png")
                .save_file()
            {
                let mut final_path = path.clone();

                if final_path.extension().is_none() {
                    let file_name = final_path.file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");

                    if file_name.contains("webp") {
                        final_path.set_extension("webp");
                    } else if file_name.contains("jpg") || file_name.contains("jpeg") {
                        final_path.set_extension("jpg");
                    } else {
                        final_path.set_extension("png");
                    }
                }

                img.save(&final_path).expect("Failed to save image");
                self.save_as_path = final_path.to_string_lossy().to_string();
            }
        }
    }

    fn save_overwrite(&mut self) {
        if let (Some(img), Some(path)) = (&self.current_image, &self.image_path) {
            img.save(path).expect("Failed to overwrite image");
        }
    }
}

impl eframe::App for SnippingApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("Lightweight Snipping Tool");
            if let Some(img) = &self.current_image {
                let (w, h) = img.dimensions();
                let file_name = self.image_path
                    .as_ref()
                    .map(|p| Path::new(p).file_name().unwrap().to_string_lossy().to_string())
                    .unwrap_or_else(|| "Unnamed".to_string());
                ui.label(format!("{} - {}x{}", file_name, w, h));
            } else {
                ui.label("No image loaded");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(texture) = self.texture.clone() {
                let tex_size = texture.size_vec2();
                let max_preview = egui::vec2(800.0, 800.0);
                let scale = (max_preview.x / tex_size.x)
                    .min(max_preview.y / tex_size.y)
                    .min(1.0);
                let preview_size = tex_size * scale * self.zoom;
            
                let available = ui.available_rect_before_wrap();
                let center = available.center();
                let rect = egui::Rect::from_center_size(center + self.image_offset, preview_size);
            
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());

                ui.put(rect, egui::Image::new((texture.id(), rect.size())));

            
                if response.dragged() {
                    self.image_offset += response.drag_delta();
                }
            
                if response.hovered() && !response.dragged() {
                    if ui.input(|i| i.pointer.primary_clicked()) {
                        self.drag_start = ui.input(|i| i.pointer.interact_pos());
                        self.drag_end = self.drag_start;
                    }
                    if ui.input(|i| i.pointer.primary_down()) {
                        self.drag_end = ui.input(|i| i.pointer.interact_pos());
                    }
                    if ui.input(|i| i.pointer.primary_released()) {
                        if let (Some(start), Some(end)) = (self.drag_start, self.drag_end) {
                            let rect = egui::Rect::from_two_pos(start, end);
                            self.crop_current_image(ctx, rect);
                        }
                        self.drag_start = None;
                        self.drag_end = None;
                    }
                }
            
                if let (Some(start), Some(end)) = (self.drag_start, self.drag_end) {
                    let rect = egui::Rect::from_two_pos(start, end);
                    ui.painter().rect_stroke(rect, 0.0, (2.0, egui::Color32::RED));
                }
            }
        });



        egui::TopBottomPanel::bottom("dock").show(ctx, |ui| {
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Load Image...").clicked() {
                    self.load_image_dialog(ctx);
                }
                if ui.button("Reload Original").clicked() {
                    self.reload_original(ctx);
                }
                if ui.button("Apply Resize").clicked() {
                    self.resize_current_image(ctx);
                }
                if ui.button("Save As...").clicked() {
                    self.save_as();
                }
                if ui.button("Overwrite Save").clicked() {
                    self.save_overwrite();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Width:");
                if ui.add(egui::DragValue::new(&mut self.target_width)).changed() {
                    if self.keep_aspect {
                        if let Some(img) = &self.current_image {
                            let aspect = img.height() as f32 / img.width() as f32;
                            self.target_height = (self.target_width as f32 * aspect) as u32;
                        }
                    }
                }

                ui.label("Height:");
                if ui.add(egui::DragValue::new(&mut self.target_height)).changed() {
                    if self.keep_aspect {
                        if let Some(img) = &self.current_image {
                            let aspect = img.width() as f32 / img.height() as f32;
                            self.target_width = (self.target_height as f32 * aspect) as u32;
                        }
                    }
                }

                ui.checkbox(&mut self.keep_aspect, "Lock aspect ratio");
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("Zoom In").clicked() {
                    self.zoom *= 1.1;
                }
                if ui.button("Zoom Out").clicked() {
                    self.zoom /= 1.1;
                }
                if ui.button("Reset Zoom").clicked() {
                    self.zoom = 1.0;
                    self.image_offset = egui::vec2(0.0, 0.0);
                }
                ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
            });
        });
    }
}
