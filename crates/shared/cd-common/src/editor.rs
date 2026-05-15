#![cfg(feature = "dev_editor")]

use bevy::app::App;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_inspector_egui::inspector_egui_impls::{InspectorEguiImpl, InspectorPrimitive};
use bevy_inspector_egui::reflect_inspector::InspectorUi;

use crate::Glyph;

/// Реализуем нативный трейт инспектора для нашего типа Glyph
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
            // Извлекаем RGB
            let mut color = [
                ((self.color() >> 16) & 0xFF) as u8,
                ((self.color() >> 8) & 0xFF) as u8,
                (self.color() & 0xFF) as u8,
            ];

            // 1. Кнопка выбора цвета
            if ui.color_edit_button_srgb(&mut color).changed() {
                let new_color =
                    ((color[0] as u32) << 16) | ((color[1] as u32) << 8) | (color[2] as u32);
                *self = Glyph::new(new_color, self.ch());
                changed = true;
            }

            // 2. Цветной символ
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

            // В Read-Only режиме просто делаем кнопку цвета неактивной
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

/// Регистрируем все кастомные UI-виджеты для типов из cd-common
pub fn register_common_editor_uis(app: &mut App) {
    app.register_type_data::<Glyph, InspectorEguiImpl>();
}
