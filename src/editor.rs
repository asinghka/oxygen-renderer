use egui::Frame;
use egui::load::SizedTexture;

pub(crate) fn build(ui: &mut egui::Ui, texture_id: egui::TextureId, size: egui::Vec2) {
    egui::Panel::bottom("debug-panel").show(ui, |ui| {
        ui.take_available_space();
    });
    egui::Panel::left("scene-tree").show(ui, |ui| {
        ui.take_available_space();
    });
    egui::Panel::right("inspector").show(ui, |ui| {
        ui.take_available_space();
    });
    egui::CentralPanel::default().frame(Frame::NONE).show(ui, |ui| {
        ui.image(SizedTexture::new(texture_id, size));
    });
}
