//! Path Mapper - JSON路径构造器
//!
//! 支持嵌套路径插入，如 `input.parameter_name` 用于Replicate
//! 支持 `a.b.c` 这样的点分路径

use serde_json::{json, Value};
use std::collections::HashMap;

/// Path Mapper错误
#[derive(Debug, thiserror::Error)]
pub enum PathMapperError {
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Cannot set value at path: {0}")]
    CannotSetValue(String),
}

/// Path Mapper - 支持嵌套路径的JSON构造器
pub struct PathMapper;

impl PathMapper {
    /// 在JSON对象中设置嵌套路径的值
    ///
    /// # 参数
    /// - `obj`: 目标JSON对象（可变引用）
    /// - `path`: 点分路径，如 `"input.temperature"` 或 `"generationConfig.maxOutputTokens"`
    /// - `value`: 要设置的值
    ///
    /// # 示例
    /// ```
    /// use ai_lib::utils::path_mapper::PathMapper;
    /// use serde_json::json;
    ///
    /// let mut obj = json!({});
    /// PathMapper::set_path(&mut obj, "input.temperature", json!(0.7)).unwrap();
    /// PathMapper::set_path(&mut obj, "input.max_tokens", json!(1000)).unwrap();
    ///
    /// assert_eq!(obj["input"]["temperature"], 0.7);
    /// assert_eq!(obj["input"]["max_tokens"], 1000);
    /// ```
    pub fn set_path(obj: &mut Value, path: &str, value: Value) -> Result<(), PathMapperError> {
        if path.is_empty() {
            return Err(PathMapperError::InvalidPath("Empty path".to_string()));
        }

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(PathMapperError::InvalidPath("Empty path parts".to_string()));
        }

        // 确保根对象是Object
        if !obj.is_object() {
            *obj = json!({});
        }

        let mut current = obj
            .as_object_mut()
            .ok_or_else(|| PathMapperError::CannotSetValue("Root is not an object".to_string()))?;

        // 处理除最后一个部分外的所有路径段
        for (idx, part) in parts.iter().enumerate().take(parts.len() - 1) {
            if part.is_empty() {
                return Err(PathMapperError::InvalidPath(format!(
                    "Empty path part at index {}",
                    idx
                )));
            }

            // 如果路径不存在或不是对象，创建新对象
            if !current.contains_key(*part) || !current[*part].is_object() {
                current.insert(part.to_string(), json!({}));
            }

            // 移动到下一层
            current = current[*part].as_object_mut().ok_or_else(|| {
                PathMapperError::CannotSetValue(format!("Cannot access object at path: {}", part))
            })?;
        }

        // 设置最后一个路径段的值
        let last_part = parts
            .last()
            .ok_or_else(|| PathMapperError::InvalidPath("No last part".to_string()))?;

        if last_part.is_empty() {
            return Err(PathMapperError::InvalidPath(
                "Last path part is empty".to_string(),
            ));
        }

        current.insert(last_part.to_string(), value);
        Ok(())
    }

    /// 从JSON对象中获取嵌套路径的值
    ///
    /// # 参数
    /// - `obj`: JSON对象
    /// - `path`: 点分路径，支持数组索引如 `choices[0].delta.content`
    ///
    /// # 返回
    /// 如果路径存在，返回Some(Value)，否则返回None
    pub fn get_path<'a>(obj: &'a Value, path: &str) -> Option<&'a Value> {
        if path.is_empty() {
            return None;
        }

        let parts: Vec<&str> = path.split('.').collect();
        let mut current = obj;

        for part in parts {
            if part.is_empty() {
                return None;
            }

            // Check if part contains array index, e.g., "choices[0]"
            if let Some(bracket_pos) = part.find('[') {
                // Extract key and index
                let key = &part[..bracket_pos];
                let idx_str = part[bracket_pos + 1..].trim_end_matches(']');

                // First access the object key
                if !key.is_empty() {
                    match current {
                        Value::Object(map) => {
                            current = map.get(key)?;
                        }
                        _ => return None,
                    }
                }

                // Then access the array index
                if let Ok(idx) = idx_str.parse::<usize>() {
                    match current {
                        Value::Array(arr) => {
                            current = arr.get(idx)?;
                        }
                        _ => return None,
                    }
                } else if idx_str == "*" {
                    // Wildcard: get first element for now
                    match current {
                        Value::Array(arr) => {
                            current = arr.first()?;
                        }
                        _ => return None,
                    }
                } else {
                    return None;
                }
            } else {
                // Simple key access
                match current {
                    Value::Object(map) => {
                        current = map.get(part)?;
                    }
                    Value::Array(arr) => {
                        // Pure index like "[0]" as a separate part
                        if let Some(idx_str) =
                            part.strip_suffix(']').and_then(|s| s.strip_prefix('['))
                        {
                            if let Ok(idx) = idx_str.parse::<usize>() {
                                current = arr.get(idx)?;
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    }
                    _ => return None,
                }
            }
        }

        Some(current)
    }

    /// 批量设置路径值
    pub fn set_paths(
        obj: &mut Value,
        paths: &HashMap<String, Value>,
    ) -> Result<(), PathMapperError> {
        for (path, value) in paths {
            Self::set_path(obj, path, value.clone())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_path() {
        let mut obj = json!({});
        PathMapper::set_path(&mut obj, "temperature", json!(0.7)).unwrap();
        assert_eq!(obj["temperature"], 0.7);
    }

    #[test]
    fn test_nested_path() {
        let mut obj = json!({});
        PathMapper::set_path(&mut obj, "input.temperature", json!(0.7)).unwrap();
        PathMapper::set_path(&mut obj, "input.max_tokens", json!(1000)).unwrap();

        assert_eq!(obj["input"]["temperature"], 0.7);
        assert_eq!(obj["input"]["max_tokens"], 1000);
    }

    #[test]
    fn test_deeply_nested_path() {
        let mut obj = json!({});
        PathMapper::set_path(&mut obj, "generationConfig.maxOutputTokens", json!(2000)).unwrap();

        assert_eq!(obj["generationConfig"]["maxOutputTokens"], 2000);
    }

    #[test]
    fn test_replicate_input_path() {
        let mut obj = json!({});
        PathMapper::set_path(&mut obj, "input.messages", json!([])).unwrap();
        PathMapper::set_path(&mut obj, "input.temperature", json!(0.8)).unwrap();

        assert!(obj["input"]["messages"].is_array());
        assert_eq!(obj["input"]["temperature"], 0.8);
    }

    #[test]
    fn test_get_path() {
        let obj = json!({
            "input": {
                "temperature": 0.7,
                "max_tokens": 1000
            }
        });

        assert_eq!(
            PathMapper::get_path(&obj, "input.temperature"),
            Some(&json!(0.7))
        );
        assert_eq!(
            PathMapper::get_path(&obj, "input.max_tokens"),
            Some(&json!(1000))
        );
        assert_eq!(PathMapper::get_path(&obj, "input.nonexistent"), None);
    }

    #[test]
    fn test_set_paths_batch() {
        let mut obj = json!({});
        let mut paths = HashMap::new();
        paths.insert("input.temperature".to_string(), json!(0.7));
        paths.insert("input.max_tokens".to_string(), json!(1000));
        paths.insert("generationConfig.topP".to_string(), json!(0.9));

        PathMapper::set_paths(&mut obj, &paths).unwrap();

        assert_eq!(obj["input"]["temperature"], 0.7);
        assert_eq!(obj["input"]["max_tokens"], 1000);
        assert_eq!(obj["generationConfig"]["topP"], 0.9);
    }
}
