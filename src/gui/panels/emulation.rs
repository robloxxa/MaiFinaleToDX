use eframe::egui;

pub struct EmulationState {
    pub mock_enabled: bool,
}

impl Default for EmulationState {
    fn default() -> Self {
        Self {
            mock_enabled: false,
        }
    }
}

pub fn show(ui: &mut egui::Ui, state: &mut EmulationState) {
    ui.heading("Hardware Emulation");
    ui.separator();

    ui.checkbox(&mut state.mock_enabled, "Enable Mock Ports");

    if !state.mock_enabled {
        ui.label("Enable mock ports to test protocols without physical hardware.");
        return;
    }

    ui.add_space(10.0);

    ui.collapsing("FiNALE Touch Emulation", |ui| {
        ui.label("Click zones to simulate FiNALE touch input:");
        ui.label("(Touch circle will be added in a follow-up)");
    });

    ui.add_space(5.0);

    ui.collapsing("DX Commands", |ui| {
        ui.horizontal(|ui| {
            if ui.button("RSET").clicked() {
                // TODO: push RSET command to mock port
            }
            if ui.button("HALT").clicked() {
                // TODO: push HALT command to mock port
            }
            if ui.button("STAT").clicked() {
                // TODO: push STAT command to mock port
            }
        });
    });

    ui.add_space(5.0);

    ui.collapsing("JVS Emulation", |ui| {
        ui.label("(JVS command buttons will be added in a follow-up)");
    });

    ui.add_space(5.0);

    ui.collapsing("Card Reader Emulation", |ui| {
        ui.label("(Card ID input will be added in a follow-up)");
    });
}
