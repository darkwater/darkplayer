use egui::{
    Align, Color32, CornerRadius,
    FontFamily::{Monospace, Proportional},
    FontId, Margin, Shadow, Spacing, Stroke, TextStyle, Visuals,
    epaint::{FontColorTransferFunction, TextOptions},
    style::{
        HandleShape, ImeComposition, Interaction, NumberFormatter, NumericColorSpace,
        ScrollAnimation, ScrollStyle, Selection, WidgetVisuals, Widgets,
    },
    vec2,
};

pub fn style() -> egui::Style {
    {
        egui::Style {
            visuals: Visuals {
                dark_mode: true,
                text_options: TextOptions {
                    color_transfer_function: FontColorTransferFunction::DARK_MODE_DEFAULT,
                    max_texture_side: 2048, // Small but portable
                    font_hinting: true,
                    subpixel_binning: true,
                },
                override_text_color: None,
                weak_text_alpha: 0.6,
                weak_text_color: None,
                widgets: Widgets {
                    noninteractive: WidgetVisuals {
                        weak_bg_fill: Color32::from_black_alpha(27),
                        bg_fill: Color32::from_black_alpha(27),
                        bg_stroke: Stroke::new(2.0, Color32::from_black_alpha(60)), // separators, indentation lines
                        fg_stroke: Stroke::new(2.0, Color32::from_white_alpha(140)), // normal text color
                        corner_radius: CornerRadius::same(50),
                        expansion: 0.0,
                    },
                    inactive: WidgetVisuals {
                        weak_bg_fill: Color32::from_black_alpha(40), // button background
                        bg_fill: Color32::from_black_alpha(60),      // checkbox background
                        bg_stroke: Stroke::new(2.0, Color32::from_black_alpha(60)), // button border
                        fg_stroke: Stroke::new(2.0, Color32::WHITE), // button text
                        corner_radius: CornerRadius::same(50),
                        expansion: 0.0,
                    },
                    hovered: WidgetVisuals {
                        weak_bg_fill: Color32::from_black_alpha(70),
                        bg_fill: Color32::from_black_alpha(70),
                        bg_stroke: Stroke::new(2.0, Color32::from_black_alpha(150)), // e.g. hover over window edge or button
                        fg_stroke: Stroke::new(3.0, Color32::WHITE),
                        corner_radius: CornerRadius::same(50),
                        expansion: 0.0,
                    },
                    active: WidgetVisuals {
                        weak_bg_fill: Color32::from_black_alpha(55),
                        bg_fill: Color32::from_black_alpha(55),
                        bg_stroke: Stroke::new(2.0, Color32::WHITE),
                        fg_stroke: Stroke::new(4.0, Color32::WHITE),
                        corner_radius: CornerRadius::same(50),
                        expansion: 0.0,
                    },
                    open: WidgetVisuals {
                        weak_bg_fill: Color32::from_black_alpha(45),
                        bg_fill: Color32::from_black_alpha(27),
                        bg_stroke: Stroke::new(2.0, Color32::from_black_alpha(60)),
                        fg_stroke: Stroke::new(4.0, Color32::from_white_alpha(210)),
                        corner_radius: CornerRadius::same(50),
                        expansion: 0.0,
                    },
                },
                selection: Selection::default(),
                ime_composition: ImeComposition::default(),
                hyperlink_color: Color32::from_rgb(90, 170, 255),
                faint_bg_color: Color32::from_additive_luminance(5), // visible, but barely so
                extreme_bg_color: Color32::from_black_alpha(10),     // e.g. TextEdit background
                text_edit_bg_color: None, // use `extreme_bg_color` by default
                code_bg_color: Color32::from_black_alpha(64),
                warn_fg_color: Color32::from_rgb(255, 143, 0), // orange
                error_fg_color: Color32::from_rgb(255, 0, 0),  // red

                window_corner_radius: CornerRadius::ZERO,
                window_shadow: Shadow {
                    offset: [10, 20],
                    blur: 15,
                    spread: 0,
                    color: Color32::from_black_alpha(96),
                },
                window_fill: Color32::from_black_alpha(27),
                window_stroke: Stroke::new(1.0, Color32::from_black_alpha(60)),
                window_highlight_topmost: true,

                menu_corner_radius: CornerRadius::ZERO,

                panel_fill: Color32::from_black_alpha(164),

                popup_shadow: Shadow {
                    offset: [0, 0],
                    blur: 8,
                    spread: 0,
                    color: Color32::from_black_alpha(96),
                },

                resize_corner_size: 12.0,

                text_cursor: Default::default(),

                clip_rect_margin: 0.0,
                button_frame: true,
                collapsing_header_frame: false,
                indent_has_left_vline: true,

                striped: true,

                slider_trailing_fill: false,
                handle_shape: HandleShape::Rect { aspect_ratio: 0.75 },

                interact_cursor: None,

                image_loading_spinners: true,

                numeric_color_space: NumericColorSpace::GammaByte,
                disabled_alpha: 0.5,
            },
            override_font_id: None,
            override_text_style: None,
            override_text_valign: Some(Align::Center),
            text_styles: {
                [
                    (TextStyle::Small, FontId::new(18.0, Proportional)),
                    (TextStyle::Body, FontId::new(26.0, Proportional)),
                    (TextStyle::Button, FontId::new(26.0, Proportional)),
                    (TextStyle::Heading, FontId::new(36.0, Proportional)),
                    (TextStyle::Monospace, FontId::new(26.0, Monospace)),
                ]
                .into()
            },
            drag_value_text_style: TextStyle::Button,
            number_formatter: NumberFormatter::new(egui::emath::format_with_decimals_in_range),
            wrap_mode: None,
            spacing: Spacing {
                item_spacing: vec2(8.0, 3.0),
                window_margin: Margin::same(6),
                menu_margin: Margin::same(6),
                button_padding: vec2(6.0, 4.0),
                indent: 20.0, // match checkbox/radio-button with `button_padding.x + icon_width + icon_spacing`
                interact_size: vec2(40.0, 18.0),
                slider_width: 100.0,
                slider_rail_height: 8.0,
                combo_width: 100.0,
                text_edit_width: 280.0,
                icon_width: 14.0,
                icon_width_inner: 8.0,
                icon_spacing: 4.0,
                default_area_size: vec2(600.0, 400.0),
                tooltip_width: 500.0,
                menu_width: 400.0,
                menu_spacing: 2.0,
                combo_height: 200.0,
                scroll: ScrollStyle::floating(),
                indent_ends_with_horizontal_line: false,
            },
            interaction: Interaction {
                interact_radius: 5.0,
                resize_grab_radius_side: 3.0,
                resize_grab_radius_corner: 10.0,
                show_tooltips_only_when_still: true,
                tooltip_delay: 0.5,
                tooltip_grace_time: 0.2,
                selectable_labels: false,
                multi_widget_text_select: false,
            },
            animation_time: 0.2,
            #[cfg(debug_assertions)]
            debug: Default::default(),
            explanation_tooltips: false,
            url_in_tooltip: false,
            always_scroll_the_only_direction: false,
            scroll_animation: ScrollAnimation::default(),
            compact_menu_style: true,
        }
    }
}
