use crate::app::FrameStats;
use crate::mesh::{Scene, SceneNode};
use crate::renderer::RenderSettings;
use egui::load::SizedTexture;
use egui::{Align, Button, CentralPanel, CollapsingHeader, Frame, Layout, Margin, MenuBar, Panel, ScrollArea, Slider, Widget};
use re_ui::UiExt;

pub(crate) fn build(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    scene: &mut Scene,
    settings: &mut RenderSettings,
    stats: &FrameStats,
) -> egui::Rect {
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

    Panel::left("left-panel")
        .frame(Frame::NONE.fill(ui.tokens().panel_bg_color))
        .show(ui, |ui| {
            Panel::top("left-panel-header")
                .exact_size(ui.tokens().title_bar_height())
                .frame(Frame::default().inner_margin(Margin::symmetric(ui.tokens().view_padding(), 0)))
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.strong("Scene Tree");

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let at_least_one_visible = scene.at_least_one_visible();
                            let icon = if at_least_one_visible {
                                &re_ui::icons::VISIBLE
                            } else {
                                &re_ui::icons::INVISIBLE
                            };

                            let image = icon.as_image().fit_to_exact_size(ui.tokens().small_icon_size);
                            let response = Button::image(image).image_tint_follows_text_color(true).small().ui(ui);

                            if response.clicked() {
                                scene.set_all_visible(!at_least_one_visible)
                            }
                        })
                    });
                });

            ScrollArea::vertical().show(ui, |ui| {
                ui.take_available_space();
                Frame::default().inner_margin(12).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    for &root in &scene.root_indices {
                        node_tree(ui, &mut scene.scene_nodes, root);
                    }
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

fn node_tree(ui: &mut egui::Ui, nodes: &mut [SceneNode], index: usize) {
    let node = &mut nodes[index];
    let children = node.children.clone();
    let name = node.name.as_deref().unwrap_or("<unnamed>");

    if node.children.is_empty() {
        ui.horizontal(|ui| {
            ui.visuals_mut().widgets.hovered.expansion = 0.0;
            ui.visuals_mut().widgets.active.expansion = 0.0;

            ui.label(name);

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let icon = if node.visible {
                    &re_ui::icons::VISIBLE
                } else {
                    &re_ui::icons::INVISIBLE
                };

                let image = icon.as_image().fit_to_exact_size(ui.tokens().small_icon_size);
                let response = Button::image(image).image_tint_follows_text_color(true).small().ui(ui);

                if response.clicked() {
                    node.visible = !node.visible;
                }
            });
        });
    } else {
        CollapsingHeader::new(name).id_salt(index).default_open(false).show(ui, |ui| {
            for child in children {
                node_tree(ui, nodes, child);
            }
        });
    }
}
