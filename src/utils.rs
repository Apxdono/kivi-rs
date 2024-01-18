use base64::{engine::general_purpose, Engine as _};

const PATH_DELIMITER: &str = "/";

/// Safely decodes Base 64 encoded string
pub fn decodeb_64_safe(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    let bytes: Vec<u8> = general_purpose::STANDARD.decode(value).unwrap_or(vec![]);
    return String::from_utf8(bytes).unwrap_or_default();
}

/// Simply return a copy of this string
pub fn identity_str(value: &str) -> String {
    return value.to_string();
}

/**
Create a string linter function that can optionally trim,
remove prefix or suffix from a string.

* `prefix` - optional prefix to remove
* `suffix` - optional suffix to remove
* `force_trim` - optionally call the [`str::trim()`] of input string

Examples

```
use kivi_rs::utils::create_str_linter;
let linter = create_str_linter(Some("https://".to_owned()), Some(":443".to_owned()), true);

assert_eq!("example.com", linter("  https://example.com:443  ".to_owned()));
assert_eq!("another-example.com", linter("another-example.com".to_owned()));
```
*/
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
/// A linter for system path that trims input and
/// removes leading `'/`'.
///
/// See [`create_str_linter`]
pub fn create_path_linter() -> impl Fn(String) -> String {
    return create_str_linter(Some(PATH_DELIMITER.to_owned()), None, true);
}

/**
Build a full URL using url parts. All url parts are [`str::trim()`]'ed before url is built.

* `url` - &lt;scheme&gt;://&lt;host&gt;\[:&lt;port&gt;\]\[/\]. Trailing `'/'` will be ignored
* `base_path` - base path appended to url **as is**
* `suffix` - *relative* path chunks. Leading `'/'` will be ignored.

Examples:

```
use kivi_rs::utils::build_url;
let url = build_url("https://example.com/", "/base_path/", "/new-post");

assert_eq!("https://example.com/base_path/new-post", url);
```
OR
```
use kivi_rs::utils::build_url;
let url = build_url("https://example.com", "/base_path/", "new-post");

assert_eq!("https://example.com/base_path/new-post", url);
```
**PAY ATTENTION**
```
use kivi_rs::utils::build_url;
let url = build_url("https://example.com", "/base_path", "/new-post");
/// Probably not what was intended.
assert_eq!("https://example.com/base_pathnew-post", url);
```
*/
pub fn build_url(url: &str, base_path: &str, suffix: &str) -> String {
    let base_url_linter = create_str_linter(None, Some(PATH_DELIMITER.to_owned()), true);
    let str_linter = create_path_linter();

    let linted_base_url = base_url_linter(url.to_owned());
    let linted_suffix = str_linter(suffix.to_owned());
    let consul_url_parts = vec![&linted_base_url, base_path, &linted_suffix];

    return consul_url_parts.join("");
}
