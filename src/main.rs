#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod model;
mod templates;
mod ui;

use std::path::PathBuf;

use eframe::egui::ViewportBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let screenshot = match args.next() {
        None => None,
        Some(option) if option == "--screenshot" => Some(PathBuf::from(
            args.next().ok_or("Informe o caminho de saída para --screenshot.")?,
        )),
        Some(_) => return Err("Opção desconhecida.".into()),
    };
    if args.next().is_some() {
        return Err("Informe um único caminho para a captura.".into());
    }

    #[cfg(debug_assertions)]
    let measurements = screenshot.as_ref().map(|_| app::verify_mechanics());

    let mut viewport = ViewportBuilder::default()
        .with_inner_size([1160.0, 740.0])
        .with_min_inner_size([680.0, 490.0])
        .with_icon(ui::app_icon());
    if screenshot.is_some() {
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        { viewport = viewport.with_window_level(eframe::egui::WindowLevel::AlwaysOnBottom); }
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    #[cfg(target_os = "macos")]
    let options = if screenshot.is_some() {
        use winit::platform::macos::EventLoopBuilderExtMacOS;
        eframe::NativeOptions {
            event_loop_builder: Some(Box::new(|builder| {
                builder.with_activate_ignoring_other_apps(false);
            })),
            ..options
        }
    } else { options };
    let shot = screenshot.clone();
    eframe::run_native("Margem", options, Box::new(move |creation| {
        Ok(Box::new(app::Margem::new(&creation.egui_ctx, shot)))
    }))?;
    if let Some(path) = screenshot {
        if !path.is_file() {
            return Err("O aplicativo encerrou sem salvar a captura.".into());
        }
        #[cfg(debug_assertions)]
        if let Some(measurements) = measurements {
            std::fs::write(
                path.with_extension("mechanics.json"),
                serde_json::to_vec_pretty(&measurements)?,
            )?;
        }
    }
    Ok(())
}
