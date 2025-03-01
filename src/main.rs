use rsrusl;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;

// A variable definition mapping a letter to a category string.
struct VarDef {
    var: char,
    category: String,
}

// A command line parsed from the file.
struct CommandLine {
    var: char,
    command: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} file.list", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    let file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            rsrusl::log("error", &format!("Error opening file {}: {}", filename, e));
            std::process::exit(1);
        }
    };
    let reader = BufReader::new(file);

    let mut var_defs: Vec<VarDef> = Vec::new();
    let mut commands: Vec<CommandLine> = Vec::new();

    for line_result in reader.lines() {
        if let Ok(mut line) = line_result {
            line = line.trim_start().to_string();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with('$') {
                let mut chars = line.chars();
                chars.next();
                if let Some(var_char) = chars.next() {
                    if let Some(equal_idx) = line.find('=') {
                        let after_equal = line[equal_idx + 1..].trim();
                        if after_equal.starts_with("^*.") {
                            let category = &after_equal[3..].split_whitespace().next().unwrap_or("").to_string();
                            var_defs.push(VarDef { var: var_char, category: category.to_string() });
                        }
                    }
                }
            } else if line.starts_with('{') {
                if let Some(_) = line.find('}') {
                    let var_letter = line.chars().nth(1).unwrap_or_default();
                    if let Some(paren_idx) = line.find('(') {
                        let after_paren = &line[paren_idx..];
                        if let Some(first_quote) = after_paren.find('"') {
                            let start = paren_idx + first_quote + 1;
                            if let Some(second_quote) = line[start..].find('"') {
                                let command_str = &line[start..start + second_quote];
                                commands.push(CommandLine { var: var_letter, command: command_str.to_string() });
                            }
                        }
                    }
                }
            }
        }
    }

    let current_category = if cfg!(target_os = "windows") { "windows" } else { "other" };

    rsrusl::cls("yes");

    let mut command_list = Vec::new();
    for cmd in &commands {
        if var_defs.iter().any(|vd| vd.var == cmd.var && vd.category == current_category) {
            command_list.push(cmd.command.clone());
        }
    }

    if command_list.is_empty() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let full_command = command_list.join(" && ");
        let _ = Command::new("cmd")
            .args(&["/C", &full_command])
            .spawn()
            .expect("Failed to execute commands")
            .wait();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let full_command = command_list.join("; ");
        let _ = Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .spawn()
            .expect("Failed to execute commands")
            .wait();
    }
}
