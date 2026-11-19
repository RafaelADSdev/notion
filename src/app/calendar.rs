use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

const MONTHS: [&str; 12] = [
    "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
    "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Date {
    year: u32,
    month: u32,
    day: u32,
}

impl Date {
    fn today() -> Self {
        let mut days = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() / 86_400;
        let mut year = 1970;
        let mut month = 1;
        while days >= u64::from(days_in_month(year, month)) {
            days -= u64::from(days_in_month(year, month));
            if month == 12 { year += 1; month = 1; } else { month += 1; }
        }
        Self { year, month, day: days as u32 + 1 }
    }

    fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let (year, month, day) = if let Some((year, rest)) = text.split_once('-') {
            let (month, day) = rest.split_once('-')?;
            (year, month, day)
        } else {
            let (day, rest) = text.split_once('/')?;
            let (month, year) = rest.split_once('/')?;
            (year, month, day)
        };
        if year.len() != 4 { return None; }
        let (year, month, day) = (year.parse().ok()?, month.parse().ok()?, day.parse().ok()?);
        if year == 0 || !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
            return None;
        }
        Some(Self { year, month, day })
    }

    fn label(self) -> String {
        format!("{:02}/{:02}/{:04}", self.day, self.month, self.year)
    }
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn first_weekday(year: u32, month: u32) -> u32 {
    let previous_year = year - 1;
    let days_before_year = 365 * previous_year + previous_year / 4 - previous_year / 100 + previous_year / 400;
    let days_before_month: u32 = (1..month).map(|month| days_in_month(year, month)).sum();
    (days_before_year + days_before_month) % 7
}

pub(super) struct Calendar {
    collection: Option<Id>,
    date: Date,
}

impl Default for Calendar {
    fn default() -> Self {
        Self { collection: None, date: Date::today() }
    }
}

impl Calendar {
    fn change_month(&mut self, offset: i32) {
        let month = (self.date.year as i32 - 1) * 12 + self.date.month as i32 - 1;
        let next = (month + offset).clamp(0, 9999 * 12 - 1) as u32;
        self.date = Date { year: next / 12 + 1, month: next % 12 + 1, day: 1 };
    }
}

