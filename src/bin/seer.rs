extern crate seer;
extern crate env_logger;
extern crate log_settings;
extern crate log;

const SEER_HELP: &str = r#"Attempts to find all possible execution paths in a program.

Usage:
    seer <opts>

The options are passed to rustc.
"#;


fn show_help() {
    println!("{}", SEER_HELP);
}

fn show_version() {
    println!("{}", env!("CARGO_PKG_VERSION"));
}

fn init_logger() {
    use std::io::Write;
    let format = |formatter: &mut env_logger::fmt::Formatter, record: &log::Record| {
        if record.level() == log::Level::Trace {
            // prepend spaces to indent the final string
            let indentation = log_settings::settings().indentation;
            writeln!(
                formatter,
                "{lvl}:{module}:{indent:<indentation$} {text}",
                lvl = record.level(),
                module = record.module_path().unwrap_or("<unknown module>"),
                indentation = indentation,
                indent = "",
                text = record.args(),
            )
        } else {
            writeln!(
                formatter,
                "{lvl}:{module}: {text}",
                lvl = record.level(),
                module = record.module_path().unwrap_or("<unknown module>"),
                text = record.args(),
            )
        }
    };

    let mut builder = env_logger::Builder::new();
    builder.format(format).filter(None, log::LevelFilter::Info);

    if std::env::var("MIRI_LOG").is_ok() {
        builder.parse(&std::env::var("MIRI_LOG").unwrap());
    }

    builder.init();
}

fn main() {
    let mut args: Vec<String> = ::std::env::args().collect();

    if args.iter().any(|a| a == "--version" || a == "-V") {
        show_version();
        return;
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        show_help();
        return;
    }

    init_logger();
    let consumer = |complete: ::seer::ExecutionComplete | {
        println!("{:?}", complete);
        if let Err(_) = complete.result {
            println!("hit an error. halting");
            false
        } else {
            true
        }
    };

    let mut config = ::seer::ExecutionConfig::new();

    let mut emit_error_idx = None;
    for (idx, arg) in args.iter().enumerate() {
        if arg == "--emit-error" {
            emit_error_idx = Some(idx);
            break;
        }
    }
    if let Some(idx) = emit_error_idx {
        config.emit_error(true);
        args.remove(idx);
    } else {
        config.consumer(consumer);
    }

    config.run(args);
}
