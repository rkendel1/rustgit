use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeRegistryEntry {
    pub runtime: &'static str,
    pub package_manager: Option<&'static str>,
    pub dev: Option<&'static str>,
    pub build: Option<&'static str>,
    pub start: Option<&'static str>,
}

pub fn runtime_registry() -> HashMap<&'static str, RuntimeRegistryEntry> {
    HashMap::from([
        (
            "nextjs",
            RuntimeRegistryEntry {
                runtime: "node",
                package_manager: Some("npm"),
                dev: Some("npm run dev"),
                build: Some("npm run build"),
                start: Some("npm run start"),
            },
        ),
        (
            "react",
            RuntimeRegistryEntry {
                runtime: "node",
                package_manager: Some("npm"),
                dev: Some("npm run dev"),
                build: Some("npm run build"),
                start: Some("npm run start"),
            },
        ),
        (
            "vite",
            RuntimeRegistryEntry {
                runtime: "node",
                package_manager: Some("npm"),
                dev: Some("npm run dev"),
                build: Some("npm run build"),
                start: Some("npm run preview"),
            },
        ),
        (
            "rust",
            RuntimeRegistryEntry {
                runtime: "rust",
                package_manager: Some("cargo"),
                dev: Some("cargo run"),
                build: Some("cargo build"),
                start: Some("cargo run"),
            },
        ),
        (
            "go",
            RuntimeRegistryEntry {
                runtime: "go",
                package_manager: Some("go"),
                dev: Some("go run ."),
                build: Some("go build ./..."),
                start: Some("go run ."),
            },
        ),
        (
            "python",
            RuntimeRegistryEntry {
                runtime: "python",
                package_manager: Some("pip"),
                dev: Some("python main.py"),
                build: None,
                start: Some("python main.py"),
            },
        ),
        (
            "java",
            RuntimeRegistryEntry {
                runtime: "java",
                package_manager: Some("maven"),
                dev: Some("mvn spring-boot:run"),
                build: Some("mvn package"),
                start: Some("java -jar target/*.jar"),
            },
        ),
        (
            "php",
            RuntimeRegistryEntry {
                runtime: "php",
                package_manager: Some("composer"),
                dev: Some("php -S 0.0.0.0:8000"),
                build: None,
                start: Some("php -S 0.0.0.0:8000"),
            },
        ),
        (
            "ruby",
            RuntimeRegistryEntry {
                runtime: "ruby",
                package_manager: Some("bundler"),
                dev: Some("bundle exec rails server"),
                build: None,
                start: Some("bundle exec rails server"),
            },
        ),
        (
            "deno",
            RuntimeRegistryEntry {
                runtime: "deno",
                package_manager: Some("deno"),
                dev: Some("deno task dev"),
                build: Some("deno task build"),
                start: Some("deno task start"),
            },
        ),
        (
            "node",
            RuntimeRegistryEntry {
                runtime: "node",
                package_manager: Some("npm"),
                dev: Some("npm run dev"),
                build: Some("npm run build"),
                start: Some("npm run start"),
            },
        ),
    ])
}

pub fn runtime_from_lockfile(file_name: &str) -> Option<RuntimeRegistryEntry> {
    match file_name {
        "bun.lockb" | "bun.lock" => Some(RuntimeRegistryEntry {
            runtime: "bun",
            package_manager: Some("bun"),
            dev: Some("bun run dev"),
            build: Some("bun run build"),
            start: Some("bun run start"),
        }),
        "pnpm-lock.yaml" => Some(RuntimeRegistryEntry {
            runtime: "node",
            package_manager: Some("pnpm"),
            dev: Some("pnpm run dev"),
            build: Some("pnpm run build"),
            start: Some("pnpm run start"),
        }),
        "package-lock.json" => Some(RuntimeRegistryEntry {
            runtime: "node",
            package_manager: Some("npm"),
            dev: Some("npm run dev"),
            build: Some("npm run build"),
            start: Some("npm run start"),
        }),
        "yarn.lock" => Some(RuntimeRegistryEntry {
            runtime: "node",
            package_manager: Some("yarn"),
            dev: Some("yarn dev"),
            build: Some("yarn build"),
            start: Some("yarn start"),
        }),
        "requirements.txt" | "pyproject.toml" => runtime_registry().get("python").copied(),
        "Cargo.toml" => runtime_registry().get("rust").copied(),
        "go.mod" => runtime_registry().get("go").copied(),
        "pom.xml" => runtime_registry().get("java").copied(),
        "composer.json" => runtime_registry().get("php").copied(),
        "Gemfile" => runtime_registry().get("ruby").copied(),
        "deno.json" => runtime_registry().get("deno").copied(),
        _ => None,
    }
}
