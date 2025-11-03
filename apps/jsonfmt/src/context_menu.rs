use eframe::egui;
use egui_json_tree::{
    pointer::JsonPointerSegment,
    render::{DefaultRender, RenderContext},
};

use crate::edit::{EditDialog, JsonEdit};

/// 显示右键菜单
pub fn show_context_menu(
    ui: &mut egui::Ui,
    context: RenderContext<serde_json::Value>,
    pointer: String,
    pending_edits: &mut Vec<JsonEdit>,
    edit_dialog: &mut Option<EditDialog>,
) {
    context
        .render_default(ui)
        .on_hover_cursor(egui::CursorIcon::ContextMenu)
        .context_menu(|ui| {
            // 复制功能
            if !pointer.is_empty() && ui.button("📋 复制路径").clicked() {
                ui.ctx().copy_text(pointer.clone());
                ui.close();
            }

            if ui.button("📄 复制内容").clicked() {
                if let Ok(pretty_str) = serde_json::to_string_pretty(context.value()) {
                    ui.ctx().copy_text(pretty_str);
                }
                ui.close();
            }

            // 添加功能
            match context {
                RenderContext::Property(mut ctx) => {
                    let has_edit_options = ctx.value.is_object() || ctx.value.is_array() || ctx.pointer.parent().is_some();
                    if has_edit_options {
                        ui.separator();
                    }

                    if ctx.value.is_object() && ui.button("➕ 添加到对象").clicked() {
                        pending_edits.push(JsonEdit::AddToObject {
                            pointer: pointer.clone(),
                        });
                        if let Some(ref mut state) = ctx.collapsing_state {
                            state.set_open(true);
                        }
                        ui.close();
                    }

                    if ctx.value.is_array() && ui.button("➕ 添加到数组").clicked() {
                        pending_edits.push(JsonEdit::AddToArray {
                            pointer: pointer.clone(),
                        });
                        if let Some(ref mut state) = ctx.collapsing_state {
                            state.set_open(true);
                        }
                        ui.close();
                    }

                    // 编辑键功能
                    if let (Some(parent), JsonPointerSegment::Key(key)) = (ctx.pointer.parent(), ctx.property) {
                        if ui.button("✏ 编辑键").clicked() {
                            *edit_dialog = Some(EditDialog::EditKey {
                                object_pointer: parent.to_json_pointer_string(),
                                old_key: key.to_string(),
                                input: key.to_string(),
                            });
                            ui.close();
                        }
                    }

                    // 删除功能
                    if let Some(parent) = ctx.pointer.parent() {
                        if ui.button("🗑 删除").clicked() {
                            let edit = match ctx.property {
                                JsonPointerSegment::Key(key) => JsonEdit::DeleteFromObject {
                                    object_pointer: parent.to_json_pointer_string(),
                                    key: key.to_string(),
                                },
                                JsonPointerSegment::Index(idx) => JsonEdit::DeleteFromArray {
                                    array_pointer: parent.to_json_pointer_string(),
                                    idx,
                                },
                            };
                            pending_edits.push(edit);
                            ui.close();
                        }
                    }
                }
                RenderContext::BaseValue(ctx) => {
                    ui.separator();
                    
                    // 编辑值功能
                    if ui.button("✏ 编辑值").clicked() {
                        *edit_dialog = Some(EditDialog::EditValue {
                            pointer: pointer.clone(),
                            input: ctx.value.to_string(),
                        });
                        ui.close();
                    }

                    // 基础值的删除功能
                    if let (Some(parent), Some(segment)) = (ctx.pointer.parent(), ctx.pointer.last()) {
                        if ui.button("🗑 删除").clicked() {
                            let edit = match segment {
                                JsonPointerSegment::Key(key) => JsonEdit::DeleteFromObject {
                                    object_pointer: parent.to_json_pointer_string(),
                                    key: key.to_string(),
                                },
                                JsonPointerSegment::Index(idx) => JsonEdit::DeleteFromArray {
                                    array_pointer: parent.to_json_pointer_string(),
                                    idx: *idx,
                                },
                            };
                            pending_edits.push(edit);
                            ui.close();
                        }
                    }
                }
                RenderContext::ExpandableDelimiter(ctx) => {
                    // 在分隔符上也可以添加
                    ui.separator();
                    if ctx.value.is_object() && ui.button("➕ 添加到对象").clicked() {
                        pending_edits.push(JsonEdit::AddToObject {
                            pointer: pointer.clone(),
                        });
                        ctx.collapsing_state.set_open(true);
                        ui.close();
                    }

                    if ctx.value.is_array() && ui.button("➕ 添加到数组").clicked() {
                        pending_edits.push(JsonEdit::AddToArray {
                            pointer: pointer.clone(),
                        });
                        ctx.collapsing_state.set_open(true);
                        ui.close();
                    }
                }
            }
        });
}
