# Scwape
A command line tool in Rust to scrape data from websites via CSS selectors. 

# Install
```bash
cargo install scwape
```

# Usage
```bash
# syntax: scwape <url> -s "#css-selector"
# get all elements with an href from wikipedia's home page
scwape "https://www.wikipedia.org/" -s '[href]'
# get mdn's list of css selectors
scwape "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors" -s '[href*="/en-US/docs/Web/CSS/"]'
```
```bash
# syntax: scwape <file> -s "#css-selector" -f "format specifier\n"
# get the headers in mdn's list of css selectors. When specifying a format, appending a newline is typically desirable. 
curl "https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Selectors" > selector.html
scwape selector.html -s "h2" -f "\id: \text\n"
```

The default format is `\text\n`, and extra format specifiers are ignored. 

```bash
scwape <url_or_file> -s "#selector1" -s ".selector2" -f -f "format1\n" -f "format2\n" -f "format3\n"
```
is equivalent to
```bash
scwape <url_or_file> -s "#selector1" -s ".selector2" -f "format1\n" -f "format2\n"
```
The blank `-f` and the extra `"format3\n` are ignored.

The possible format specifiers are 
* Id (`\id`, the element id)
* Name (`\name`, the element name)
* Classes (`\classes`, the classes for the element)
* Text (`\text`, the combined text of child nodes for the element)
* Html (`\html`, the html of the element)
* Attrs (`\attrs`, the attributes of the element)

`\id` will be replaced, but `\\id`, `\\\id`, and so on will not. Likewise for the other format specifiers. 

The disparate `-d` option exists to allow for printing out each selector independently. The default behavior is to print the matching elements for all selectors in the order they appear. The disparate option would instead print the elements for the first selector, then the second, then the third and so on. 
    
# Completions
Fish and Bash shell completions are available on github and are generate upon cargo build. 
To generate your own, select the appropiate shell in `build.rs`, then run `cargo build`. The shell completion will be available in the completions directory. The list of available shells is at `https://docs.rs/clap/2.33.3/clap/enum.Shell.html`. 