impl Margem {
    pub(super) fn calendar_view(
        &mut self,
        ctx: &Context,
        ui: &mut Ui,
        p: Palette,
        id: Id,
        rows: &[Id],
        status: Status,
    ) {
        let today = Date::today();
        if self.calendar.collection != Some(id) {
            self.calendar = Calendar {
                collection: Some(id),
                date: today,
            };
        }
        ui.horizontal_wrapped(|ui| {
            let date = self.calendar.date;
            ui.label(p.heading(
                format!("{} de {}", MONTHS[date.month as usize - 1], date.year),
                17.0,
            ));
            ui.horizontal(|ui| {
                if action(ui, p, "‹", false)
                    .on_hover_text("Mês anterior")
                    .clicked()
                {
                    self.calendar.change_month(-1);
                }
                if action(ui, p, "Hoje", false).clicked() {
                    self.calendar.date = today;
                }
                if action(ui, p, "›", false)
                    .on_hover_text("Próximo mês")
                    .clicked()
                {
                    self.calendar.change_month(1);
                }
            });
        });
        let date = self.calendar.date;
        let mut entries = BTreeMap::<u32, Vec<Id>>::new();
        let mut undated = Vec::new();
        for &row in rows {
            let item = self.space.page(row).unwrap();
            match Date::parse(&item.date) {
                Some(entry) if entry.year == date.year && entry.month == date.month => {
                    entries.entry(entry.day).or_default().push(row);
                }
                None => undated.push(row),
                _ => {}
            }
        }

        let offset = first_weekday(date.year, date.month);
        let days = days_in_month(date.year, date.month);
        let mut open = None;
        let cell_height = if self.space.settings.compact {
            31.0
        } else {
            37.0
        };
        Frame::new()
            .stroke(egui::Stroke::new(1.0_f32, p.line))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                let width = ui.available_width() / 7.0;
                let line = egui::Stroke::new(1.0_f32, p.line);
                Frame::new().fill(p.side).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for weekday in ["Seg", "Ter", "Qua", "Qui", "Sex", "Sáb", "Dom"] {
                            ui.add_sized(
                                [width, 27.0],
                                egui::Label::new(p.label(weekday).size(12.0).strong()),
                            );
                        }
                    });
                });
                for week in 0..(days + offset).div_ceil(7) {
                    ui.horizontal(|ui| {
                        for column in 0..7 {
                            let cell = week * 7 + column;
                            let in_month = (offset..offset + days).contains(&cell);
                            let size = vec2(width, cell_height);
                            let rect = if in_month {
                                let day = cell - offset + 1;
                                let entry_date = Date { day, ..date };
                                let count = entries.get(&day).map_or(0, Vec::len);
                                let selected = self.calendar.date == entry_date;
                                let response = day_cell(
                                    ui,
                                    p,
                                    size,
                                    entry_date,
                                    count,
                                    selected,
                                    entry_date == today,
                                );
                                if response.clicked() {
                                    self.calendar.date.day = day;
                                }
                                response.rect
                            } else {
                                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                                ui.painter().rect_filled(rect, 0, p.side);
                                rect
                            };
                            ui.painter()
                                .line_segment([rect.left_top(), rect.right_top()], line);
                            if column > 0 {
                                ui.painter()
                                    .line_segment([rect.left_top(), rect.left_bottom()], line);
                            }
                        }
                    });
                }
            });
        ui.add_space(7.0);
        let selected = self.calendar.date;
        let add = ui
            .horizontal_wrapped(|ui| {
                ui.label(p.heading(selected.label(), 15.0));
                action(ui, p, "+ Adicionar neste dia", false).clicked()
            })
            .inner;
        ui.separator();
        Frame::new()
            .inner_margin(egui::Margin::symmetric(4, 1))
            .show(ui, |ui| {
                if let Some(rows) = entries.get(&selected.day) {
                    for &row in rows {
                        let item = self.space.page(row).unwrap();
                        if page_row(ui, p, item.title(), item.kind, "›").clicked() {
                            open = Some(row);
                        }
                    }
                } else {
                    ui.label(p.label("Nenhum item neste dia."));
                }
            });
        if !undated.is_empty() {
            ui.add_space(8.0);
            section(ui, p, "Sem data válida");
            ui.label(p.label("Use DD/MM/AAAA ou AAAA-MM-DD para colocar o item no calendário."));
            for row in undated {
                let item = self.space.page(row).unwrap();
                if page_row(ui, p, item.title(), item.kind, "›").clicked() {
                    open = Some(row);
                }
            }
        }
        if let Some(row) = open {
            self.select(ctx, View::Page(row));
        }
        if add {
            let item = self.add_collection_item(ctx, id, status);
            self.space.page_mut(item).unwrap().date = self.calendar.date.label();
        }
    }
}

fn day_cell(
    ui: &mut Ui,
    p: Palette,
    size: Vec2,
    date: Date,
    count: usize,
    selected: bool,
    today: bool,
) -> egui::Response {
    let detail = match count {
        0 => format!("{} · Sem itens", date.label()),
        1 => format!("{} · 1 item", date.label()),
        _ => format!("{} · {count} itens", date.label()),
    };
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    response.widget_info(|| {
        egui::WidgetInfo::selected(
            egui::WidgetType::SelectableLabel,
            ui.is_enabled(),
            selected,
            &detail,
        )
    });
    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(
        rect,
        0,
        if selected || response.hovered() {
            p.tint
        } else {
            p.white
        },
    );
    if response.has_focus() || selected {
        painter.rect_stroke(
            rect.shrink(2.0),
            3,
            egui::Stroke::new(1.0_f32, p.accent),
            egui::StrokeKind::Inside,
        );
    }
    let center = rect.left_center() + vec2(16.0, 0.0);
    if today {
        painter.circle_filled(center, 10.0, p.accent);
    }
    painter.text(
        center,
        Align2::CENTER_CENTER,
        date.day.to_string(),
        FontId::proportional(12.0),
        if today { p.on_accent } else { p.ink },
    );
    if count > 0 {
        let position = rect.right_center() - vec2(10.0, 0.0);
        if rect.width() >= 58.0 {
            painter.text(
                position,
                Align2::RIGHT_CENTER,
                count.to_string(),
                FontId::proportional(11.0),
                p.accent,
            );
        } else {
            painter.circle_filled(position, 2.5, p.accent);
        }
    }
    response
        .on_hover_text(detail)
        .on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_dates_and_weekdays() {
        assert_eq!(first_weekday(2026, 11), 6);
        assert_eq!(Date::parse("2024-02-29"), Date::parse("29/02/2024"));
        assert_eq!(Date::parse("29/02/2024").unwrap().day, 29);
        assert_eq!(Date::parse("1900-02-29"), None);
        assert_eq!(Date::parse("31/04/2026"), None);
    }
}
