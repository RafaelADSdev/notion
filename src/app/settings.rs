use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingsTab {
    Appearance,
    Editor,
    Sidebar,
    Workspace,
}

impl SettingsTab {
    const ALL: [Self; 4] = [Self::Appearance, Self::Editor, Self::Sidebar, Self::Workspace];

    fn label(self) -> &'static str {
        match self {
            Self::Appearance => "Aparência",
            Self::Editor => "Editor",
            Self::Sidebar => "Barra lateral",
            Self::Workspace => "Espaço de trabalho",
        }
    }
}

impl Margem {
    pub(super) fn settings_window(&mut self, ctx: &Context, p: Palette) {
        let Some(mut tab) = self.settings_tab else { return };
        let screen = ctx.screen_rect();
        let width = (screen.width() - 48.0).min(730.0);
        let before = self.space.settings.clone();
        let mut close = false;
        let mut import = false;
        let modal = egui::Modal::new(egui::Id::new("settings-dialog"))
            .frame(Frame::popup(&ctx.style()).fill(p.white).corner_radius(15).inner_margin(0))
            .backdrop_color(egui::Color32::from_black_alpha(90))
            .show(ctx, |ui| {
                ui.set_width(width);
                Frame::new().fill(p.tint).inner_margin(egui::Margin::symmetric(22, 16))
                    .corner_radius(egui::CornerRadius { nw: 15, ne: 15, sw: 0, se: 0 })
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(p.heading("Configurações", 27.0));
                            right(ui, |ui| {
                                close = action(ui, p, "Fechar", false).clicked();
                            });
                        });
                        ui.add_space(8.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.visuals_mut().selection.bg_fill = p.white;
                            for choice in SettingsTab::ALL {
                                ui.selectable_value(&mut tab, choice, RichText::new(choice.label()).size(13.0));
                            }
                        });
                    });
                Frame::new().inner_margin(egui::Margin::symmetric(22, 16)).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ScrollArea::vertical().id_salt(("settings-content", tab.label()))
                        .max_height((screen.height() - 210.0).clamp(105.0, 280.0)).auto_shrink([false, false])
                        .show(ui, |ui| match tab {
                            SettingsTab::Appearance => {
                                let settings = &mut self.space.settings;
                                setting_row(ui, p, "Tema", "Escolha as cores do aplicativo.", |ui| {
                                    ComboBox::from_id_salt("theme").width(138.0)
                                        .selected_text(settings.theme.label()).show_ui(ui, |ui| {
                                            for theme in [Theme::Light, Theme::Dark, Theme::System] {
                                                ui.selectable_value(&mut settings.theme, theme, theme.label());
                                            }
                                        });
                                });
                                setting_row(ui, p, "Alto contraste", "Realça os contornos e o texto secundário.", |ui| {
                                    ui.checkbox(&mut settings.high_contrast, "Ativar alto contraste");
                                });
                                setting_row(ui, p, "Escala da interface", "Ajusta o tamanho do texto e dos controles.", |ui| {
                                    ui.add(egui::Slider::new(&mut settings.zoom, 0.85..=1.35).step_by(0.05)
                                        .custom_formatter(|zoom, _| format!("{:.0}%", zoom * 100.0))
                                        .custom_parser(|text| text.trim_end_matches('%').trim().parse::<f64>().ok().map(|value| value / 100.0)));
                                });
                            }
                            SettingsTab::Editor => {
                                let settings = &mut self.space.settings;
                                setting_row(ui, p, "Largura da página", "Aumenta a área de escrita dos documentos.", |ui| {
                                    ui.checkbox(&mut settings.wide_documents, "Usar largura ampla");
                                });
                                setting_row(ui, p, "Ao iniciar", "Volta ao documento que estava aberto ao sair.", |ui| {
                                    ui.checkbox(&mut settings.restore_last, "Reabrir a última página");
                                });
                            }
                            SettingsTab::Sidebar => {
                                let settings = &mut self.space.settings;
                                setting_row(ui, p, "Visibilidade", "Use Ctrl+E para alternar rapidamente.", |ui| {
                                    ui.checkbox(&mut settings.show_sidebar, "Exibir barra lateral");
                                });
                                setting_row(ui, p, "Favoritos", "Mostra suas páginas favoritas acima da lista.", |ui| {
                                    ui.checkbox(&mut settings.show_favorites, "Exibir favoritos");
                                });
                                setting_row(ui, p, "Espaçamento", "Reduz a distância entre os itens da interface.", |ui| {
                                    ui.checkbox(&mut settings.compact, "Usar espaçamento compacto");
                                });
                            }
                            SettingsTab::Workspace => {
                                let settings = &mut self.space.settings;
                                setting_row(ui, p, "Nome do espaço", "Aparece na página inicial.", |ui| {
                                    ui.add(TextEdit::singleline(&mut settings.name).desired_width(192.0).char_limit(44));
                                });
                                setting_row(ui, p, "Importação", "Abra um arquivo Markdown como uma página local.", |ui| {
                                    import = action(ui, p, "Importar Markdown", false).clicked();
                                });
                            }
                        });
                });
            });
        if before != self.space.settings {
            self.dirty(ctx);
        }
        self.settings_tab = if close || import || modal.should_close() { None } else { Some(tab) };
        if import {
            self.start_transfer(Transfer::Import);
        }
    }
}

fn setting_row(ui: &mut Ui, p: Palette, title: &str, description: &str, control: impl FnOnce(&mut Ui)) {
    ui.add_space(9.0);
    if ui.available_width() >= 450.0 {
        ui.horizontal_top(|ui| {
            let left_width = ui.available_width() * 0.55;
            ui.allocate_ui_with_layout(vec2(left_width, 0.0), Layout::top_down(Align::Min), |ui| {
                ui.label(p.heading(title, 15.0));
                ui.label(p.label(description));
            });
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 30.0),
                Layout::right_to_left(Align::Center),
                control,
            );
        });
    } else {
        ui.label(p.heading(title, 15.0));
        ui.label(p.label(description));
        control(ui);
    }
    ui.add_space(9.0);
    ui.separator();
}
