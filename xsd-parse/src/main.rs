use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use xsd_parser::{
    Config, IdentType,
    config::{GeneratorFlags, InterpreterFlags, OptimizerFlags, ParserFlags, Resolver, Schema},
    generate,
};

fn main() -> Result<()> {
    let (schemas, roots): (Vec<String>, Vec<String>) = std::env::args()
        .skip(1)
        .partition(|a| a.to_lowercase().ends_with(".xsd"));

    if schemas.is_empty() {
        bail!("No schema was provided");
    }

    let schemas: Vec<PathBuf> = schemas
        .iter()
        .map(|s| {
            PathBuf::from(s)
                .canonicalize()
                .with_context(|| format!("File not found: {s}"))
        })
        .collect::<Result<_>>()?;

    let mut config = Config::default();

    config.parser.resolver = vec![Resolver::File];
    config.parser.flags = ParserFlags::all();
    config.parser.schemas = schemas.into_iter().map(Schema::File).collect();

    config.interpreter.flags = InterpreterFlags::all();
    config.optimizer.flags = OptimizerFlags::all();
    config.generator.flags = GeneratorFlags::all() - GeneratorFlags::NILLABLE_TYPE_SUPPORT;

    let mut config = config.with_serde_quick_xml().with_derive(["Deserialize"]);

    if !roots.is_empty() {
        config = config.with_generate(roots.iter().map(|r| (IdentType::Element, r.as_str())));
    }

    let tokens = generate(config).context("Couldn't generate the code")?;
    let file = syn::parse2(tokens).context("Couldn't parse the generated code")?;

    print!("{}", prettyplease::unparse(&file));

    Ok(())
}
