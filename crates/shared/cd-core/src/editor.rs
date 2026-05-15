#![cfg(feature = "dev_editor")]

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
use bevy_inspector_egui::reflect_inspector::InspectorUi;

use crate::{ObjectGuid, WorldPos, glyph::Glyph};

// ============================================================================
// UI для GLYPH (Цветная буква + Палитра)
// ============================================================================
impl InspectorPrimitive for Glyph {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            let mut color = [
                ((self.color() >> 16) & 0xFF) as u8,
                ((self.color() >> 8) & 0xFF) as u8,
                (self.color() & 0xFF) as u8,
            ];

            if ui.color_edit_button_srgb(&mut color).changed() {
                let new_color =
                    ((color[0] as u32) << 16) | ((color[1] as u32) << 8) | (color[2] as u32);
                *self = Glyph::new(new_color, self.ch());
                changed = true;
            }

            let text = egui::RichText::new(self.to_char().to_string())
                .color(egui::Color32::from_rgb(color[0], color[1], color[2]))
                .strong()
                .size(18.0);

            ui.label(text);
        });

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        ui.horizontal(|ui| {
            let mut color = [
                ((self.color() >> 16) & 0xFF) as u8,
                ((self.color() >> 8) & 0xFF) as u8,
                (self.color() & 0xFF) as u8,
            ];

            ui.add_enabled_ui(false, |ui| {
                ui.color_edit_button_srgb(&mut color);
            });

            let text = egui::RichText::new(self.to_char().to_string())
                .color(egui::Color32::from_rgb(color[0], color[1], color[2]))
                .strong()
                .size(18.0);

            ui.label(text);
        });
    }
}

// ============================================================================
// UI для WORLD POS (X, Y, Z редактируемые ползунки)
// ============================================================================
impl InspectorPrimitive for WorldPos {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        let mut changed = false;
        let (mut x, mut y, mut z) = self.xyz();

        ui.horizontal(|ui| {
            ui.label("X:");
            changed |= ui.add(egui::DragValue::new(&mut x)).changed();
            ui.label("Y:");
            changed |= ui.add(egui::DragValue::new(&mut y)).changed();
            ui.label("Z:");
            changed |= ui.add(egui::DragValue::new(&mut z)).changed();
        });

        if changed {
            *self = WorldPos::new(x, y, z);
        }

        changed
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        let (x, y, z) = self.xyz();
        ui.label(format!("X: {} | Y: {} | Z: {}", x, y, z));
    }
}

// ============================================================================
// UI для OBJECT GUID (Только чтение, чтобы не сломать кэши сервера!)
// ============================================================================
impl InspectorPrimitive for ObjectGuid {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) -> bool {
        ui.vertical(|ui| {
            // Выводим сырое число (u64) жирным шрифтом
            ui.label(egui::RichText::new(self.to_string()).strong());
            // Выводим расшифровку битовой маски тусклым шрифтом
            ui.label(egui::RichText::new(format!("{:?}", self)).weak());
        });

        // Возвращаем false, так как мы не разрешаем редактировать GUID через UI
        false
    }

    fn ui_readonly(
        &self,
        ui: &mut egui::Ui,
        _: &dyn std::any::Any,
        _: egui::Id,
        _: InspectorUi<'_, '_>,
    ) {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(self.to_string()).strong());
            ui.label(egui::RichText::new(format!("{:?}", self)).weak());
        });
    }
}

/// Регистрация всех кастомных отображений ядра
pub fn register_core_editor_uis(app: &mut App) {
    app.register_type_data::<Glyph, InspectorEguiImpl>();
    app.register_type_data::<WorldPos, InspectorEguiImpl>();
    app.register_type_data::<ObjectGuid, InspectorEguiImpl>();
}
