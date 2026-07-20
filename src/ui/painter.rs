use egui::{Color32, Direction, Mesh, Rect, TextureId, epaint::Vertex};
use egui_material_icons::MaterialIcon;

pub trait PainterExt {
    fn shadow_color(&self) -> Color32 {
        Color32::from_black_alpha(127)
    }

    fn gradient(
        &self,
        direction: Direction,
        rect: Rect,
        start: Color32,
        stops: &[(f32, Color32)],
        end: Color32,
    );

    fn shadow_text(
        &self,
        pos: egui::Pos2,
        align: egui::Align2,
        text: impl ToString,
        font_id: egui::FontId,
        color: Color32,
    );

    fn shadow_icon(&self, pos: egui::Pos2, icon: MaterialIcon, font_size: f32, color: Color32) {
        self.shadow_text(
            pos,
            egui::Align2::CENTER_CENTER,
            <&str>::from(icon),
            egui::FontId::new(font_size, icon.font_family()),
            color,
        );
    }
}

impl PainterExt for egui::Painter {
    fn gradient(
        &self,
        direction: Direction,
        rect: Rect,
        start: Color32,
        stops: &[(f32, Color32)],
        end: Color32,
    ) {
        // normalize to pretending it's a left-to-right gradient
        let (lt, lb, rt, rb) = match direction {
            Direction::LeftToRight => {
                (rect.left_top(), rect.left_bottom(), rect.right_top(), rect.right_bottom())
            }
            Direction::RightToLeft => {
                (rect.right_bottom(), rect.right_top(), rect.left_bottom(), rect.left_top())
            }
            Direction::TopDown => {
                (rect.right_top(), rect.left_top(), rect.right_bottom(), rect.left_bottom())
            }
            Direction::BottomUp => {
                (rect.left_bottom(), rect.right_bottom(), rect.left_top(), rect.right_top())
            }
        };

        let mut mesh = Mesh {
            indices: vec![],
            vertices: vec![Vertex::untextured(lt, start), Vertex::untextured(lb, start)],
            texture_id: TextureId::default(),
        };

        for (frac, color) in stops.iter().copied().chain([(1.0, end)]) {
            mesh.vertices.extend_from_slice(&[
                Vertex::untextured(lt.lerp(rt, frac), color),
                Vertex::untextured(lb.lerp(rb, frac), color),
            ]);

            mesh.indices
                .extend_from_slice(&[0, 1, 2, 1, 2, 3].map(|n| n + mesh.vertices.len() as u32 - 4));
        }

        self.add(mesh);
    }

    fn shadow_text(
        &self,
        pos: egui::Pos2,
        align: egui::Align2,
        text: impl ToString,
        font_id: egui::FontId,
        color: Color32,
    ) {
        let text = text.to_string();
        let offset = egui::vec2(2., 2.);

        let shadow_color = self.shadow_color().gamma_multiply_u8(color.a());

        self.text(pos + offset, align, &text, font_id.clone(), shadow_color);
        self.text(pos, align, text, font_id, color);
    }
}
