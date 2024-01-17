use base64::{engine::general_purpose, Engine as _};

use crate::kvsource::KVError;

const PATH_DELIMITER: &str = "/";

/// Safely decodes Base 64 encoded string
pub fn decodeb64_safe(value: &str) -> String {
    let bytes: Vec<u8> = general_purpose::STANDARD.decode(value).unwrap_or(vec![]);
    return String::from_utf8(bytes).unwrap_or_default();
}

/// Simply return a copy of this string
pub fn identity_str(value: &str) -> String {
    return value.to_string();
}

pub fn create_str_linter(
    prefix: Option<String>,
    suffix: Option<String>,
    force_trim: bool,
) -> impl Fn(String) -> String {
    return move |s| -> String {
        let mut res = s;
        if force_trim {
            res = res.trim().to_owned();
        }
        if let Some(pfx) = &prefix {
            res = res.strip_prefix(pfx).unwrap_or(&res).to_owned()
        }
        if let Some(sfx) = &suffix {
            res = res.strip_suffix(sfx).unwrap_or(&res).to_owned()
        }
        return res;
    };
}

pub fn create_path_linter() -> impl Fn(String) -> String {
    return create_str_linter(Some(PATH_DELIMITER.to_owned()), None, true);
}

pub fn build_url(url: &str, base_path: &str, suffix: &str) -> String {
    let base_url_linter = create_str_linter(None, Some(PATH_DELIMITER.to_owned()), true);
    let str_linter = create_path_linter();

    let linted_base_url = base_url_linter(url.to_owned());
    let linted_suffix = str_linter(suffix.to_owned());
    let consul_url_parts = vec![&linted_base_url, base_path, &linted_suffix];

    return consul_url_parts.join("");
}

pub fn edit_old_value(content: String) -> Result<String, KVError> {
    let modified = edit::edit(content);

    return match modified {
        Ok(body) => Ok(body),
        Err(err) => Err(KVError::ValueWriteErr(format!("{err}"))),
    };
}
