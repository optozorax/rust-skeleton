use std::{fs, io::Write, path::PathBuf, process::ExitCode};

use clap::Parser;
use ra_ap_syntax::{AstNode, Edition, SourceFile, ast};

#[derive(Parser)]
struct Args {
    /// Rust source file to reduce
    file: PathBuf,
}

fn main() -> ExitCode {
    let path = Args::parse().file;

    let Ok(mut source) = fs::read_to_string(&path) else {
        eprintln!("rust-skeleton: could not read {}", path.display());
        return ExitCode::FAILURE;
    };
    let before = (source.lines().count(), source.chars().count());
    let parsed = SourceFile::parse(&source, Edition::CURRENT);
    if let Some(error) = parsed.errors().first() {
        eprintln!("rust-skeleton: {}: {error}", path.display());
        return ExitCode::FAILURE;
    }

    let mut replacements = Vec::new();
    for function in parsed
        .tree()
        .syntax()
        .descendants()
        .filter_map(ast::Fn::cast)
    {
        if function
            .syntax()
            .ancestors()
            .skip(1)
            .any(|node| ast::Fn::cast(node).is_some())
        {
            continue;
        }
        let Some(statements) = function.body().and_then(|body| body.stmt_list()) else {
            continue;
        };
        let (Some(open), Some(close)) = (statements.l_curly_token(), statements.r_curly_token())
        else {
            continue;
        };
        let start = usize::from(open.text_range().end());
        let end = usize::from(close.text_range().start());
        if source[start..end].trim().is_empty() {
            continue;
        }

        let block = statements.syntax().text_range();
        let first_line = source[..usize::from(block.start())]
            .bytes()
            .filter(|&byte| byte == b'\n')
            .count()
            + 1;
        let last_line = source[..usize::from(block.end())]
            .bytes()
            .filter(|&byte| byte == b'\n')
            .count()
            + 1;
        let line = &source[..usize::from(open.text_range().start())]
            .rsplit('\n')
            .next()
            .unwrap();
        let indent: String = line
            .chars()
            .take_while(|character| character.is_whitespace())
            .collect();
        replacements.push((
            start..end,
            format!(
                "\n{indent}    /* implementation omitted: source lines {first_line}-{last_line} */\n{indent}"
            ),
        ));
    }

    for (range, replacement) in replacements.into_iter().rev() {
        source.replace_range(range, &replacement);
    }
    let after = (source.lines().count(), source.chars().count());
    eprintln!(
        "{} lines, {} symbols -> {} lines, {} symbols",
        before.0, before.1, after.0, after.1
    );
    std::io::stdout().write_all(source.as_bytes()).unwrap();
    ExitCode::SUCCESS
}
