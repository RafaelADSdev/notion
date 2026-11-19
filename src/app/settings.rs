use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SettingsTab {
    Appearance,
    Editor,
    Sidebar,
    Workspace,
}

impl SettingsTab {
    const ALL: [Self; 4] = [
        Self::Appearance,
        Self::Editor,
        Self::Sidebar,
        Self::Workspace,
    ];

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
        let Some(mut tab) = self.settings_tab else {
            return;
        };
        let screen = ctx.screen_rect();
        let width = (screen.width() - 40.0).min(752.0);
        let split = width >= 576.0;
        let before = self.space.settings.clone();
        let mut close = false;
        let mut import = false;
        let modal = egui::Modal::new(egui::Id::new("settings-dialog"))
            .frame(
                Frame::popup(&ctx.style())
                    .fill(p.white)
                    .stroke(egui::Stroke::new(1.0_f32, p.line))
                    .corner_radius(12)
                    .inner_margin(0),
            )
            .backdrop_color(egui::Color32::from_black_alpha(85))
            .show(ctx, |ui| {
                ui.set_width(width);
                ui.spacing_mut().item_spacing.y = 0.0;
                Frame::new()
                    .inner_margin(egui::Margin::symmetric(20, 14))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(p.heading("Configurações", 24.0));
                            right(ui, |ui| close = action(ui, p, "Fechar", false).clicked());
                        });
                    });
                ui.separator();

                let body_height =
                    (screen.height() - if split { 122.0 } else { 180.0 }).clamp(104.0, 308.0);
                let mut settings = |ui: &mut Ui, tab| {
                    import = self.settings_content(ui, p, tab, body_height);
                };
                if split {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.vertical(|ui| {
                            Frame::new()
                                .fill(p.side)
                                .inner_margin(egui::Margin::symmetric(12, 16))
                                .corner_radius(egui::CornerRadius {
                                    nw: 0,
                                    ne: 0,
                                    sw: 11,
                                    se: 0,
                                })
                                .show(ui, |ui| {
                                    ui.set_width(164.0);
                                    ui.set_min_height(body_height - 8.0);
                                    ui.spacing_mut().item_spacing.y = 4.0;
                                    for choice in SettingsTab::ALL {
                                        if design::nav(ui, p, choice.label(), tab == choice, 32.0)
                                            .clicked()
                                        {
                                            tab = choice;
                                        }
                                    }
                                });
                        });
                        ui.vertical(|ui| settings(ui, tab));
                    });
                } else {
                    Frame::new()
                        .fill(p.side)
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = vec2(4.0, 4.0);
                            ui.horizontal_wrapped(|ui| {
                                for choice in SettingsTab::ALL {
                                    ui.selectable_value(&mut tab, choice, choice.label());
                                }
                            });
                        });
                    settings(ui, tab);
                }
            });

        if before != self.space.settings {
            self.dirty(ctx);
        }
        self.settings_tab = if close || import || modal.should_close() {
            None
        } else {
            Some(tab)
        };
        if import {
            self.start_transfer(Transfer::Import);
        }
    }

    fn settings_content(&mut self, ui: &mut Ui, p: Palette, tab: SettingsTab, height: f32) -> bool {
        let settings = &mut self.space.settings;
        let mut import = false;
        Frame::new()
            .inner_margin(egui::Margin::symmetric(20, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing = vec2(8.0, 4.0);
                ScrollArea::vertical()
                    .id_salt(("settings-content", tab.label()))
                    .max_height(height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label(p.heading(tab.label(), 20.0));
                        ui.add_space(8.0);
                        match tab {
                            SettingsTab::Appearance => {
                                setting_row(
                                    ui,
                                    p,
                                    "Tema",
                                    "Escolha as cores do aplicativo.",
                                    |ui| {
                                        ComboBox::from_id_salt("theme")
                                            .width(128.0)
                                            .selected_text(settings.theme.label())
                                            .show_ui(ui, |ui| {
                                                for theme in
                                                    [Theme::Light, Theme::Dark, Theme::System]
                                                {
                                                    ui.selectable_value(
                                                        &mut settings.theme,
                                                        theme,
                                                        theme.label(),
                                                    );
                                                }
                                            });
                                    },
                                );
                                setting_row(
                                    ui,
                                    p,
                                    "Alto contraste",
                                    "Realça os contornos e o texto secundário.",
                                    |ui| {
                                        ui.checkbox(
                                            &mut settings.high_contrast,
                                            "Ativar alto contraste",
                                        );
                                    },
                                );
                                setting_row(
                                    ui,
                                    p,
                                    "Escala da interface",
                                    "Ajusta o tamanho do texto e dos controles.",
                                    |ui| {
                                        ui.add(
                                            egui::Slider::new(&mut settings.zoom, 0.85..=1.35)
                                                .step_by(0.05)
                                                .custom_formatter(|zoom, _| {
                                                    format!("{:.0}%", zoom * 100.0)
                                                })
                                                .custom_parser(|text| {
                                                    text.trim_end_matches('%')
                                                        .trim()
                                                        .parse::<f64>()
                                                        .ok()
                                                        .map(|value| value / 100.0)
                                                }),
                                        );
                                    },
                                );
                            }
                            SettingsTab::Editor => {
                                setting_row(
                                    ui,
                                    p,
                                    "Largura da página",
                                    "Aumenta a área de escrita dos documentos.",
                                    |ui| {
                                        ui.checkbox(
                                            &mut settings.wide_documents,
                                            "Usar largura ampla",
                                        );
                                    },
                                );
                                setting_row(
                                    ui,
                                    p,
                                    "Ao iniciar",
                                    "Volta ao documento que estava aberto ao sair.",
                                    |ui| {
                                        ui.checkbox(
                                            &mut settings.restore_last,
                                            "Reabrir a última página",
                                        );
                                    },
                                );
                            }
                            SettingsTab::Sidebar => {
                                setting_row(
                                    ui,
                                    p,
                                    "Visibilidade",
                                    "Use Ctrl+E para alternar rapidamente.",
                                    |ui| {
                                        ui.checkbox(
                                            &mut settings.show_sidebar,
                                            "Exibir barra lateral",
                                        );
                                    },
                                );
                                setting_row(
                                    ui,
                                    p,
                                    "Favoritos",
                                    "Mostra suas páginas favoritas acima da lista.",
                                    |ui| {
                                        ui.checkbox(
                                            &mut settings.show_favorites,
                                            "Exibir favoritos",
                                        );
                                    },
                                );
                                setting_row(
                                    ui,
                                    p,
                                    "Espaçamento",
                                    "Reduz a distância entre os itens da interface.",
                                    |ui| {
                                        ui.checkbox(
                                            &mut settings.compact,
                                            "Usar espaçamento compacto",
                                        );
                                    },
                                );
                            }
                            SettingsTab::Workspace => {
                                setting_row(
                                    ui,
                                    p,
                                    "Nome do espaço",
                                    "Aparece na página inicial.",
                                    |ui| {
                                        ui.add(
                                            TextEdit::singleline(&mut settings.name)
                                                .desired_width(176.0)
                                                .char_limit(44),
                                        );
                                    },
                                );
                                setting_row(
                                    ui,
                                    p,
                                    "Importação",
                                    "Abra um arquivo Markdown como uma página local.",
                                    |ui| {
                                        import =
                                            action(ui, p, "Importar Markdown", false).clicked();
                                    },
                                );
                            }
                        }
                    });
            });
        import
    }
}

fn setting_row(
    ui: &mut Ui,
    p: Palette,
    title: &str,
    description: &str,
    control: impl FnOnce(&mut Ui),
) {
    ui.separator();
    ui.add_space(8.0);
    let width = ui.available_width();
    if width >= 440.0 {
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(width - 214.0, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    ui.label(p.heading(title, 14.0));
                    ui.label(p.label(description));
                },
            );
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 38.0),
                Layout::right_to_left(Align::Center),
                control,
            );
        });
    } else {
        ui.label(p.heading(title, 14.0));
        ui.label(p.label(description));
        ui.add_space(4.0);
        control(ui);
    }
    ui.add_space(8.0);
}
