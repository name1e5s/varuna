use anyhow::{Context, Result};
use minijinja::Environment;
use std::{collections::BTreeMap, path::Path};
pub fn render(source: &Path, target: &Path, context: &BTreeMap<String, String>) -> Result<()> {
    if !target.exists() {
        std::fs::create_dir_all(target)
            .with_context(|| format!("failed to create target directory: {}", target.display()))?;
    }
    for entry in source.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let mut env = Environment::new();
            let name = path
                .file_name()
                .with_context(|| format!("failed to get file name: {}", path.display()))?
                .to_str()
                .with_context(|| format!("failed to convert file name to str: {}", path.display()))?
                .to_string();
            let source = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read file: {}", path.display()))?;
            env.add_template(&name, &source)?;
            let template = env
                .get_template(&name)
                .with_context(|| format!("failed to get template: {}", name))?;
            let result = template
                .render(context.clone())
                .with_context(|| format!("failed to render template: {}", name))?;
            let target_path = target.join(path.file_name().unwrap());
            std::fs::write(&target_path, result)
                .with_context(|| format!("failed to write file: {}", target_path.display()))?;
        } else if path.is_dir() {
            let target_path = target.join(path.file_name().unwrap());
            if !target_path.exists() {
                std::fs::create_dir(&target_path).with_context(|| {
                    format!(
                        "failed to create target directory: {}",
                        target_path.display()
                    )
                })?;
            }
            render(&path, &target_path, context)?;
        } else {
            anyhow::bail!("unexpected file type: {}", path.display());
        }
    }
    Ok(())
}
