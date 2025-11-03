use serde_json::Value;

/// JSON 编辑操作
pub enum JsonEdit {
    DeleteFromObject { object_pointer: String, key: String },
    DeleteFromArray { array_pointer: String, idx: usize },
    AddToObject { pointer: String },
    AddToArray { pointer: String },
    EditValue { pointer: String, new_value: String },
    EditObjectKey { object_pointer: String, old_key: String, new_key: String },
}

/// 编辑对话框状态
pub enum EditDialog {
    EditValue { pointer: String, input: String },
    EditKey { object_pointer: String, old_key: String, input: String },
}

/// 应用编辑操作到 JSON 值
pub fn apply_edits(value: &mut Value, edits: &mut Vec<JsonEdit>) -> Option<String> {
    if edits.is_empty() {
        return None;
    }

    for edit in edits.drain(..) {
        match edit {
            JsonEdit::DeleteFromArray { array_pointer, idx } => {
                if let Some(arr) = value.pointer_mut(&array_pointer).and_then(|v| v.as_array_mut()) {
                    if idx < arr.len() {
                        arr.remove(idx);
                    }
                }
            }
            JsonEdit::DeleteFromObject { object_pointer, key } => {
                if let Some(obj) = value.pointer_mut(&object_pointer).and_then(|v| v.as_object_mut()) {
                    obj.remove(&key);
                }
            }
            JsonEdit::AddToObject { pointer } => {
                if let Some(obj) = value.pointer_mut(&pointer).and_then(|v| v.as_object_mut()) {
                    let mut counter = 0;
                    let mut new_key = "new_key".to_string();
                    while obj.contains_key(&new_key) {
                        counter += 1;
                        new_key = format!("new_key_{}", counter);
                    }
                    obj.insert(new_key, Value::Null);
                }
            }
            JsonEdit::AddToArray { pointer } => {
                if let Some(arr) = value.pointer_mut(&pointer).and_then(|v| v.as_array_mut()) {
                    arr.push(Value::Null);
                }
            }
            JsonEdit::EditValue { pointer, new_value } => {
                if let Some(target) = value.pointer_mut(&pointer) {
                    match serde_json::from_str(&new_value) {
                        Ok(new_val) => *target = new_val,
                        Err(_) => *target = Value::String(new_value),
                    }
                }
            }
            JsonEdit::EditObjectKey { object_pointer, old_key, new_key } => {
                if let Some(obj) = value.pointer_mut(&object_pointer).and_then(|v| v.as_object_mut()) {
                    // 保持键的顺序：收集所有键值对，替换旧键，然后重建对象
                    let entries: Vec<(String, Value)> = obj.iter()
                        .map(|(k, v)| {
                            if k == &old_key {
                                (new_key.clone(), v.clone())
                            } else {
                                (k.clone(), v.clone())
                            }
                        })
                        .collect();
                    
                    obj.clear();
                    for (k, v) in entries {
                        obj.insert(k, v);
                    }
                }
            }
        }
    }

    // 返回格式化后的 JSON
    serde_json::to_string_pretty(value).ok()
}
