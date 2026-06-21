use super::SoftwareExecutionSpec;

pub fn portable_execution_toml(spec: &SoftwareExecutionSpec) -> String {
    let package_manager = spec.runtime.package_manager.as_deref().unwrap_or("unknown");
    let services = spec
        .services
        .iter()
        .map(|service| format!("{} = \"{}\"", service.name, service.mode))
        .collect::<Vec<_>>()
        .join("\n");
    let environment = spec
        .environment
        .iter()
        .map(|(name, source)| format!("{name} = \"{source}\""))
        .collect::<Vec<_>>()
        .join("\n");
    let capabilities = spec
        .capabilities
        .iter()
        .map(|capability| format!("\"{capability}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "version = \"{}\"\n[runtime]\nlanguage = \"{}\"\nframework = \"{}\"\npackage_manager = \"{}\"\n[services]\n{}\n[environment]\n{}\n[capabilities]\nall = [{}]\n[healing]\napply_known_repairs = true\n[confidence]\nexpected_success = {:.3}",
        spec.identity.version,
        spec.runtime.language,
        spec.runtime.framework,
        package_manager,
        services,
        environment,
        capabilities,
        spec.confidence as f32 / 100.0
    )
}
