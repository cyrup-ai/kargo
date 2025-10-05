//! Tests for dependency updater

use anyhow::Result;

use kargo_upgrade::models::{Dependency, DependencyLocation, DependencyUpdater, DependencyUpdate};
use kargo_upgrade::types::{UpdateOptions, PendingDependencyUpdate};
use kargo_upgrade::updater::CratesIoUpdater;

// Mock updater for unit testing
#[derive(Clone)]
struct MockCratesIoUpdater;

impl DependencyUpdater for MockCratesIoUpdater {
    fn update(&self, dependency: &Dependency) -> PendingDependencyUpdate {
        let dependency = dependency.clone();

        let update_future = async move {
            Ok(Some(DependencyUpdate {
                name: dependency.name.clone(),
                from_version: dependency.version.clone(),
                to_version: "2.0.0".to_string(),
                dependency: dependency.clone(),
            }))
        };

        PendingDependencyUpdate::new(update_future)
    }
}

#[tokio::test]
async fn test_dependency_up2date() -> Result<()> {
    // Create test dependencies
    let dependencies = vec![
        Dependency {
            name: "anyhow".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::CargoTomlDirect,
        },
        Dependency {
            name: "tokio".to_string(),
            version: "1.0.0".to_string(),
            location: DependencyLocation::CargoTomlDirect,
        },
        Dependency {
            name: "tempfile".to_string(),
            version: "3.0.0".to_string(),
            location: DependencyLocation::CargoTomlDev,
        },
    ];

    // Create mock updater and test batch operation
    let updater = MockCratesIoUpdater;
    let updates = updater.update_all(&dependencies).collect().await?;

    // Verify the mock results
    assert_eq!(updates.len(), 3);

    let anyhw_update = updates
        .iter()
        .find(|u| u.name == "anyhow")
        .expect("TEST FAILURE: 'anyhow' update not found in results");
    assert_eq!(anyhw_update.from_version, "1.0.0");
    assert_eq!(anyhw_update.to_version, "2.0.0");

    let tokio_update = updates
        .iter()
        .find(|u| u.name == "tokio")
        .expect("TEST FAILURE: 'tokio' update not found in results");
    assert_eq!(tokio_update.from_version, "1.0.0");
    assert_eq!(tokio_update.to_version, "2.0.0");

    let tempfile_update = updates
        .iter()
        .find(|u| u.name == "tempfile")
        .expect("TEST FAILURE: 'tempfile' update not found in results");
    assert_eq!(tempfile_update.from_version, "3.0.0");
    assert_eq!(tempfile_update.to_version, "2.0.0");

    // Test with real CratesIoUpdater
    let options = UpdateOptions {
        update_workspace: false,
        compatible_only: true,
    };
    let real_updater = CratesIoUpdater::new(options);

    let test_dep = Dependency {
        name: "serde".to_string(),
        version: "1.0.0".to_string(),
        location: DependencyLocation::CargoTomlDirect,
    };

    let real_update = real_updater.update(&test_dep).await?;

    match real_update {
        Some(update) => {
            assert_eq!(update.name, "serde");
            assert_eq!(update.from_version, "1.0.0");
            assert_ne!(update.to_version, "1.0.0", "Should have found newer version");
        }
        None => {
            panic!("Expected update for serde from 1.0.0 but got None");
        }
    }

    Ok(())
}
