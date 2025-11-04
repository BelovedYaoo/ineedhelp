use eframe::egui;
use egui_json_tree::{
    DefaultExpand, JsonTree, JsonTreeMaxWidth, JsonTreeStyle, 
    JsonTreeWrapping, JsonTreeWrappingConfig,
};
use serde::Serialize;

use crate::context_menu::show_context_menu;
use crate::edit::{apply_edits, EditDialog, JsonEdit};
use crate::ui::{install_cjk_fonts, try_fill_from_clipboard};

pub struct JsonFmtApp {
    input: String,
    error: Option<String>,
    indent_spaces: usize,
    tried_clipboard_once: bool,
    last_json: Option<serde_json::Value>,
    fonts_loaded: bool,
    search_input: String,
    pending_edits: Vec<JsonEdit>,
    edit_dialog: Option<EditDialog>,
}

impl Default for JsonFmtApp {
    fn default() -> Self {
        let mut app = Self {
            input: String::new(),
            error: None,
            indent_spaces: 2,
            tried_clipboard_once: false,
            last_json: None,
            fonts_loaded: false,
            search_input: String::new(),
            pending_edits: Vec::new(),
            edit_dialog: None,
        };
        
        // 尝试从剪贴板填充
        if let Some(value) = try_fill_from_clipboard(&mut app.input, app.indent_spaces) {
            app.last_json = Some(value);
        }
        app.tried_clipboard_once = true;
        
        app
    }
}

impl JsonFmtApp {
    /// 格式化 JSON 字符串
    fn format_json(&self, json_str: &str) -> Result<String, String> {
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(v) => {
                let mut buf = Vec::new();
                let indent = " ".repeat(self.indent_spaces);
                let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                v.serialize(&mut ser).map_err(|e| e.to_string())?;
                String::from_utf8(buf).map_err(|e| e.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    }

    /// 应用待处理的编辑操作
    fn apply_edits(&mut self) {
        if let Some(ref mut value) = self.last_json {
            if let Some(formatted) = apply_edits(value, &mut self.pending_edits) {
                self.input = formatted;
            }
        }
    }

    /// 显示编辑对话框
    fn show_edit_dialog(&mut self, ctx: &egui::Context) {
        if let Some(dialog) = &mut self.edit_dialog {
            let mut should_close = false;
            let mut should_save = false;

            egui::Window::new(match dialog {
                EditDialog::EditValue { .. } => "✏️ 编辑值",
                EditDialog::EditKey { .. } => "✏️ 编辑键",
            })
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                match dialog {
                    EditDialog::EditValue { input, .. } => {
                        ui.label("输入新值：");
                        ui.text_edit_singleline(input);
                    }
                    EditDialog::EditKey { input, .. } => {
                        ui.label("输入新键名：");
                        ui.text_edit_singleline(input);
                    }
                }

                ui.horizontal(|ui| {
                    if ui.button("✅ 保存").clicked() {
                        should_save = true;
                        should_close = true;
                    }
                    if ui.button("❌ 取消").clicked() {
                        should_close = true;
                    }
                });
            });

            if should_save {
                match dialog {
                    EditDialog::EditValue { pointer, input } => {
                        self.pending_edits.push(JsonEdit::EditValue {
                            pointer: pointer.clone(),
                            new_value: input.clone(),
                        });
                    }
                    EditDialog::EditKey { object_pointer, old_key, input } => {
                        self.pending_edits.push(JsonEdit::EditObjectKey {
                            object_pointer: object_pointer.clone(),
                            old_key: old_key.clone(),
                            new_key: input.clone(),
                        });
                    }
                }
            }

            if should_close {
                self.edit_dialog = None;
            }
        }
    }
}

