use clap::Clap;
use colored::Colorize;
use scraper::{Html, Selector};
use snailquote::unescape;
use std::{fmt, fs};

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
    if let Err(err) = run().await {
        println!("{}", err.red());
    }
}

/// Run the logic, returning error if any fails
async fn run() -> Result<(), String> {
    let opts: Opts = Opts::parse();

    if opts.selector.is_empty() {
        return Err("Specify at least one selector via the -s argument. See --help for more information.".to_string());
    }

    let url_or_file = opts.url_or_file.clone();
    let body = if url_or_file.starts_with("http") {
        let resp = if let Ok(r) = reqwest::get(opts.url_or_file.clone()).await {
            r
        } else {
            return Err("Failed to get url: ".to_string() + &opts.url_or_file + " Url may be invalid. ");
        };
        if !resp.status().is_success() {
            return Err("Failed to fetch get html from url: ".to_string() + &opts.url_or_file);
        }

        resp.text().await.unwrap()
    } else if let Ok(content) = fs::read_to_string(url_or_file) {
        content
    } else {
        return Err("Failed to read file at ".to_string() + &opts.url_or_file);
    };
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
            let selector = if let Ok(s) = Selector::parse(&select_opt) {
                s
            } else {
                return Err("Failed to parse selector: ".to_string() + &select_opt);
            };
            let selection = fragment.select(&selector).collect::<Vec<_>>();
            for e in selection {
                print_element(e, &format_vec[i]);
            }
        }
        Ok(())
    } else {
        let containing_selector = opts.selector.join(",");
        let selector = if let Ok(s) = Selector::parse(&containing_selector) {
            s
        } else {
            return Err("Failed to parse CSS selector: ".to_string() + &containing_selector);
        };
        let selection = fragment.select(&selector).collect::<Vec<_>>();
        for element in selection.iter() {
            // find the corresponding selector (each one should not return an error, since the containing selector did not)
            let index = opts
                .selector
                .iter()
                .position(|s| Selector::parse(&s).unwrap().matches(&element))
                .unwrap();
            print_element(*element, &format_vec[index]);
        }
        if selection.is_empty() {
            return Err("No elements found for selector: \"".to_string() + &containing_selector + "\" in file or url: \"" + &opts.url_or_file + "\"");
        }
        Ok(())
    }
}

/// split by control sequences, respecting escaped backslashes. keeps info on which sequence was used to delimit
fn split_keep(splits: &[Splits], arg: Control) -> Vec<Splits> {
    let mut result = Vec::new();
    for split in splits.iter() {
        let mut last = 0;
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
                result.push(Splits::Text(text[last..text.len()].to_string()));
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
                    print!(
                        "{}",
                        element.value().classes().collect::<Vec<&str>>().join(",")
                    );
                }
                Control::Text => {
                    print!("{}", element.text().collect::<Vec<&str>>().join(""));
                }
                Control::Html => {
                    print!("{}", element.html());
                }
                Control::Attrs => {
                    print!(
                        "{}",
                        element
                            .value()
                            .attrs()
                            .map(|(key, value)| format!("{}: {}", key, value))
                            .collect::<Vec<String>>()
                            .join(",")
                    );
                }
            }
        }
    }
}
