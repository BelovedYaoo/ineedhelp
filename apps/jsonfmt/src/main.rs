#![windows_subsystem = "windows"]

use eframe::egui;
use egui_json_tree::{JsonTree, DefaultExpand};
use serde::Serialize;
use arboard::Clipboard;

struct JsonFmtApp {
    input: String,
    error: Option<String>,
    indent_spaces: usize,
    font_points: f32,
    dark_mode: bool,
    tried_clipboard_once: bool,
    last_json: Option<serde_json::Value>,
}

impl Default for JsonFmtApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            error: None,
            indent_spaces: 2,
            font_points: 14.0,
            dark_mode: false,
            tried_clipboard_once: false,
            last_json: None,
        }
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
}

fn install_cjk_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};

    let mut fonts = FontDefinitions::default();

    // Try to load a common CJK font from Windows; silently skip if not found
    let candidates = [
        r"C:\\Windows\\Fonts\\msyh.ttc",   // Microsoft YaHei
        r"C:\\Windows\\Fonts\\msyh.ttf",
        r"C:\\Windows\\Fonts\\simhei.ttf", // SimHei
        r"C:\\Windows\\Fonts\\simsun.ttc", // SimSun
    ];

    let mut loaded_any = false;
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.font_data.insert("cjk".to_owned(), FontData::from_owned(bytes).into());
            loaded_any = true;
            break;
        }
    }

    if loaded_any {
        if let Some(list) = fonts.families.get_mut(&FontFamily::Proportional) {
            list.insert(0, "cjk".to_owned());
        }
        if let Some(list) = fonts.families.get_mut(&FontFamily::Monospace) {
            list.insert(0, "cjk".to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

impl eframe::App for JsonFmtApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用全局字号与主题
        let mut style = (*ctx.style()).clone();
        for v in style.text_styles.values_mut() { v.size = self.font_points; }
        ctx.set_style(style);
        ctx.set_visuals(if self.dark_mode { egui::Visuals::dark() } else { egui::Visuals::light() });

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
                            if v.is_object() || v.is_array() { self.last_json = Some(v); }
                        }
                        Err(e) => { self.error = Some(e.to_string()); }
                    }
                }

                if ui.button("压缩").clicked() {
                    self.error = None;
                    match serde_json::from_str::<serde_json::Value>(&self.input) {
                        Ok(v) => {
                            self.input = serde_json::to_string(&v).unwrap_or_default();
                            if v.is_object() || v.is_array() { self.last_json = Some(v); }
                        }
                        Err(e) => { self.error = Some(e.to_string()); }
                    }
                }

                if ui.button("清空").clicked() { self.input.clear(); self.error = None; }
                if ui.button("复制").clicked() { ui.ctx().copy_text(self.input.clone()); }

                ui.separator();
                ui.label("缩进：");
                egui::ComboBox::from_id_salt("indent_top")
                    .selected_text(format!("{} 空格", self.indent_spaces))
                    .show_ui(ui, |ui| {
                        for s in [0,1,2,3,4].iter().copied() {
                            ui.selectable_value(&mut self.indent_spaces, s, format!("{} 空格", s));
                        }
                    });

                ui.label("字号：");
                ui.add(egui::Slider::new(&mut self.font_points, 10.0..=20.0).step_by(1.0).show_value(false));

                ui.label("主题：");
                let theme_label = if self.dark_mode { "暗" } else { "亮" };
                ui.toggle_value(&mut self.dark_mode, theme_label);

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
                let edit_resp = egui::ScrollArea::both()
                    .id_salt("input_scroll")
                    .auto_shrink([false, false])
                    .show(left, |ui| {
                        egui::TextEdit::multiline(&mut self.input)
                            .desired_rows(28)
                            .code_editor()
                            .hint_text("在此粘贴或输入原始 JSON")
                            .show(ui)
                    });

                if edit_resp.inner.response.changed() {
                    // 尝试自动格式化
                    match self.format_json(&self.input) {
                        Ok(formatted) => {
                            self.input = formatted;
                            self.error = None;
                            // 更新 last_json 用于树视图
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.input) {
                                if v.is_object() || v.is_array() {
                                    self.last_json = Some(v);
                                }
                            }
                        }
                        Err(e) => {
                            // 解析失败时，仍然尝试更新树视图
                            self.error = Some(e);
                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.input) {
                                if v.is_object() || v.is_array() {
                                    self.last_json = Some(v);
                                }
                            }
                        }
                    }
                }

                // 右列：JSON 树解析展示（基于最近一次成功解析）
                let right = &mut columns[1];
                right.label("解析树：");
                let to_show = if self.input.trim().is_empty() {
                    self.last_json.as_ref()
                } else if let Ok(v) = serde_json::from_str::<serde_json::Value>(&self.input) {
                    if v.is_object() || v.is_array() {
                        self.last_json = Some(v);
                        self.last_json.as_ref()
                    } else {
                        // 标量根：不更新，不提示
                        self.last_json.as_ref()
                    }
                } else {
                    self.last_json.as_ref()
                };

                if let Some(v) = to_show {
                    egui::ScrollArea::both().id_salt("tree_scroll").auto_shrink([false, false]).show(right, |ui| {
                        let _resp = JsonTree::new("json_tree", v)
                            .default_expand(DefaultExpand::All)
                            .show(ui);
                    });
                } else {
                    right.label("无解析结果");
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "JSON 格式化",
        options,
        Box::new(|cc| {
            install_cjk_fonts(&cc.egui_ctx);
            let mut app = JsonFmtApp::default();
            try_fill_from_clipboard(&mut app);
            Ok(Box::new(app))
        }),
    )
}

