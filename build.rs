use clap_generate::{generators::Fish, generate_to};
use clap::{IntoApp};

include!("src/cli.rs");

fn main() {
    let mut app = Opts::into_app();
    app.set_bin_name("scwape");

    let outdir = env!("CARGO_MANIFEST_DIR");
    generate_to::<Fish, _, _>(&mut app, "scwape", outdir);
}



// [build-dependencies]
// reqwest = "0.11.4"
// select = "0.5.0"
// scraper = "0.12.0"
// tokio = { version = "1.9.0", features = ["full"]}
// clap = "3.0.0-beta.2"
// colored = "2.0.0"
// snailquote = "0.3.0"
// clap_generate = "3.0.0-beta.2"
