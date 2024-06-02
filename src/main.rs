use carboncopycat::cat_files;
use carboncopycat::CatFilesError;
use carboncopycat::NumberingMode;
use carboncopycat::Options;
use owo_colors::OwoColorize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage(program: &str) {
    let program_color = program.bright_green();
    let option_args = "[OPTION]...".bright_blue();
    let file_args = "[FILE]...".bright_yellow();
    println!();
    print!("{}", "Usage:".bold().underline());
    println!(" {program_color} {option_args} {file_args}\n",);
    println!(
        "\
With no FILE, or when FILE is -, read standard input.

    -A, --show-all           equivalent to -vET
    -b, --number-nonblank    number nonempty output lines, overrides -n
    -e                       equivalent to -vE
    -E, --show-ends          display $ at end of each line
    -n, --number             number all output lines
    -s, --squeeze-blank      suppress repeated empty output lines
    -t                       equivalent to -vT
    -T, --show-tabs          display TAB characters as ^I
    -u                       (ignored)
    -v, --show-nonprinting   use ^ and M- notation, except for LFD and TAB
        --help               display this help and exit
        --version            output version information and exit
"
    );
    print!("{}", "Examples:".bold().underline());
    println!(
        "
    {} f - g  Output f's contents, then standard input, then g's contents.
    {}        Copy standard input to standard output.
",
        program.bright_green(),
        program.bright_green(),
    );

    println!(
        "Source code: {}",
        "<https://github.com/pilleye/carboncopycat>".bright_blue()
    );
}

fn invalid_option(program: &str, option: &str) {
    eprint!("{}: ", program.bright_green());
    eprint!("{}", "invalid option -- '".bright_red());
    eprint!("{}", option.bright_blue());
    eprintln!("{}", "'".bright_red());
    eprintln!(
        "Try '{}' for more information.",
        format!("{} --help", program).bright_green()
    );
}

fn parse_args(args: &[String]) -> (Vec<String>, Options) {
    let mut file_paths = Vec::new();
    let mut options = Options::new();
    for arg in args.iter().skip(1) {
        if arg.starts_with("--") {
            let option = arg.split_at(2).1;
            match option {
                "show-all" => {
                    options = options
                        .show_nonprinting(true)
                        .show_tabs(true)
                        .show_ends(true);
                }
                "number-nonblank" => {
                    options = options.number(NumberingMode::NonEmpty);
                }
                "show-ends" => {
                    options = options.show_ends(true);
                }
                "number" => {
                    if options.number == NumberingMode::None {
                        options = options.number(NumberingMode::All);
                    }
                }
                "squeeze-blank" => {
                    options = options.squeeze_blank(true);
                }
                "show-tabs" => {
                    options = options.show_tabs(true);
                }
                "show-nonprinting" => {
                    options = options.show_nonprinting(true);
                }
                "help" => {
                    usage(&args[0]);
                    std::process::exit(0);
                }
                "version" => {
                    println!("{} v{}", &args[0].bright_green(), VERSION);
                    std::process::exit(0);
                }
                _ => {
                    invalid_option(&args[0], arg);
                    std::process::exit(1);
                }
            }
        } else if arg.starts_with("-") {
            // FIXME: Accept "-" as a file path for stdin
            for c in arg.chars().skip(1) {
                match c {
                    'A' => {
                        options = options
                            .show_nonprinting(true)
                            .show_tabs(true)
                            .show_ends(true);
                    }
                    'b' => {
                        options = options.number(NumberingMode::NonEmpty);
                    }
                    'e' => {
                        options = options.show_nonprinting(true).show_ends(true);
                    }
                    'E' => {
                        options = options.show_ends(true);
                    }
                    'n' => {
                        if options.number == NumberingMode::None {
                            options = options.number(NumberingMode::All);
                        }
                    }
                    's' => {
                        options = options.squeeze_blank(true);
                    }
                    't' => {
                        options = options.show_nonprinting(true).show_tabs(true);
                    }
                    'T' => {
                        options = options.show_tabs(true);
                    }
                    'u' => {
                        // Ignored
                    }
                    'v' => {
                        options = options.show_nonprinting(true);
                    }
                    _ => {
                        invalid_option(&args[0], arg);
                        std::process::exit(1);
                    }
                }
            }
        } else {
            file_paths.push(arg.clone());
        }
    }
    (file_paths, options)
}

pub fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let (files, options) = parse_args(&args);
    if let Err(e) = cat_files(&files, &options) {
        match e {
            CatFilesError::NotFound(file) => {
                eprintln!(
                    "{}: {}: {}",
                    &args[0].bright_green(),
                    file.bright_yellow(),
                    "No such file or directory".bright_blue(),
                );
                std::process::exit(1);
            }
            CatFilesError::Io(e) => {
                eprintln!("{}: {}", &args[0].bright_green(), e);
                std::process::exit(1);
            }
        }
    }
}
