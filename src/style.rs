use egui::epaint::Shadow;
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Margin, Stroke, TextStyle, Theme, vec2};
use std::sync::Arc;

const FAINT_BG_COLOR: Color32 = Color32::from_rgb(0x17, 0x17, 0x17);
const EXTREME_BG_COLOR: Color32 = Color32::from_rgb(0x00, 0x00, 0x00);
const PANEL_BG_COLOR: Color32 = Color32::from_rgb(0x0d, 0x0d, 0x0d);
const TEXT_EDIT_BG_COLOR: Color32 = Color32::from_rgb(0x21, 0x21, 0x21);
const FLOATING_COLOR: Color32 = Color32::from_rgb(0x21, 0x21, 0x21);
const WIDGET_INACTIVE_BG_FILL: Color32 = Color32::from_rgb(0x2c, 0x2b, 0x2b);
const WIDGET_HOVERED_COLOR: Color32 = Color32::from_rgb(0x2c, 0x2b, 0x2b);
const WIDGET_NONINTERACTIVE_BG_STROKE: Color32 = Color32::from_rgb(0x27, 0x26, 0x26);
const SELECTION_BG_FILL: Color32 = Color32::from_rgb(0x18, 0x6a, 0xdd);
const SELECTION_STROKE_COLOR: Color32 = Color32::from_rgb(0xf0, 0xf2, 0xff);
const TEXT_SUBDUED: Color32 = Color32::from_rgb(0x93, 0x90, 0x90);
const TEXT_DEFAULT: Color32 = Color32::from_rgb(0xcf, 0xcf, 0xcf);
const TEXT_STRONG: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
const ERROR_FG_COLOR: Color32 = Color32::from_rgb(0xab, 0x01, 0x16);
const WARN_FG_COLOR: Color32 = Color32::from_rgb(0xff, 0x7a, 0x0c);
const POPUP_SHADOW_COLOR: Color32 = Color32::from_black_alpha(0x80);

pub(crate) fn apply_style(ctx: &egui::Context) {
    ctx.set_theme(Theme::Dark);

    set_fonts(ctx);

    let mut style = (*ctx.style_of(Theme::Dark)).clone();
    set_text_styles(&mut style);
    set_spacing(&mut style);
    set_colors(&mut style);
    ctx.set_style_of(Theme::Dark, style);
}

fn set_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Inter-Medium".into(),
        Arc::new(FontData::from_static(include_bytes!("fonts/Inter-Medium.otf"))),
    );
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "Inter-Medium".into());
    ctx.set_fonts(fonts);
}

fn set_text_styles(style: &mut egui::Style) {
    for text_style in [TextStyle::Body, TextStyle::Monospace, TextStyle::Button] {
        style.text_styles.get_mut(&text_style).unwrap().size = 12.0;
    }
    style.text_styles.get_mut(&TextStyle::Heading).unwrap().size = 16.0;

    style.spacing.interact_size.y = 15.0;
}

fn set_spacing(style: &mut egui::Style) {
    style.visuals.button_frame = true;

    style.visuals.widgets.inactive.bg_stroke = Default::default();
    style.visuals.widgets.hovered.bg_stroke = Default::default();
    style.visuals.widgets.active.bg_stroke = Default::default();
    style.visuals.widgets.open.bg_stroke = Default::default();

    style.visuals.widgets.hovered.expansion = 2.0;
    style.visuals.widgets.active.expansion = 2.0;
    style.visuals.widgets.open.expansion = 2.0;

    style.visuals.window_corner_radius = CornerRadius::same(6);
    style.visuals.menu_corner_radius = CornerRadius::same(6);
    
    let small_corner_radius = CornerRadius::same(4);
    style.visuals.widgets.noninteractive.corner_radius = small_corner_radius;
    style.visuals.widgets.inactive.corner_radius = small_corner_radius;
    style.visuals.widgets.hovered.corner_radius = small_corner_radius;
    style.visuals.widgets.active.corner_radius = small_corner_radius;
    style.visuals.widgets.open.corner_radius = small_corner_radius;

    style.spacing.item_spacing = vec2(8.0, 8.0);
    style.spacing.menu_margin = Margin::same(12);
    style.spacing.menu_spacing = 1.0;

    style.visuals.clip_rect_margin = 0.0;

    style.visuals.striped = false;
    style.visuals.indent_has_left_vline = false;
    style.spacing.button_padding = vec2(1.0, 0.0);
    style.spacing.indent = 14.0;

    style.spacing.combo_width = 8.0;

    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.scroll.fade.strength = 0.60;
    style.spacing.scroll.fade.size = 15.0;

    style.spacing.tooltip_width = 600.0;
}

fn set_colors(style: &mut egui::Style) {
    style.visuals.faint_bg_color = FAINT_BG_COLOR;

    style.visuals.extreme_bg_color = EXTREME_BG_COLOR;

    style.visuals.widgets.noninteractive.weak_bg_fill = PANEL_BG_COLOR;
    style.visuals.widgets.noninteractive.bg_fill = PANEL_BG_COLOR;
    style.visuals.text_edit_bg_color = Some(TEXT_EDIT_BG_COLOR);

    style.visuals.widgets.inactive.weak_bg_fill = Default::default();

    style.visuals.widgets.inactive.bg_fill = WIDGET_INACTIVE_BG_FILL;

    style.visuals.widgets.hovered.weak_bg_fill = WIDGET_HOVERED_COLOR;
    style.visuals.widgets.hovered.bg_fill = WIDGET_HOVERED_COLOR;
    style.visuals.widgets.active.weak_bg_fill = WIDGET_HOVERED_COLOR;
    style.visuals.widgets.active.bg_fill = WIDGET_HOVERED_COLOR;
    style.visuals.widgets.open.weak_bg_fill = WIDGET_HOVERED_COLOR;
    style.visuals.widgets.open.bg_fill = WIDGET_HOVERED_COLOR;

    style.visuals.selection.bg_fill = SELECTION_BG_FILL;
    style.visuals.selection.stroke.color = SELECTION_STROKE_COLOR;

    style.visuals.widgets.noninteractive.bg_stroke.color = WIDGET_NONINTERACTIVE_BG_STROKE;

    style.visuals.widgets.noninteractive.fg_stroke.color = TEXT_SUBDUED;
    style.visuals.widgets.inactive.fg_stroke.color = TEXT_DEFAULT;
    style.visuals.widgets.active.fg_stroke.color = TEXT_STRONG;

    let wide_stroke_width = 2.0;
    style.visuals.widgets.active.fg_stroke.width = wide_stroke_width;
    style.visuals.selection.stroke.width = wide_stroke_width;

    let shadow = Shadow {
        offset: [0, 15],
        blur: 50,
        spread: 0,
        color: POPUP_SHADOW_COLOR,
    };
    style.visuals.popup_shadow = shadow;
    style.visuals.window_shadow = shadow;

    style.visuals.window_fill = FLOATING_COLOR;
    style.visuals.window_stroke = Stroke::NONE;
    style.visuals.panel_fill = PANEL_BG_COLOR;

    style.visuals.hyperlink_color = TEXT_DEFAULT;

    style.visuals.error_fg_color = ERROR_FG_COLOR;
    style.visuals.warn_fg_color = WARN_FG_COLOR;
}
