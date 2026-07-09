use crate::app::FrameStats;
use crate::renderer::RenderSettings;
use egui::load::SizedTexture;
use egui::{Align, CentralPanel, CollapsingHeader, Frame, Layout, MenuBar, Panel, Slider, Widget};
use re_ui::UiExt;

pub(crate) fn build(ui: &mut egui::Ui, texture_id: egui::TextureId, settings: &mut RenderSettings, stats: &FrameStats) -> egui::Rect {
    Panel::top("top-panel")
        .frame(ui.tokens().top_panel_frame(re_ui::WindowFrameConfig::Native))
        .show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.set_height(32.0);

                ui.menu_button("File", |ui| {
                    if ui.button("Load file...").clicked() {
                        if let Some(_) = rfd::FileDialog::new().add_filter("scene", &["glb"]).pick_file() {}
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {}
                });

                ui.menu_button("View", |ui| {
                    let _ = ui.button("Reset");
                });

                ui.menu_button("Settings", |ui| {
                    let _ = ui.button("Render settings");
                    let _ = ui.button("Camera");
                });
            });
        });

    Panel::bottom("bottom-panel").show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.add_space(12.0);
            ui.monospace(stats.model());
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(12.0);
                ui.monospace(stats.time());
            });
        });
    });

    Panel::right("right-panel")
        .frame(Frame::side_top_panel(ui.style()).inner_margin(12))
        .show(ui, |ui| {
            ui.take_available_space();

            CollapsingHeader::new("Rendering").show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                ui.label("Wireframe Mode");
                ui.checkbox(&mut settings.wireframe, "");

                ui.add_space(12.0);

                ui.label("Ambient Light Strength");
                Slider::new(&mut settings.ambient, 0.0..=1.0).ui(ui);

                ui.add_space(12.0);

                ui.label("Diffuse Lighting");
                ui.checkbox(&mut settings.diffuse, "");

                ui.add_space(12.0);

                ui.label("Specular Highlights");
                ui.checkbox(&mut settings.specular, "");

                ui.label("Specular Strength");
                Slider::new(&mut settings.specular_strength, 0.0..=1.0).ui(ui);

                ui.label("Shininess");
                Slider::new(&mut settings.shininess, 0.0..=1.0).ui(ui);
            });

            ui.add_space(12.0);

            CollapsingHeader::new("Scene").show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                ui.label("Background Color");
                ui.color_edit_button_rgb(&mut settings.background);
            });
        });

    CentralPanel::default()
        .frame(Frame::NONE)
        .show(ui, |ui| ui.image(SizedTexture::new(texture_id, ui.available_size())).rect)
        .inner
}
