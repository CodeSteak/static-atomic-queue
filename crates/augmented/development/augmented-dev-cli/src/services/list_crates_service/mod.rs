use std::collections::HashSet;
use std::fs::read_dir;

use crate::manifests::CargoToml;
use crate::services::cargo_toml_reader::{CargoTomlReader, CargoTomlReaderImpl};
use crates_io_api::SyncClient;

pub struct ListCratesService {
    cargo_toml_reader: Box<dyn CargoTomlReader>,
    client: SyncClient,
}

impl Default for ListCratesService {
    fn default() -> Self {
        let client = SyncClient::new("augmented-dev-cli", std::time::Duration::from_millis(1000))
            .expect("Failed to create crates.io API client");

        ListCratesService {
            cargo_toml_reader: Box::new(CargoTomlReaderImpl::default()),
            client,
        }
    }
}

impl ListCratesService {
    pub fn run(&self) {
        log::info!("Finding crates...");

        let mut crates = Vec::new();
        self.find_entries("./crates", &mut crates);

        let manifests = self.parse_manifests(crates);
        for manifest in manifests {
            self.run_get_info(manifest);
        }
    }

    fn find_entries(&self, root: &str, crates: &mut Vec<String>) {
        log::debug!("Scanning {}", root);
        let ignore_dirs: HashSet<&str> = ["spikes", "vendor", "target", "apps"]
            .iter()
            .copied()
            .collect();

        let entries =
            read_dir(root).unwrap_or_else(|_| panic!("Failed to list {} directory", root));
        let entries: Vec<_> = entries.into_iter().collect();

        let cargo_manifest = entries.iter().find(|entry| {
            let entry = entry.as_ref().expect("Failed to get DirEntry");
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();
            file_name == "Cargo.toml"
        });
        if cargo_manifest.is_some() {
            log::info!("Manifest found at {}", root);
            crates.push(root.into());
            return;
        }

        // Recursive search
        for entry in entries {
            let entry = entry.expect("Failed to get DirEntry");
            let file_name = entry.file_name();
            let file_name = file_name.to_str().unwrap();
            let is_dir = entry.file_type().unwrap().is_dir();

            if is_dir && !ignore_dirs.contains(file_name) {
                self.find_entries(&format!("{}/{}", root, file_name), crates);
            }
        }
    }

    fn parse_manifests(&self, crates: Vec<String>) -> Vec<CargoToml> {
        crates
            .into_iter()
            .map(|c| self.cargo_toml_reader.read(&c))
            .collect()
    }

    fn run_get_info(&self, manifest: CargoToml) {
        let package = manifest.package;
        log::info!(
            "CRATE - {}@{} - {}",
            package.name,
            package.version,
            package
                .description
                .unwrap_or_else(|| "No description".into())
        );

        let is_private_package = package
            .metadata
            .map(|m| m.augmented.map(|a| a.private))
            .flatten()
            .flatten()
            .unwrap_or(false);
        if is_private_package {
            return;
        }

        let published_crate = self.client.get_crate(&package.name);
        match published_crate {
            Ok(published_crate) => {
                if published_crate.crate_data.max_version != package.version {
                    log::warn!(
                        "Published version mismatch for {}: local {} <-> crates {}",
                        package.name,
                        package.version,
                        published_crate.crate_data.max_version,
                    );
                } else {
                    log::info!(
                        "{} crates.io version {} <-> {}",
                        package.name,
                        published_crate.crate_data.max_version,
                        package.version
                    );
                }
            }
            Err(crates_io_api::Error::NotFound(_)) => {
                log::warn!("Crate is not published {}", package.name);
            }
            Err(err) => {
                log::error!("Failed to fetch crate {}: {}", package.name, err);
                panic!("Failed to list crates");
            }
        }
    }
}