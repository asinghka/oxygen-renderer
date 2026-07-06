use egui::Frame;
use egui::load::SizedTexture;

pub(crate) fn build(ui: &mut egui::Ui, texture_id: egui::TextureId) -> egui::Vec2 {
    egui::CentralPanel::default()
        .frame(Frame::NONE)
        .show(ui, |ui| {
            let size = ui.available_size();

            ui.image(SizedTexture::new(texture_id, size));

            size
        })
        .inner
}
