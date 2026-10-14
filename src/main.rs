use std::{fs, io::Write, path::PathBuf, process::ExitCode};

use clap::Parser;
use ra_ap_syntax::{
    AstNode, Edition, SourceFile,
    ast::{self, AstToken, IsString},
};

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

    // A long string literal outside function bodies (a GLSL source in a
    // constant, a documentation text) keeps only its first non-empty line.
    let mut strings = Vec::new();
    for string in parsed
        .tree()
        .syntax()
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter_map(ast::AnyString::cast)
    {
        let range = string.syntax().text_range();
        let (start, end) = (usize::from(range.start()), usize::from(range.end()));
        let line_count = string.text().lines().count();
        if line_count <= 3
            || replacements
                .iter()
                .any(|(body, _)| body.start <= start && end <= body.end)
        {
            continue;
        }
        let Some(quotes) = string.quote_offsets() else {
            continue;
        };
        let raw = string.is_raw();
        let first_line = string
            .text_without_quotes()
            .lines()
            .map(|line| {
                if raw {
                    line.trim()
                } else {
                    // A line continuation `\` would escape the closing quote.
                    line.trim().trim_end_matches('\\').trim_end()
                }
            })
            .find(|line| !line.is_empty())
            .unwrap_or("");
        let first_source_line = source[..start]
            .bytes()
            .filter(|&byte| byte == b'\n')
            .count()
            + 1;
        let last_source_line = first_source_line + line_count - 1;
        strings.push((
            start..end,
            format!(
                "{}{first_line}{} /* string literal omitted: source lines {first_source_line}-{last_source_line} */",
                &source[quotes.quotes.0],
                &source[quotes.quotes.1],
            ),
        ));
    }
    replacements.extend(strings);
    replacements.sort_by_key(|(range, _)| range.start);

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
