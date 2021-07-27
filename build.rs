use clap_generate::{generators::Fish, generate_to};
use clap::{IntoApp};

include!("src/cli.rs");

fn main() {
    let mut app = Opts::into_app();
    app.set_bin_name("scwape");

    let outdir = env!("CARGO_MANIFEST_DIR");
    generate_to::<Fish, _, _>(&mut app, "scwape", outdir);
}

