use std::time::Duration;

use eframe::egui::{self, Event, ImeEvent, Key, Modifiers};
use winit::platform::android::activity::{
    input::{ImeOptions, InputType, TextInputAction, TextInputState, TextSpan},
    AndroidApp,
};

use crate::app::Margem;

// An empty IME buffer cannot report a backspace. Leave one invisible character
// before the cursor so Android can distinguish deleting from doing nothing.
const IME_START: &str = "\u{200b}";

#[no_mangle]
fn android_main(android: AndroidApp) {
    let path = android.internal_data_path()
        .map(|dir| dir.join("workspace.json"))
        .ok_or_else(|| "A pasta de dados do Android não está disponível.".to_owned());
    let options = eframe::NativeOptions {
        android_app: Some(android.clone()),
        ..Default::default()
    };
    if let Err(error) = eframe::run_native("Margem", options, Box::new(move |creation| {
        let workspace = Margem::with_storage(&creation.egui_ctx, None, || path);
        Ok(Box::new(MobileMargem { workspace, android, field: None, last_input: None }))
    })) {
        eprintln!("Não foi possível abrir o Margem: {error}");
    }
}

struct MobileMargem {
    workspace: Margem,
    android: AndroidApp,
    field: Option<egui::Id>,
    last_input: Option<(String, bool)>,
}

impl MobileMargem {
    fn reset_input(&self) {
        self.android.set_text_input_state(TextInputState {
            text: IME_START.to_owned(),
            selection: TextSpan { start: 1, end: 1 },
            compose_region: None,
        });
    }
}

impl eframe::App for MobileMargem {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.workspace.update(ctx, frame);
        let field = ctx.output(|output| output.ime.is_some())
            .then(|| ctx.memory(|memory| memory.focused())).flatten();
        if field != self.field {
            self.field = field;
            self.last_input = Some((IME_START.to_owned(), false));
            if field.is_some() {
                self.android.set_ime_editor_info(
                    InputType::TYPE_CLASS_TEXT | InputType::TYPE_TEXT_FLAG_MULTI_LINE,
                    TextInputAction::None,
                    ImeOptions::IMG_FLAG_NO_EXTRACT_UI,
                );
                self.reset_input();
            }
        }
        if field.is_some() {
            ctx.request_repaint_after(Duration::from_millis(32));
        }
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if self.field.is_none() {
            return;
        }
        // Winit handles physical keys but does not forward GameTextInput updates.
        let state = self.android.text_input_state();
        let composing = state.compose_region.is_some();
        if self.last_input.as_ref().is_some_and(|(text, pending)| *text == state.text && *pending == composing) {
            return;
        }
        let had_composition = self.last_input.as_ref().is_some_and(|(_, pending)| *pending);
        let committed = state.text.strip_prefix(IME_START).unwrap_or(&state.text);
        if composing {
            if !had_composition { input.events.push(Event::Ime(ImeEvent::Enabled)); }
            input.events.push(Event::Ime(ImeEvent::Preedit(committed.to_owned())));
        } else {
            if had_composition {
                input.events.push(Event::Ime(ImeEvent::Commit(committed.to_owned())));
            } else if !committed.is_empty() {
                input.events.push(Event::Text(committed.to_owned()));
            } else if state.text.is_empty() {
                for pressed in [true, false] {
                    input.events.push(Event::Key {
                        key: Key::Backspace, physical_key: None, pressed, repeat: false,
                        modifiers: Modifiers::NONE,
                    });
                }
            }
            if state.text != IME_START { self.reset_input(); }
        }
        self.last_input = Some((state.text, composing));
    }

    fn on_exit(&mut self, gl: Option<&eframe::glow::Context>) {
        self.workspace.on_exit(gl);
    }
}
