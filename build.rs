use std::env;
use std::fs;

macro_rules! warning_exit {
    ($txt: tt) => {{
        println!("cargo:warning={}", $txt);
        return None;
    }};
}

fn main() {
    gen_readme();
}

/// Use Doc Comment from lib.rs to gen the readme
fn gen_readme() -> Option<()> {
    let path = env::current_dir().ok()?;

    // Make sure Path has 'lib/lib.rs' and 'README.md'
    match path.join("lib/lib.rs").exists() {
        true => {}
        false => warning_exit!("No lib.rs found"),
    }

    match path.join("README.md").exists() {
        true => {}
        false => warning_exit!("No README.md found"),
    }

    let lib_raw = fs::read_to_string(path.join("lib/lib.rs")).ok()?;

    let mut new_lib = String::new();
    let mut in_comment = false;
    let mut in_code = false;
    for line in lib_raw.lines() {
        if line.ends_with("*/") && in_comment {
            in_comment = false;
        }

        if line.starts_with("```") {
            in_code = !in_code;
        }

        if in_code && line.starts_with('#') {
            continue;
        }

        if line.starts_with("///") || in_comment {
            new_lib.push_str(line);
            new_lib.push('\n')
        }

        if line.starts_with("/*!") && !in_comment {
            in_comment = true;
        }
    }

    new_lib = new_lib.replace("no_run", "rust");

    fs::write(path.join("README.md"), new_lib).ok()?;

    Some(())
}
