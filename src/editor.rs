use egui::Frame;
use egui::load::SizedTexture;

pub(crate) fn build(ui: &mut egui::Ui, texture_id: egui::TextureId) -> egui::Rect {
    egui::CentralPanel::default()
        .frame(Frame::NONE)
        .show(ui, |ui| ui.image(SizedTexture::new(texture_id, ui.available_size())).rect)
        .inner
}
