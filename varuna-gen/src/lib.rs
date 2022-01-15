#![feature(path_file_prefix)]

use anyhow::{Context, Result};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub mod config;
pub mod context;
pub mod render;

pub fn gen_target(
    package: impl AsRef<Path>,
    args: BTreeMap<String, String>,
    target_id: Option<usize>,
) -> Result<()> {
    let package = package.as_ref();
    let config = config::Config::from_file(&package.join("package.toml")).with_context(|| {
        format!(
            "failed to load config from file: {}",
            package.join("package.toml").display()
        )
    })?;
    let mut context = set_context(&config, args)?;

    let source = package.join(&config.package.source);
    let target = set_target_dir(
        &package.join(&config.package.target),
        &config.package.target_prefix,
        target_id,
    )?;
    let template = package.join(&config.package.template);

    let choice = context::set_pre_render_context(&template, &mut context)?;
    let choice = toml::to_string_pretty(&choice)
        .with_context(|| format!("failed to convert choice to string: {}", template.display()))?;
    std::fs::write(&target.join("choice.toml"), choice)
        .with_context(|| format!("failed to write file: {}", target.display()))?;
    render::render(&source, &target, &context)?;
    Ok(())
}

fn set_target_dir(target: &Path, target_prefix: &str, target_id: Option<usize>) -> Result<PathBuf> {
    if !target.exists() {
        std::fs::create_dir_all(target).unwrap();
    }
    let target_counter = target.join("counter.txt");
    let mut counter = target_id.unwrap_or(1);
    if target_id.is_none() && target_counter.exists() {
        counter = std::fs::read_to_string(&target_counter)
            .with_context(|| format!("failed to read counter file: {}", target_counter.display()))?
            .parse()
            .with_context(|| {
                format!("failed to parse counter file: {}", target_counter.display())
            })?;
    }
    std::fs::write(&target_counter, (counter + 1).to_string())
        .with_context(|| format!("failed to write counter file: {}", target_counter.display()))?;
    let result = target.join(format!("{}-{}", target_prefix, counter));
    std::fs::create_dir_all(&result)
        .with_context(|| format!("failed to create target directory: {}", result.display()))?;
    Ok(result)
}

fn set_context(
    config: &config::Config,
    args: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for (k, v) in &config.args {
        if let Some(default) = &v.default {
            result.insert(k.clone(), args.get(k).unwrap_or(default).clone());
        } else {
            let value = args
                .get(k)
                .ok_or_else(|| anyhow::anyhow!("missing argument: {}", k))?;
            result.insert(k.clone(), value.clone());
        }
    }
    Ok(result)
}