fn try_fill_from_clipboard(app: &mut JsonFmtApp) {
    if app.tried_clipboard_once { return; }
    app.tried_clipboard_once = true;

    // 不覆盖用户已输入内容
    if !app.input.trim().is_empty() { return; }

    // Best-effort: 读取当前剪贴板文本，若为 JSON 则格式化并填充
    if let Ok(mut cb) = Clipboard::new() {
        if let Ok(text) = cb.get_text() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                if value.is_object() || value.is_array() {
                    let indent = " ".repeat(app.indent_spaces);
                    let mut buf = Vec::new();
                    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                    let _ = value.serialize(&mut ser);
                    app.input = String::from_utf8(buf).unwrap_or(text);
                    app.error = None;
                    app.last_json = Some(value);
                    return;
                }
            }
        }
    }

    // Windows: 尝试读取剪贴板历史（最多 10 条），失败不影响运行
    #[cfg(target_os = "windows")]
    {
        use windows::ApplicationModel::DataTransfer::Clipboard;
        use windows::ApplicationModel::DataTransfer::StandardDataFormats;

        let mut try_history = || -> windows::core::Result<()> {
            // 若系统未开启历史记录，此处将返回错误或空，直接忽略
            let result = Clipboard::GetHistoryItemsAsync()?.GetResults()?;
            let items = result.Items()?;
            let size = items.Size()?;
            let limit = size.min(10);
            for i in 0..limit {
                let item = items.GetAt(i)?;
                let content = item.Content()?;
                let fmt = StandardDataFormats::Text()?;
                if content.Contains(&fmt)? {
                    let text = content.GetTextAsync()?.GetResults()?;
                    let text = text.to_string();
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                        if value.is_object() || value.is_array() {
                            let indent = " ".repeat(app.indent_spaces);
                            let mut buf = Vec::new();
                            let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                            let _ = value.serialize(&mut ser);
                            if app.input.trim().is_empty() { app.input = String::from_utf8(buf).unwrap_or(text); }
                            app.error = None;
                            app.last_json = Some(value);
                            break;
                        }
                    }
                }
            }
            Ok(())
        };

        let _ = try_history(); // 忽略任何错误，仅作尝试
    }
}