impl eframe::App for JsonFmtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 延迟加载 CJK 字体
        if !self.fonts_loaded {
            install_cjk_fonts(ctx);
            self.fonts_loaded = true;
        }

        // 显示编辑对话框
        self.show_edit_dialog(ctx);

        // 顶部工具栏
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("格式化").clicked() {
                    self.error = None;
                    match serde_json::from_str::<serde_json::Value>(&self.input) {
                        Ok(v) => {
                            let mut buf = Vec::new();
                            let indent = " ".repeat(self.indent_spaces);
                            let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                            let _ = v.serialize(&mut ser);
                            self.input = String::from_utf8(buf).unwrap_or_default();
                            if v.is_object() || v.is_array() {
                                self.last_json = Some(v);
                            }
                        }
                        Err(e) => {
                            self.error = Some(e.to_string());
                        }
                    }
                }

                if ui.button("压缩").clicked() {
                    self.error = None;
                    match serde_json::from_str::<serde_json::Value>(&self.input) {
                        Ok(v) => {
                            self.input = serde_json::to_string(&v).unwrap_or_default();
                            if v.is_object() || v.is_array() {
                                self.last_json = Some(v);
                            }
                        }
                        Err(e) => {
                            self.error = Some(e.to_string());
                        }
                    }
                }

                if ui.button("清空").clicked() {
                    self.input.clear();
                    self.error = None;
                }
                if ui.button("复制").clicked() {
                    ui.ctx().copy_text(self.input.clone());
                }

                ui.separator();
                ui.label("缩进：");
                egui::ComboBox::from_id_salt("indent_top")
                    .selected_text(format!("{} 空格", self.indent_spaces))
                    .show_ui(ui, |ui| {
                        for s in [0, 1, 2, 3, 4].iter().copied() {
                            ui.selectable_value(&mut self.indent_spaces, s, format!("{} 空格", s));
                        }
                    });

                if let Some(err) = &self.error {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(200, 60, 60), format!("错误：{}", err));
                }
            });
        });

        // 中央左右分栏
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.columns(2, |columns| {
                // 左列：原始输入
                let left = &mut columns[0];
                left.label("原始 JSON：");
                let available_height = left.available_height();
                
                let edit_resp = egui::ScrollArea::vertical()
                    .id_salt("input_scroll")
                    .auto_shrink(false)
                    .show(left, |ui| {
                        // 行号列宽度
                        let line_number_width = 50.0;
                        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
                        
                        // 使用 allocate_ui 预留行号空间
                        let total_width = ui.available_width();
                        
                        ui.horizontal_top(|ui| {
                            // 预留行号区域
                            let line_num_rect = ui.allocate_space(egui::vec2(line_number_width, available_height)).1;
                            
                            ui.add_space(4.0); // 小间距
                            
                            // 文本编辑器
                            let text_edit_output = egui::TextEdit::multiline(&mut self.input)
                                .desired_width(total_width - line_number_width - 4.0)
                                .min_size(egui::vec2(total_width - line_number_width - 4.0, available_height))
                                .code_editor()
                                .hint_text("在此粘贴或输入原始 JSON")
                                .show(ui);
                            
                            // 绘制行号
                            let galley = &text_edit_output.galley;
                            let galley_pos = text_edit_output.galley_pos;
                            
                            let mut current_line = 1;
                            let mut y_offset = galley_pos.y;
                            
                            for (row_idx, row) in galley.rows.iter().enumerate() {
                                // 检查这一行是否是新的逻辑行的开始
                                let is_new_line = row_idx == 0 || {
                                    if row_idx > 0 {
                                        let prev_row = &galley.rows[row_idx - 1];
                                        prev_row.ends_with_newline
                                    } else {
                                        false
                                    }
                                };
                                
                                if is_new_line {
                                    // 绘制行号在预留区域
                                    ui.painter().text(
                                        egui::pos2(line_num_rect.right() - 4.0, y_offset),
                                        egui::Align2::RIGHT_TOP,
                                        format!("{}", current_line),
                                        font_id.clone(),
                                        egui::Color32::from_gray(128),
                                    );
                                    current_line += 1;
                                }
                                
                                y_offset += row.rect().height();
                            }
                            
                            text_edit_output
                        }).inner
                    });

                if edit_resp.inner.response.changed() {
                    // 尝试自动格式化
                    match self.format_json(&self.input) {
                        Ok(formatted) => {
                            self.input = formatted;
                            self.error = None;
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.input) {
                                if v.is_object() || v.is_array() {
                                    self.last_json = Some(v);
                                }
                            }
                        }
                        Err(e) => {
                            self.error = Some(e);
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.input) {
                                if v.is_object() || v.is_array() {
                                    self.last_json = Some(v);
                                }
                            }
                        }
                    }
                }

                // 右列：JSON 树解析展示
                let right = &mut columns[1];
                right.horizontal(|ui| {
                    ui.label("解析树：");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_input)
                            .hint_text("🔍 搜索...")
                            .desired_width(f32::INFINITY)
                    );
                });

                let to_show = if self.input.trim().is_empty() {
                    self.last_json.as_ref()
                } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.input) {
                    if v.is_object() || v.is_array() {
                        self.last_json = Some(v);
                        self.last_json.as_ref()
                    } else {
                        self.last_json.as_ref()
                    }
                } else {
                    self.last_json.as_ref()
                };

                if let Some(v) = to_show {
                    let available_height = right.available_height();
                    let pending_edits = &mut self.pending_edits;
                    let edit_dialog = &mut self.edit_dialog;
                    let search_input = &self.search_input;
                    
                    egui::ScrollArea::vertical()
                        .id_salt("tree_scroll")
                        .auto_shrink(false)
                        .max_height(available_height)
                        .show(right, |ui| {
                            let default_expand = if search_input.is_empty() {
                                DefaultExpand::ToLevel(3)
                            } else {
                                DefaultExpand::SearchResultsOrAll(search_input)
                            };

                            // 配置文本换行：最多显示2行，使用可用宽度，不在任意位置断行
                            let wrapping = JsonTreeWrapping {
                                max_rows: 2,
                                max_width: JsonTreeMaxWidth::UiAvailableWidth,
                                break_anywhere: false,
                            };
                            let wrapping_config = JsonTreeWrappingConfig {
                                value_when_root: wrapping,
                                value_with_expanded_parent: wrapping,
                                value_in_collapsed_root: wrapping,
                            };

                            JsonTree::new("json_tree", v)
                                .default_expand(default_expand)
                                .style(JsonTreeStyle::new().wrapping_config(wrapping_config))
                                .on_render(|ui, context| {
                                    let pointer = context.pointer().to_json_pointer_string();
                                    show_context_menu(ui, context, pointer, pending_edits, edit_dialog);
                                })
                                .show(ui);
                        });

                    // 应用所有待处理的编辑
                    self.apply_edits();
                } else {
                    right.label("无解析结果");
                }
            });
        });
    }
}
