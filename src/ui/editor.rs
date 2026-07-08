use crate::app::FrameStats;
use crate::renderer::RenderSettings;
use egui::load::SizedTexture;
use egui::{Align, CentralPanel, CollapsingHeader, Frame, Layout, Panel, Slider, Widget};

pub(crate) fn build(ui: &mut egui::Ui, texture_id: egui::TextureId, settings: &mut RenderSettings, stats: &FrameStats) -> egui::Rect {
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
            });

            ui.add_space(12.0);

            CollapsingHeader::new("Scene").show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 4.0;

                ui.label("Background Color");
                ui.color_edit_button_rgb(&mut settings.background);

                ui.add_space(12.0);

                ui.label("Model Color");
                ui.color_edit_button_rgb(&mut settings.color);
            });
        });

    CentralPanel::default()
        .frame(Frame::NONE)
        .show(ui, |ui| ui.image(SizedTexture::new(texture_id, ui.available_size())).rect)
        .inner
}
