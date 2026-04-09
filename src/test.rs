use eframe::egui;
fn test(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Name");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Col 3");
            });
            ui.allocate_ui_with_layout(egui::vec2(80.0, 20.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Col 2");
            });
        });
    });
}
