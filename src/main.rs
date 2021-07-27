use clap::{Clap};
use colored::Colorize;
use scraper::{Html, Selector};
use snailquote::unescape;
use std::fmt;

mod cli;
use cli::Opts;

/// Used in vector to represent format
#[derive(Debug)]
enum Splits {
    /// A string
    Text(String),
    /// Control character
    Control(Control),
}

// Control sequences as an enum
#[derive(Copy, Clone, Debug)]
enum Control {
    Id,
    Name,
    Classes,
    Text,
    Html,
    Attrs,
}

/// Enum to string in order to parse format string
impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Control::Id => write!(f, "id"),
            Control::Name => write!(f, "name"),
            Control::Classes => write!(f, "classes"),
            Control::Text => write!(f, "text"),
            Control::Html => write!(f, "html"),
            Control::Attrs => write!(f, "attrs"),
        }
    }
}

/// async due to reqwest
#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let resp = reqwest::get(opts.url.clone()).await.unwrap();
    if !resp.status().is_success() {
        println!(
            "{}",
            ("Failed to fetch get html from url: ".to_string() + &opts.url).red()
        );
    }

    let body = resp.text().await.unwrap();
    // parses string of HTML as a document
    let fragment = Html::parse_document(&body);

    let mut format_vec = Vec::new();
    format_vec.reserve(opts.selector.len());
    for i in 0..opts.selector.len() {
        if let Some(format) = (&opts.format).as_ref().map(|f| f.get(i)).flatten() {
            format_vec.push(parse_format(format.to_string()));
        } else {
            format_vec.push(parse_format("\\text\\n".to_string()));
        }
    }

    if opts.disparate {
        for (i, select_opt) in opts.selector.iter().enumerate() {
            // parses based on a CSS selector
            let selector = Selector::parse(&select_opt).unwrap();
            let selection = fragment.select(&selector).collect::<Vec<_>>();
            for e in selection {
                print_element(e, &format_vec[i]);
            }
        }
    } else {
        let containing_selector = opts.selector.join(",");
        let selector = Selector::parse(&containing_selector).unwrap();
        let selection = fragment.select(&selector).collect::<Vec<_>>();
        for element in selection.iter() {
            // find the corresponding selector
            let index = opts
                .selector
                .iter()
                .position(|s| Selector::parse(&s).unwrap().matches(&element))
                .unwrap();
            print_element(*element, &format_vec[index]);
        }
    }
}

/// split by control sequences, respecting escaped backslashes. keeps info on which sequence was used to delimit
fn split_keep(splits: &[Splits], arg: Control) -> Vec<Splits> {
    let mut result = Vec::new();
    let mut last = 0;
    for split in splits.iter() {
        if let Splits::Text(text) = split {
            for (index, matched) in
                text.match_indices(&("\\".to_string() + &arg.clone().to_string()))
            {
                if index == 0 || text.get((index - 1)..index) != Some(&"\\") {
                    if last != index {
                        result.push(Splits::Text(text[last..index].to_string()));
                    }
                    result.push(Splits::Control(arg));
                    last = index + matched.len();
                }
            }
            if last < text.len() {
                result.push(Splits::Text(text[last..].to_string()));
            }
        } else if let Splits::Control(control) = split {
            result.push(Splits::Control(*control));
        }
    }
    result
}

/// parse a format string to a vector of Splits
fn parse_format(format: String) -> Vec<Splits> {
    let mut a = split_keep(&[Splits::Text(format)], Control::Id);
    a = split_keep(&a, Control::Name);
    a = split_keep(&a, Control::Classes);
    a = split_keep(&a, Control::Text);
    a = split_keep(&a, Control::Html);
    a = split_keep(&a, Control::Attrs);
    let mut out = Vec::new();
    for split in a {
        if let Splits::Text(text) = split {
            out.push(Splits::Text(
                unescape(&("\"".to_string() + &text + "\""))
                    .unwrap_or(text)
                    .to_string(),
            ));
        } else {
            out.push(split);
        }
    }
    out
}

/// print a given element with the format specified via a slice of Splits
fn print_element(element: scraper::ElementRef<'_>, format_vec: &[Splits]) {
    for item in format_vec {
        if let Splits::Text(string) = item {
            print!("{}", string);
        } else if let Splits::Control(control) = item {
            match control {
                Control::Id => print!("{}", element.value().id().unwrap_or("")),
                Control::Name => {
                    print!("{}", element.value().name())
                }
                Control::Classes => {
                    for class in element.value().classes() {
                        print!("{},", class);
                    }
                }
                Control::Text => {
                    for text in element.text() {
                        print!("{}", text);
                    }
                }
                Control::Html => {
                    print!("{}", element.html());
                }
                Control::Attrs => {
                    for (key, value) in element.value().attrs() {
                        print!("{}: {},", key, value);
                    }
                }
            }
        }
    }
}
