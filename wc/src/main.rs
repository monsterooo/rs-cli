use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::{
    io::{self, BufRead},
    path::PathBuf,
};

#[derive(Parser, Debug)]
struct Cli {
    /// 打印行总数
    #[arg(short, long)]
    lines: bool,
    /// 打印单词数
    #[arg(short, long)]
    words: bool,
    /// 打印字符数
    #[arg(short, long)]
    chars: bool,
    #[arg(short, long)]
    file: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    // 如果这里是在终端运行则返回 true， 如果是通过管道运行则返回 false
    let is_tty = atty::is(atty::Stream::Stdin);

    // 如果没有文件参数并且是在终端运行，则退出程序
    if is_tty && cli.file.is_none() {
        println!("wc: no file provided.");
        std::process::exit(1);
    } else if let Some(file) = cli.file {
        let counts = count_from_file(&file, cli.lines, cli.words, cli.chars)?;
        print_counts(&counts, cli.lines, cli.words, cli.chars);
    } else if !is_tty && cli.file.is_none() {
        let counts = count_from_reader(io::stdin(), cli.lines, cli.words, cli.chars)?;
        print_counts(&counts, cli.lines, cli.words, cli.chars);
    }

    Ok(())
}

fn count_from_file(
    path: &Path,
    show_lines: bool,
    show_words: bool,
    show_chars: bool,
) -> io::Result<(usize, usize, usize)> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    count_from_reader(reader, show_lines, show_words, show_chars)
}

fn count_from_reader<R: Read>(
    reader: R,
    show_lines: bool,
    show_words: bool,
    show_chars: bool,
) -> io::Result<(usize, usize, usize)> {
    let mut lines = 0;
    let mut words = 0;
    let mut chars = 0;

    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;

        if show_lines {
            lines += 1;
        }

        if show_words {
            words += line.split_whitespace().count();
        }

        if show_chars {
            // 包含换行符
            chars += line.chars().count() + 1;
        }
    }

    Ok((lines, words, chars))
}

fn print_counts(
    counts: &(usize, usize, usize),
    show_lines: bool,
    show_words: bool,
    show_chars: bool,
) {
    if show_lines {
        println!("{:>8}", counts.0)
    }
    if show_words {
        println!("{:>8}", counts.1)
    }
    if show_chars {
        println!("{:>8}", counts.2)
    }
}
