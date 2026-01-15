// BigC UI Elements & Widgets
// [BigC Universal Public Mandate License (BUPML): Open for Contribution]

use crate::guy_engine::window::{css_color, BigGuyApp};
use crate::tokens::{Token, TokenType};
use eframe::egui;
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag};
use std::collections::HashMap;

pub fn apply_style(app: &BigGuyApp, style_name: &str, properties: &mut HashMap<String, String>) {
    if let Ok(styles) = app.interpreter.styles.read() {
        let lookup = style_name.to_lowercase();
        if let Some(style_props) = styles.get(&lookup) {
            for (k, v) in style_props {
                properties.insert(k.clone(), v.clone());
            }
        }
    }
}

pub fn draw_text(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let text_raw = app.interpreter.get_complex_value(i, tokens);
    let text = app.interpreter.interpolate_string(&text_raw);

    let mut props = HashMap::new();
    let mut pos: Option<egui::Pos2> = None;

    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                let style_name = app.interpreter.get_complex_value(&mut j, tokens);
                apply_style(app, &style_name, &mut props);
            }
            TokenType::Identifier(s) if s == "size" => {
                j += 1;
                props.insert(
                    "size".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Font => {
                j += 1;
                props.insert(
                    "font".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Fill => {
                j += 1;
                props.insert(
                    "fill".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Layer => {
                j += 1;
                props.insert(
                    "layer".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Alpha => {
                j += 1;
                props.insert(
                    "alpha".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Spacing => {
                j += 1;
                props.insert(
                    "spacing".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Padding => {
                j += 1;
                props.insert(
                    "padding".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let raw_x = app.interpreter.consume_math(&mut j, tokens);
                let raw_y = app.interpreter.consume_math(&mut j, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                pos = Some(egui::pos2(raw_x * s + ox, raw_y * s + oy));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;

    let s = app.interpreter.global_scale;
    let size = props
        .get("size")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(16.0)
        * s;
    let color = props
        .get("fill")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::WHITE);
    let font_name = props.get("font").cloned().unwrap_or_default();
    let line_height = props
        .get("spacing")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.5);
    let padding = props
        .get("padding")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        * s;
    let alpha = props
        .get("alpha")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);

    let final_color = if alpha != 1.0 {
        egui::Color32::from_rgba_premultiplied(
            (color.r() as f32 * alpha) as u8,
            (color.g() as f32 * alpha) as u8,
            (color.b() as f32 * alpha) as u8,
            (255.0 * alpha) as u8,
        )
    } else {
        color
    };

    let painter = if app.interpreter.autolayering_enabled {
        if let Some(layer_name) = props.get("layer") {
            match layer_name.as_str() {
                "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Background,
                    egui::Id::new("bg"),
                )),
                "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("ui"),
                )),
                "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::new("master"),
                )),
                _ => ui.painter().clone(),
            }
        } else {
            ui.painter().clone()
        }
    } else {
        ui.painter().clone()
    };

    let key = format!(
        "Txt:{}:{}:{}:{:?}:{}:{}:{}",
        text,
        size,
        font_name,
        color,
        line_height,
        alpha,
        ui.id().value()
    );
    if let Some(texture) = app.get_cached_texture(
        ui.ctx(),
        &key,
        &text,
        size,
        final_color,
        &font_name,
        line_height,
    ) {
        let size_vec = texture.size_vec2();
        let rect = if let Some(p) = pos {
            egui::Rect::from_min_size(p, size_vec)
        } else {
            let (r, _) = ui.allocate_exact_size(size_vec, egui::Sense::hover());
            r
        };
        painter.image(
            texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
        app.interpreter.last_condition_met = ui.rect_contains_pointer(rect);
    } else {
        let label = egui::RichText::new(text).size(size).color(final_color);
        let resp = if let Some(p) = pos {
            ui.put(
                egui::Rect::from_min_size(p, egui::vec2(100.0, size)),
                egui::Label::new(label),
            )
        } else {
            ui.add(egui::Label::new(label))
        };
        app.interpreter.last_condition_met = resp.hovered();
    }
    if padding > 0.0 {
        ui.add_space(padding);
    }
}

pub fn draw_button(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let text_raw = app.interpreter.get_complex_value(i, tokens);
    let text = app.interpreter.interpolate_string(&text_raw);

    let mut props = HashMap::new();
    let mut pos: Option<egui::Pos2> = None;

    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                let style_name = app.interpreter.get_complex_value(&mut j, tokens);
                apply_style(app, &style_name, &mut props);
            }
            TokenType::Identifier(s) if s == "size" => {
                j += 1;
                props.insert(
                    "width".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
                props.insert(
                    "height".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Font => {
                j += 1;
                props.insert(
                    "font".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Identifier(s) if s == "fontsize" => {
                j += 1;
                props.insert(
                    "fontsize".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Spacing => {
                j += 1;
                props.insert(
                    "spacing".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Padding => {
                j += 1;
                props.insert(
                    "padding".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Fill => {
                j += 1;
                props.insert(
                    "fill".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Identifier(s) if s == "color" => {
                j += 1;
                props.insert(
                    "color".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Alpha => {
                j += 1;
                props.insert(
                    "alpha".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Layer => {
                j += 1;
                props.insert(
                    "layer".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let raw_x = app.interpreter.consume_math(&mut j, tokens);
                let raw_y = app.interpreter.consume_math(&mut j, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                pos = Some(egui::pos2(raw_x * s + ox, raw_y * s + oy));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;

    let s = app.interpreter.global_scale;
    let w = props
        .get("width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(120.0)
        * s;
    let h = props
        .get("height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(40.0)
        * s;
    let size = props
        .get("fontsize")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(16.0)
        * s;
    let bg_color = props
        .get("fill")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::from_rgb(50, 50, 50));
    let text_color = props
        .get("color")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::WHITE);
    let alpha = props
        .get("alpha")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);

    let (final_bg, final_text) = if alpha != 1.0 {
        let a = (255.0 * alpha) as u8;
        (
            egui::Color32::from_rgba_premultiplied(
                (bg_color.r() as f32 * alpha) as u8,
                (bg_color.g() as f32 * alpha) as u8,
                (bg_color.b() as f32 * alpha) as u8,
                a,
            ),
            egui::Color32::from_rgba_premultiplied(
                (text_color.r() as f32 * alpha) as u8,
                (text_color.g() as f32 * alpha) as u8,
                (text_color.b() as f32 * alpha) as u8,
                a,
            ),
        )
    } else {
        (bg_color, text_color)
    };

    let font_name = props.get("font").cloned().unwrap_or_default();
    let line_height = props
        .get("spacing")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.5);
    let padding = props
        .get("padding")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);

    let painter = if app.interpreter.autolayering_enabled {
        if let Some(layer_name) = props.get("layer") {
            match layer_name.as_str() {
                "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Background,
                    egui::Id::new("bg"),
                )),
                "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("ui"),
                )),
                "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::new("master"),
                )),
                _ => ui.painter().clone(),
            }
        } else {
            ui.painter().clone()
        }
    } else {
        ui.painter().clone()
    };

    let (rect, response) = if let Some(p) = pos {
        let r = egui::Rect::from_min_size(p, egui::vec2(w, h));
        let resp = ui.allocate_rect(r, egui::Sense::click());
        (r, resp)
    } else {
        ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::click())
    };
    if response.hovered() {
        painter.rect_filled(rect, 5.0, final_bg.linear_multiply(1.2));
    } else {
        painter.rect_filled(rect, 5.0, final_bg);
    }
    let key = format!(
        "Btn:{}:{}:{}:{:?}:{}:{}:{}",
        text,
        size,
        font_name,
        text_color,
        line_height,
        alpha,
        ui.id().value()
    );
    if let Some(texture) = app.get_cached_texture(
        ui.ctx(),
        &key,
        &text,
        size,
        final_text,
        &font_name,
        line_height,
    ) {
        let tex_size = texture.size_vec2();
        let text_pos = rect.center() - tex_size / 2.0;
        painter.image(
            texture.id(),
            egui::Rect::from_min_size(text_pos, tex_size),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            egui::FontId::proportional(size),
            final_text,
        );
    }
    if padding > 0.0 {
        ui.add_space(padding);
    }
    if let Ok(mut clicked) = app.interpreter.last_widget_clicked.write() {
        *clicked = response.clicked();
    }
    app.interpreter.last_condition_met = response.hovered();
}

pub fn draw_rect(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    let is_rounded = tokens[*i].token_type == TokenType::Rounded;
    *i += 1;
    let mut props = HashMap::new();
    let mut pos: Option<egui::Pos2> = None;
    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                let style_name = app.interpreter.get_complex_value(&mut j, tokens);
                apply_style(app, &style_name, &mut props);
            }
            TokenType::Identifier(s) if s == "size" => {
                j += 1;
                props.insert(
                    "width".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
                props.insert(
                    "height".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Identifier(s) if s == "radius" => {
                j += 1;
                props.insert(
                    "radius".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Fill => {
                j += 1;
                props.insert(
                    "fill".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Stroke => {
                j += 1;
                props.insert(
                    "stroke".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Identifier(s) if s == "border" => {
                j += 1;
                props.insert(
                    "stroke_width".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Alpha => {
                j += 1;
                props.insert(
                    "alpha".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Layer => {
                j += 1;
                props.insert(
                    "layer".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Tag => {
                j += 1;
                props.insert(
                    "tag".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Scale => {
                j += 1;
                props.insert(
                    "scale".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let raw_x = app.interpreter.consume_math(&mut j, tokens);
                let raw_y = app.interpreter.consume_math(&mut j, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                pos = Some(egui::pos2(raw_x * s + ox, raw_y * s + oy));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;
    let s = app.interpreter.global_scale;
    let scale = props
        .get("scale")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);

    let w = props
        .get("width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(100.0)
        * s
        * scale;
    let h = props
        .get("height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(100.0)
        * s
        * scale;
    let color = props
        .get("fill")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::GRAY);
    let radius = props
        .get("radius")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(if is_rounded { 10.0 } else { 0.0 })
        * scale;

    let stroke_color = props
        .get("stroke")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_width = props
        .get("stroke_width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        * s
        * scale;
    let stroke = egui::Stroke::new(stroke_width, stroke_color);

    let painter = if app.interpreter.autolayering_enabled {
        if let Some(layer_name) = props.get("layer") {
            match layer_name.as_str() {
                "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Background,
                    egui::Id::new("bg"),
                )),
                "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("ui"),
                )),
                "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::new("master"),
                )),
                _ => ui.painter().clone(),
            }
        } else {
            ui.painter().clone()
        }
    } else {
        ui.painter().clone()
    };

    let (rect, response) = if let Some(p) = pos {
        let r = egui::Rect::from_min_size(p, egui::vec2(w, h));
        let sense = if props.contains_key("tag") {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        };
        let resp = ui.allocate_rect(r, sense);
        (r, resp)
    } else {
        let sense = if props.contains_key("tag") {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::hover()
        };
        ui.allocate_exact_size(egui::vec2(w, h), sense)
    };

    if let Some(alpha_str) = props.get("alpha") {
        if let Ok(alpha) = alpha_str.parse::<f32>() {
            let a = (255.0 * alpha) as u8;
            let final_color = egui::Color32::from_rgba_premultiplied(
                (color.r() as f32 * alpha) as u8,
                (color.g() as f32 * alpha) as u8,
                (color.b() as f32 * alpha) as u8,
                a,
            );
            painter.rect(rect, radius, final_color, stroke);
        } else {
            painter.rect(rect, radius, color, stroke);
        }
    } else {
        painter.rect(rect, radius, color, stroke);
    }

    if let Some(tag) = props.get("tag") {
        if response.hovered() {
            if let Ok(mut hovered) = app.interpreter.hovered_tags.write() {
                hovered.insert(tag.clone(), true);
            }
        }
        if response.is_pointer_button_down_on() {
            if let Ok(mut pressed) = app.interpreter.pressed_tags.write() {
                pressed.insert(tag.clone(), true);
            }
        }
        if response.dragged() {
            let delta = response.drag_delta();
            if let Ok(mut dragged) = app.interpreter.dragged_tags.write() {
                dragged.insert(tag.clone(), (delta.x, delta.y));
            }
            if let Ok(mut global_delta) = app.interpreter.frame_drag_delta.write() {
                *global_delta = (delta.x, delta.y);
            }
        }
        if response.clicked() {
            if let Ok(mut clicked) = app.interpreter.clicked_tags.write() {
                clicked.insert(tag.clone(), true);
            }
            let func_data = if let Ok(funcs) = app.interpreter.functions.read() {
                funcs.get(tag).cloned()
            } else {
                None
            };
            if let Some((_, func_tokens)) = func_data {
                let mut li = 0;
                app.render_recursive_standalone(&func_tokens, &mut li);
            }
        }
    }

    app.interpreter.last_condition_met = response.hovered();
}

pub fn draw_circle(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let mut props = HashMap::new();
    let mut pos: Option<egui::Pos2> = None;
    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                let style_name = app.interpreter.get_complex_value(&mut j, tokens);
                apply_style(app, &style_name, &mut props);
            }
            TokenType::Identifier(s) if s == "radius" => {
                j += 1;
                props.insert(
                    "radius".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Fill => {
                j += 1;
                props.insert(
                    "fill".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Stroke => {
                j += 1;
                props.insert(
                    "stroke".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Identifier(s) if s == "border" => {
                j += 1;
                props.insert(
                    "stroke_width".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Alpha => {
                j += 1;
                props.insert(
                    "alpha".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Layer => {
                j += 1;
                props.insert(
                    "layer".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let x = app.interpreter.consume_math(&mut j, tokens);
                let y = app.interpreter.consume_math(&mut j, tokens);
                pos = Some(egui::pos2(x, y));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;
    let s = app.interpreter.global_scale;
    let r = props
        .get("radius")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(20.0)
        * s;
    let color = props
        .get("fill")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::RED);

    let stroke_color = props
        .get("stroke")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_width = props
        .get("stroke_width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        * s;
    let stroke = egui::Stroke::new(stroke_width, stroke_color);

    let final_color = if let Some(alpha_str) = props.get("alpha") {
        if let Ok(alpha) = alpha_str.parse::<f32>() {
            let a = (255.0 * alpha) as u8;
            egui::Color32::from_rgba_premultiplied(
                (color.r() as f32 * alpha) as u8,
                (color.g() as f32 * alpha) as u8,
                (color.b() as f32 * alpha) as u8,
                a,
            )
        } else {
            color
        }
    } else {
        color
    };

    let painter = if app.interpreter.autolayering_enabled {
        if let Some(layer_name) = props.get("layer") {
            match layer_name.as_str() {
                "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Background,
                    egui::Id::new("bg"),
                )),
                "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("ui"),
                )),
                "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::new("master"),
                )),
                _ => ui.painter().clone(),
            }
        } else {
            ui.painter().clone()
        }
    } else {
        ui.painter().clone()
    };

    let (rect, response) = if let Some(p) = pos {
        let rect = egui::Rect::from_center_size(p, egui::vec2(r * 2.0, r * 2.0));
        let resp = ui.allocate_rect(rect, egui::Sense::hover());
        (rect, resp)
    } else {
        ui.allocate_exact_size(egui::vec2(r * 2.0, r * 2.0), egui::Sense::hover())
    };
    painter.circle(rect.center(), r, final_color, stroke);
    app.interpreter.last_condition_met = response.hovered();
}

pub fn draw_input(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let var_name = if *i < tokens.len() && tokens[*i].token_type == TokenType::LBrace {
        *i += 1;
        let name = app.interpreter.get_token_raw_name(&tokens[*i]);
        *i += 2;
        name
    } else {
        String::new()
    };
    if var_name.is_empty() {
        return;
    }
    let mut buffer = app
        .interpreter
        .get_variable(&var_name)
        .unwrap_or_default();

    // NOTE: Input currently uses default layout flow. If we add 'at' later, we need scaling.
    // For now, it's just 'draw input {Var}'.

    let response = ui.text_edit_singleline(&mut buffer);
    if response.changed() {
        app.interpreter.set_variable(var_name, buffer);
    }
    app.interpreter.last_condition_met = response.hovered();
}

pub fn draw_triangle(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let mut props = HashMap::new();
    let mut pos: Option<egui::Pos2> = None;
    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                let style_name = app.interpreter.get_complex_value(&mut j, tokens);
                apply_style(app, &style_name, &mut props);
            }
            TokenType::Identifier(s) if s == "size" => {
                j += 1;
                props.insert(
                    "width".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
                props.insert(
                    "height".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Fill => {
                j += 1;
                props.insert(
                    "fill".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Stroke => {
                j += 1;
                props.insert(
                    "stroke".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Identifier(s) if s == "border" => {
                j += 1;
                props.insert(
                    "stroke_width".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Alpha => {
                j += 1;
                props.insert(
                    "alpha".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Layer => {
                j += 1;
                props.insert(
                    "layer".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let x = app.interpreter.consume_math(&mut j, tokens);
                let y = app.interpreter.consume_math(&mut j, tokens);
                pos = Some(egui::pos2(x, y));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;
    let s = app.interpreter.global_scale;
    let w = props
        .get("width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(100.0)
        * s;
    let h = props
        .get("height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(100.0)
        * s;
    let color = props
        .get("fill")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::YELLOW);

    let stroke_color = props
        .get("stroke")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::TRANSPARENT);
    let stroke_width = props
        .get("stroke_width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        * s;
    let stroke = egui::Stroke::new(stroke_width, stroke_color);

    let final_color = if let Some(alpha_str) = props.get("alpha") {
        if let Ok(alpha) = alpha_str.parse::<f32>() {
            let a = (255.0 * alpha) as u8;
            egui::Color32::from_rgba_premultiplied(
                (color.r() as f32 * alpha) as u8,
                (color.g() as f32 * alpha) as u8,
                (color.b() as f32 * alpha) as u8,
                a,
            )
        } else {
            color
        }
    } else {
        color
    };

    let painter = if app.interpreter.autolayering_enabled {
        if let Some(layer_name) = props.get("layer") {
            match layer_name.as_str() {
                "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Background,
                    egui::Id::new("bg"),
                )),
                "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("ui"),
                )),
                "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                    egui::Order::Middle,
                    egui::Id::new("master"),
                )),
                _ => ui.painter().clone(),
            }
        } else {
            ui.painter().clone()
        }
    } else {
        ui.painter().clone()
    };

    let (rect, response) = if let Some(p) = pos {
        let r = egui::Rect::from_min_size(p, egui::vec2(w, h));
        let resp = ui.allocate_rect(r, egui::Sense::hover());
        (r, resp)
    } else {
        ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover())
    };
    painter.add(egui::Shape::convex_polygon(
        vec![rect.center_top(), rect.right_bottom(), rect.left_bottom()],
        final_color,
        stroke,
    ));
    app.interpreter.last_condition_met = response.hovered();
}

pub fn draw_line(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let mut p1 = egui::Pos2::default();
    let mut p2 = egui::Pos2::default();
    let mut color = egui::Color32::WHITE;
    let mut width = 1.0;
    let mut layer_name = String::new();
    while *i < tokens.len() {
        match &tokens[*i].token_type {
            TokenType::From => {
                *i += 1;
                let raw_x = app.interpreter.consume_math(i, tokens);
                let raw_y = app.interpreter.consume_math(i, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                p1.x = raw_x * s + ox;
                p1.y = raw_y * s + oy;
            }
            TokenType::To => {
                *i += 1;
                let raw_x = app.interpreter.consume_math(i, tokens);
                let raw_y = app.interpreter.consume_math(i, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                p2.x = raw_x * s + ox;
                p2.y = raw_y * s + oy;
            }
            TokenType::Stroke => {
                *i += 1;
                if let Ok(c) = css_color(&app.interpreter.get_complex_value(i, tokens)) {
                    color = c;
                }
            }
            TokenType::Identifier(s) if s == "width" => {
                *i += 1;
                width = app.interpreter.consume_math(i, tokens) * app.interpreter.global_scale;
            }
            TokenType::Alpha => {
                *i += 1;
                let a = app.interpreter.consume_math(i, tokens);
                color = egui::Color32::from_rgba_premultiplied(
                    (color.r() as f32 * a) as u8,
                    (color.g() as f32 * a) as u8,
                    (color.b() as f32 * a) as u8,
                    (255.0 * a) as u8,
                );
            }
            TokenType::Layer => {
                *i += 1;
                layer_name = app.interpreter.get_complex_value(i, tokens);
            }
            _ => {
                break;
            }
        }
    }
    *i -= 1;

    let painter = if app.interpreter.autolayering_enabled {
        match layer_name.as_str() {
            "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                egui::Order::Background,
                egui::Id::new("bg"),
            )),
            "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                egui::Order::Foreground,
                egui::Id::new("ui"),
            )),
            "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                egui::Order::Middle,
                egui::Id::new("master"),
            )),
            _ => ui.painter().clone(),
        }
    } else {
        ui.painter().clone()
    };

    painter.line_segment([p1, p2], egui::Stroke::new(width, color));
}

pub fn draw_image(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let path = app.interpreter.get_complex_value(i, tokens);
    let mut props = HashMap::new();
    let mut pos: Option<egui::Pos2> = None;
    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                let style_name = app.interpreter.get_complex_value(&mut j, tokens);
                apply_style(app, &style_name, &mut props);
            }
            TokenType::Identifier(s) if s == "size" => {
                j += 1;
                props.insert(
                    "width".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
                props.insert(
                    "height".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Tag => {
                j += 1;
                props.insert(
                    "tag".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Rotate => {
                j += 1;
                props.insert(
                    "rotate".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Scale => {
                j += 1;
                props.insert(
                    "scale".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Alpha => {
                j += 1;
                props.insert(
                    "alpha".to_string(),
                    app.interpreter.consume_math(&mut j, tokens).to_string(),
                );
            }
            TokenType::Tint => {
                j += 1;
                props.insert(
                    "tint".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::Layer => {
                j += 1;
                props.insert(
                    "layer".to_string(),
                    app.interpreter.get_complex_value(&mut j, tokens),
                );
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let raw_x = app.interpreter.consume_math(&mut j, tokens);
                let raw_y = app.interpreter.consume_math(&mut j, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                pos = Some(egui::pos2(raw_x * s + ox, raw_y * s + oy));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;

    let s = app.interpreter.global_scale;
    let width = props
        .get("width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        * s;
    let height = props
        .get("height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        * s;
    let rotate = props
        .get("rotate")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0)
        .to_radians();
    let scale = props
        .get("scale")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);
    let alpha = props
        .get("alpha")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(1.0);
    let tint_color = props
        .get("tint")
        .and_then(|s| css_color(s).ok())
        .unwrap_or(egui::Color32::WHITE);

    let mut real_path = path.clone();
    if let Ok(assets) = app.interpreter.assets.read() {
        if let Some(p) = assets.get(&path) {
            real_path = p.clone();
        }
    }

    if let Some(texture) = app.get_cached_image_file(ui.ctx(), &real_path) {
        let mut base_size = if width > 0.0 && height > 0.0 {
            egui::vec2(width, height)
        } else {
            texture.size_vec2()
        };
        if scale != 1.0 {
            base_size *= scale;
        }

        let final_tint = if alpha != 1.0 {
            let a = (255.0 * alpha) as u8;
            egui::Color32::from_rgba_premultiplied(
                (tint_color.r() as f32 * alpha) as u8,
                (tint_color.g() as f32 * alpha) as u8,
                (tint_color.b() as f32 * alpha) as u8,
                a,
            )
        } else {
            tint_color
        };

        let painter = if app.interpreter.autolayering_enabled {
            if let Some(layer_name) = props.get("layer") {
                match layer_name.as_str() {
                    "bg" | "background" => ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Background,
                        egui::Id::new("bg"),
                    )),
                    "ui" | "overlay" => ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Foreground,
                        egui::Id::new("ui"),
                    )),
                    "master" | "default" => ui.ctx().layer_painter(egui::LayerId::new(
                        egui::Order::Middle,
                        egui::Id::new("master"),
                    )),
                    _ => ui.painter().clone(),
                }
            } else {
                ui.painter().clone()
            }
        } else {
            ui.painter().clone()
        };

        let rect = if let Some(p) = pos {
            egui::Rect::from_min_size(p, base_size)
        } else {
            let (r, _) = ui.allocate_exact_size(base_size, egui::Sense::hover());
            r
        };
        let response = if props.contains_key("tag") {
            ui.allocate_rect(rect, egui::Sense::click_and_drag())
        } else {
            ui.allocate_rect(rect, egui::Sense::hover())
        };

        if rotate != 0.0 {
            let mut mesh = egui::Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                final_tint,
            );
            mesh.rotate(egui::emath::Rot2::from_angle(rotate), rect.center());
            painter.add(egui::Shape::mesh(mesh));
        } else {
            painter.image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                final_tint,
            );
        }

        if let Some(tag) = props.get("tag") {
            if response.hovered() {
                if let Ok(mut hovered) = app.interpreter.hovered_tags.write() {
                    hovered.insert(tag.clone(), true);
                }
            }
            if response.is_pointer_button_down_on() {
                if let Ok(mut pressed) = app.interpreter.pressed_tags.write() {
                    pressed.insert(tag.clone(), true);
                }
            }
            if response.dragged() {
                let delta = response.drag_delta();
                if let Ok(mut dragged) = app.interpreter.dragged_tags.write() {
                    dragged.insert(tag.clone(), (delta.x, delta.y));
                }
                if let Ok(mut global_delta) = app.interpreter.frame_drag_delta.write() {
                    *global_delta = (delta.x, delta.y);
                }
            }
            if response.clicked() {
                if let Ok(mut clicked) = app.interpreter.clicked_tags.write() {
                    clicked.insert(tag.clone(), true);
                }
                let func_data = if let Ok(funcs) = app.interpreter.functions.read() {
                    funcs.get(tag).cloned()
                } else {
                    None
                };
                if let Some((_, func_tokens)) = func_data {
                    let mut li = 0;
                    app.render_recursive_standalone(&func_tokens, &mut li);
                }
            }
        }
        app.interpreter.last_condition_met = response.hovered();
    } else {
        ui.label(format!("[Image Failed: {}]", real_path));
    }
}

pub fn draw_scroll_area(
    app: &mut BigGuyApp,
    i: &mut usize,
    tokens: &Vec<Token>,
    ui: &mut egui::Ui,
) {
    *i += 1;
    let mut w = 0.0;
    let mut h = 0.0;
    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Identifier(s) if s == "size" => {
                j += 1;
                w = app.interpreter.consume_math(&mut j, tokens);
                h = app.interpreter.consume_math(&mut j, tokens);
            }
            TokenType::Identifier(s) if s == "width" => {
                j += 1;
                w = app.interpreter.consume_math(&mut j, tokens);
            }
            TokenType::Identifier(s) if s == "height" => {
                j += 1;
                h = app.interpreter.consume_math(&mut j, tokens);
            }
            TokenType::AtWord | TokenType::At => break,
            TokenType::Draw | TokenType::Print | TokenType::If | TokenType::End => break,
            _ => j += 1,
        }
    }
    *i = j;
    let start = *i;
    let mut next_i = *i;
    egui::ScrollArea::vertical()
        .max_width(if w > 0.0 { w } else { f32::INFINITY })
        .max_height(if h > 0.0 { h } else { f32::INFINITY })
        .show(ui, |ui| {
            if w > 0.0 || h > 0.0 {
                ui.set_min_size(egui::vec2(w, h));
            }
            let mut li = start;
            app.render_recursive(ui, tokens, &mut li);
            next_i = li;
        });
    *i = next_i;
}

pub fn draw_markdown(app: &mut BigGuyApp, i: &mut usize, tokens: &Vec<Token>, ui: &mut egui::Ui) {
    *i += 1;
    let md_text_raw = app.interpreter.get_complex_value(i, tokens);
    let md_text = app.interpreter.interpolate_string(&md_text_raw);

    let mut pos: Option<egui::Pos2> = None;
    let mut style_base = String::new();
    let mut width = 0.0;

    let mut j = *i;
    while j < tokens.len() {
        match &tokens[j].token_type {
            TokenType::Style => {
                j += 1;
                style_base = app.interpreter.get_complex_value(&mut j, tokens);
            }
            TokenType::Identifier(s) if s == "width" => {
                j += 1;
                width = app.interpreter.consume_math(&mut j, tokens);
            }
            TokenType::AtWord | TokenType::At => {
                j += 1;
                let raw_x = app.interpreter.consume_math(&mut j, tokens);
                let raw_y = app.interpreter.consume_math(&mut j, tokens);
                let (ox, oy) = app.interpreter.global_offset;
                let s = app.interpreter.global_scale;
                pos = Some(egui::pos2(raw_x * s + ox, raw_y * s + oy));
            }
            _ => {
                break;
            }
        }
    }
    *i = j - 1;

    let s = app.interpreter.global_scale;
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(&md_text, options);

    let mut blocks = Vec::new();
    let mut current_block = Vec::new();
    let mut quote_depth = 0;

    for event in parser {
        match &event {
            Event::Start(Tag::BlockQuote) => {
                if quote_depth == 0 && !current_block.is_empty() {
                    blocks.push(current_block);
                    current_block = Vec::new();
                }
                quote_depth += 1;
                current_block.push(event.clone());
            }
            Event::End(Tag::BlockQuote) => {
                quote_depth -= 1;
                current_block.push(event.clone());
                if quote_depth == 0 {
                    blocks.push(current_block);
                    current_block = Vec::new();
                }
            }
            Event::Start(
                Tag::Heading(_, _, _) | Tag::Paragraph | Tag::Item | Tag::CodeBlock(_),
            ) if quote_depth == 0 => {
                if !current_block.is_empty() {
                    blocks.push(current_block);
                }
                current_block = vec![event.clone()];
            }
            Event::End(Tag::Heading(_, _, _) | Tag::Paragraph | Tag::Item | Tag::CodeBlock(_))
                if quote_depth == 0 =>
            {
                current_block.push(event.clone());
                blocks.push(current_block);
                current_block = Vec::new();
            }
            _ => current_block.push(event.clone()),
        }
    }
    if !current_block.is_empty() {
        blocks.push(current_block);
    }

    let container_rect = if let Some(p) = pos {
        egui::Rect::from_min_size(
            p,
            egui::vec2(if width > 0.0 { width } else { 400.0 }, 10000.0),
        )
    } else {
        ui.available_rect_before_wrap()
    };

    ui.allocate_ui_at_rect(container_rect, |ui| {
        ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        ui.vertical(|ui| {
            let mut list_index = 0;
            for block in blocks {
                ui.horizontal_wrapped(|ui| {
                    let mut block_sub = "body".to_string();
                    let mut inline_tags = Vec::new();
                    let mut current_url = String::new();
                    let mut is_in_quote = false;

                    for event in block {
                        match event {
                            Event::Start(tag) => {
                                match tag {
                                    Tag::Heading(level, _, _) => {
                                        block_sub = match level {
                                            HeadingLevel::H1 => "h1",
                                            HeadingLevel::H2 => "h2",
                                            HeadingLevel::H3 => "h3",
                                            _ => "h4",
                                        }
                                        .to_string();
                                    }
                                    Tag::Paragraph | Tag::List(_) => {
                                        // Do not overwrite block_sub if we are already in something special
                                        if block_sub == "body" {
                                            block_sub = "body".to_string();
                                        }
                                    }
                                    Tag::BlockQuote => {
                                        is_in_quote = true;
                                        block_sub = "quote".to_string();
                                        ui.add_space(20.0 * s);

                                        let mut qprops = HashMap::new();
                                        if !style_base.is_empty() {
                                            apply_style(
                                                app,
                                                &format!("{}.quote", style_base),
                                                &mut qprops,
                                            );
                                        }
                                        let qcolor = qprops
                                            .get("fill")
                                            .and_then(|v| css_color(v).ok())
                                            .unwrap_or(egui::Color32::GRAY);
                                        ui.label(egui::RichText::new("| ").color(qcolor).strong());
                                    }
                                    Tag::Item => {
                                        block_sub = "body".to_string();
                                        list_index += 1;

                                        let mut props = HashMap::new();
                                        if !style_base.is_empty() {
                                            apply_style(
                                                app,
                                                &format!("{}.{}", style_base, block_sub),
                                                &mut props,
                                            );
                                        }
                                        let size = props
                                            .get("size")
                                            .and_then(|v| v.parse::<f32>().ok())
                                            .unwrap_or(16.0)
                                            * s;
                                        let color = props
                                            .get("fill")
                                            .and_then(|v| css_color(v).ok())
                                            .unwrap_or(egui::Color32::WHITE);
                                        ui.label(
                                            egui::RichText::new(format!(" {}. ", list_index))
                                                .size(size)
                                                .color(color),
                                        );
                                    }
                                    Tag::Link(_, url, _) => {
                                        current_url = url.to_string();
                                        inline_tags.push(Tag::Link(
                                            pulldown_cmark::LinkType::Inline,
                                            "".into(),
                                            "".into(),
                                        ));
                                    }
                                    Tag::Strong | Tag::Emphasis | Tag::CodeBlock(_) => {
                                        inline_tags.push(tag);
                                    }
                                    _ => {}
                                }
                            }
                            Event::Text(text) => {
                                let mut props = HashMap::new();
                                // 1. Base Block Style
                                if !style_base.is_empty() {
                                    apply_style(
                                        app,
                                        &format!("{}.{}", style_base, block_sub),
                                        &mut props,
                                    );
                                }

                                let mut is_bold = false;
                                let mut is_italic = false;
                                let mut is_link = false;
                                let mut is_code = block_sub == "code";

                                for itag in &inline_tags {
                                    match itag {
                                        Tag::Strong => {
                                            is_bold = true;
                                        }
                                        Tag::Emphasis => {
                                            is_italic = true;
                                        }
                                        Tag::Link(..) => {
                                            is_link = true;
                                        }
                                        Tag::CodeBlock(_) => {
                                            is_code = true;
                                        }
                                        _ => {}
                                    }
                                }

                                // Inline overrides
                                if is_bold && !style_base.is_empty() {
                                    apply_style(app, &format!("{}.bold", style_base), &mut props);
                                }
                                if is_italic && !style_base.is_empty() {
                                    apply_style(app, &format!("{}.italic", style_base), &mut props);
                                }
                                if is_link && !style_base.is_empty() {
                                    apply_style(app, &format!("{}.link", style_base), &mut props);
                                }

                                let def_size = if block_sub.starts_with('h') {
                                    24.0
                                } else {
                                    16.0
                                };
                                let font_name = props.get("font").cloned().unwrap_or_default();
                                let size = props
                                    .get("size")
                                    .and_then(|v| v.parse::<f32>().ok())
                                    .unwrap_or(def_size)
                                    * s;
                                let color = props
                                    .get("fill")
                                    .and_then(|v| css_color(v).ok())
                                    .unwrap_or(if is_link {
                                        egui::Color32::from_rgb(30, 144, 255)
                                    } else {
                                        egui::Color32::WHITE
                                    });

                                let mut rt =
                                    egui::RichText::new(text.as_ref()).size(size).color(color);

                                // Apply Family
                                if !font_name.is_empty() {
                                    let family = match font_name.to_lowercase().as_str() {
                                        "monospace" | "code" => egui::FontFamily::Monospace,
                                        _ => egui::FontFamily::Name(font_name.clone().into()),
                                    };
                                    rt = rt.family(family);
                                }

                                if is_bold {
                                    rt = rt.strong();
                                }
                                if is_italic {
                                    rt = rt.italics();
                                }
                                if is_code {
                                    rt = rt.code();
                                }

                                if is_link {
                                    if ui.link(rt.underline()).clicked() {
                                        ui.ctx().output_mut(|o| {
                                            o.open_url = Some(egui::output::OpenUrl::new_tab(
                                                current_url.clone(),
                                            ))
                                        });
                                    }
                                } else {
                                    ui.label(rt);
                                }
                            }
                            Event::End(tag) => {
                                if matches!(
                                    tag,
                                    Tag::Strong | Tag::Emphasis | Tag::Link(..) | Tag::CodeBlock(_)
                                ) {
                                    inline_tags.pop();
                                }
                                if matches!(tag, Tag::BlockQuote) {
                                    is_in_quote = false;
                                }
                            }
                            Event::SoftBreak => {
                                ui.label(" ");
                            }
                            _ => {}
                        }
                    }
                });
                ui.add_space(8.0 * s); // Paragraph spacing
            }
        });
    });
}
