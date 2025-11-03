use eframe::egui;

/// 安装 CJK 字体
pub fn install_cjk_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontDefinitions, FontFamily};

    let mut fonts = FontDefinitions::default();

    // 只尝试加载最常用的微软雅黑，减少 I/O
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\msyh.ttc") {
        fonts.font_data.insert("cjk".to_owned(), FontData::from_owned(bytes).into());
        if let Some(list) = fonts.families.get_mut(&FontFamily::Proportional) {
            list.insert(0, "cjk".to_owned());
        }
        if let Some(list) = fonts.families.get_mut(&FontFamily::Monospace) {
            list.insert(0, "cjk".to_owned());
        }
        ctx.set_fonts(fonts);
    }
}

/// 尝试从剪贴板填充内容
pub fn try_fill_from_clipboard(input: &mut String, indent_spaces: usize) -> Option<serde_json::Value> {
    use arboard::Clipboard;
    use serde::Serialize;

    if !input.trim().is_empty() {
        return None;
    }

    // 快速读取当前剪贴板文本，若为 JSON 则格式化并填充
    if let Ok(mut cb) = Clipboard::new() {
        if let Ok(text) = cb.get_text() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                if value.is_object() || value.is_array() {
                    let indent = " ".repeat(indent_spaces);
                    let mut buf = Vec::new();
                    let formatter = serde_json::ser::PrettyFormatter::with_indent(indent.as_bytes());
                    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
                    let _ = value.serialize(&mut ser);
                    *input = String::from_utf8(buf).unwrap_or(text);
                    return Some(value);
                }
            }
        }
    }
    None
}
