use crate::app::FrameStats;
use crate::renderer::RenderSettings;
use crate::scene::{Model, ModelNode};
use egui::collapsing_header::CollapsingState;
use egui::load::SizedTexture;
use egui::{Align, Button, CentralPanel, CollapsingHeader, Frame, Layout, Margin, MenuBar, Panel, ScrollArea, Slider, Widget};
use re_ui::UiExt;
use std::collections::VecDeque;
use std::path::PathBuf;

pub(crate) enum EditorCommand {
    LoadFile(PathBuf),
    ResetCamera,
    Quit,
}

pub(crate) fn build(
    ui: &mut egui::Ui,
    texture_id: egui::TextureId,
    scene: &mut Model,
    settings: &mut RenderSettings,
    stats: &FrameStats,
    editor_commands: &mut VecDeque<EditorCommand>,
) -> egui::Rect {
    Panel::top("top-panel")
        .frame(ui.tokens().top_panel_frame(re_ui::WindowFrameConfig::Native))
        .show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.set_height(32.0);

                ui.menu_button("File", |ui| {
                    if ui.button("Load file...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().add_filter("scene", &["glb"]).pick_file() {
                            editor_commands.push_back(EditorCommand::LoadFile(path));
                        }
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        editor_commands.push_back(EditorCommand::Quit);
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Reset Camera").clicked() {
                        editor_commands.push_back(EditorCommand::ResetCamera);
                    }
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
        .default_size(260.0)
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
                Frame::default().inner_margin(ui.tokens().view_padding()).show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0;
                    for &root in &scene.root_indices {
                        node_tree(ui, &mut scene.scene_nodes, root);
                    }
                });
            });
        });

    Panel::right("right-panel")
        .frame(Frame::NONE.fill(ui.tokens().panel_bg_color))
        .default_size(220.0)
        .show(ui, |ui| {
            ui.take_available_space();

            Panel::top("right-panel-header")
                .exact_size(ui.tokens().title_bar_height())
                .frame(Frame::default().inner_margin(Margin::symmetric(ui.tokens().view_padding(), 0)))
                .show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.strong("Settings");
                    });
                });

            Frame::NONE
                .fill(ui.tokens().panel_bg_color)
                .inner_margin(ui.tokens().view_padding())
                .show(ui, |ui| {
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

                        ui.label("Show grid");
                        ui.checkbox(&mut settings.grid, "");

                        ui.add_space(12.0);

                        ui.label("Background Color");
                        ui.color_edit_button_rgb(&mut settings.background);

                        ui.add_space(12.0);

                        ui.label("Bump strength");
                        Slider::new(&mut settings.bump, 0.0..=5.0).ui(ui);
                    });
                });
        });

    CentralPanel::default()
        .frame(Frame::NONE)
        .show(ui, |ui| ui.image(SizedTexture::new(texture_id, ui.available_size())).rect)
        .inner
}

fn node_tree(ui: &mut egui::Ui, nodes: &mut [ModelNode], index: usize) {
    let children = nodes[index].children.clone();

    if children.is_empty() {
        ui.horizontal(|ui| {
            let indent = ui.spacing().indent;
            ui.add_space(indent);
            visibility_row(ui, &mut nodes[index]);
        });
    } else {
        let id = ui.make_persistent_id(index);
        let header = CollapsingState::load_with_default_open(ui.ctx(), id, false).show_header(ui, |ui| visibility_row(ui, &mut nodes[index]));

        header.body(|ui| {
            for child in children {
                node_tree(ui, nodes, child);
            }
        });
    }
}

fn visibility_row(ui: &mut egui::Ui, node: &mut ModelNode) {
    ui.visuals_mut().widgets.hovered.expansion = 0.0;
    ui.visuals_mut().widgets.active.expansion = 0.0;

    let name = node.name.as_deref().unwrap_or("<unnamed>");
    ui.label(name);

    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if let Some(visible) = node.visible {
            let icon = if visible { &re_ui::icons::VISIBLE } else { &re_ui::icons::INVISIBLE };

            let image = icon.as_image().fit_to_exact_size(ui.tokens().small_icon_size);
            if Button::image(image).image_tint_follows_text_color(true).small().ui(ui).clicked() {
                node.visible = Some(!visible);
            }
        };
    });
}
