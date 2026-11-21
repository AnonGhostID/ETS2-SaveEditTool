//! Global modal overlays: unsaved-changes prompt and destructive confirm.

use eframe::egui::{self, RichText};

use crate::gui::components;
use crate::gui::screens::ContextAction;
use crate::gui::shell::{AppShell, ConfirmAction};
use crate::gui::theme;

pub fn render(shell: &mut AppShell, ctx: &egui::Context) {
    if let Some(action) = shell.unsaved_prompt.clone() {
        render_unsaved_prompt(shell, ctx, &action);
    } else if let Some(confirm) = shell.confirm.clone() {
        render_confirm(shell, ctx, &confirm);
    }
}

fn render_unsaved_prompt(shell: &mut AppShell, ctx: &egui::Context, action: &ContextAction) {
    let action_text = match action {
        ContextAction::SwitchSave(_) => "switching saves",
        ContextAction::SwitchProfile(_) => "switching profiles",
        ContextAction::SwitchGame(_) => "switching games",
        ContextAction::Reload => "reloading",
        ContextAction::Exit => "closing the application",
    };
    let dirty = crate::gui::data::session::dirty_paths(&shell.data).len();
    let mut choice: Option<bool> = None; // Some(true) = save, Some(false) = discard
    let mut cancelled = ctx.input(|input| input.key_pressed(egui::Key::Escape));

    let response = components::modal(ctx, "Unsaved changes", |ui| {
        ui.label(
            RichText::new(format!(
                "You have unsaved edits in {dirty} save file(s). Save them before {action_text}?"
            ))
            .size(12.5)
            .color(theme::TEXT),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let save = ui.push_id(components::modal_focus_id(0), |ui| {
                components::primary_button(ui, "Save")
            });
            if save.inner.clicked() {
                choice = Some(true);
            }
            let discard = ui.push_id(components::modal_focus_id(1), |ui| {
                components::button(ui, "Discard")
            });
            if discard.inner.clicked() {
                choice = Some(false);
            }
            let cancel = ui.push_id(components::modal_focus_id(2), |ui| {
                components::button(ui, "Cancel")
            });
            if cancel.inner.clicked() {
                cancelled = true;
            }
        });
    });

    // A click outside the modal window dismisses it as Cancel.
    if response.is_none() {
        cancelled = true;
    }

    if cancelled {
        shell.unsaved_prompt = None;
        shell.clear_pending();
        shell.restore_focus(ctx);
    } else if let Some(save_first) = choice {
        shell.resolve_unsaved_prompt(save_first);
    }
}

fn render_confirm(shell: &mut AppShell, ctx: &egui::Context, confirm: &ConfirmAction) {
    let (title, message) = match confirm {
        ConfirmAction::UndoLast => (
            "Confirm undo",
            "Undo restores the previous save from a backup. Staged edits must be saved or discarded first. Continue?",
        ),
    };
    let mut answer: Option<bool> = None;
    let mut cancelled = ctx.input(|input| input.key_pressed(egui::Key::Escape));
    let response = components::modal(ctx, title, |ui| {
        ui.label(RichText::new(message).size(12.5).color(theme::TEXT).wrap());
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let yes = ui.push_id(components::modal_focus_id(0), |ui| {
                components::primary_button(ui, "Yes")
            });
            if yes.inner.clicked() {
                answer = Some(true);
            }
            let no = ui.push_id(components::modal_focus_id(1), |ui| {
                components::button(ui, "No")
            });
            if no.inner.clicked() {
                answer = Some(false);
            }
        });
    });
    if response.is_none() {
        cancelled = true;
    }
    if cancelled {
        answer = Some(false);
    }
    if let Some(yes) = answer {
        shell.confirm = None;
        if yes {
            finish_undo(shell);
        }
        shell.restore_focus(ctx);
    }
}

fn finish_undo(shell: &mut AppShell) {
    match crate::gui::data::session::undo_last(&shell.data) {
        Ok(()) => {
            shell.push_toast("Last save change undone from backup.", false);
            shell.screen_states = crate::gui::screens::ScreenStates::default();
            shell.refresh_safety();
            shell.refresh_context();
        }
        Err(err) => shell.push_error(format!("Undo failed: {}", err.message)),
    }
}
