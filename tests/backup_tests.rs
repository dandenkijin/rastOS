//! Integration tests for the backup system

mod backup_helpers;

use anyhow::Result;
use backup_helpers::TestEnvironment;
use rastos::backup::BackupManager;
use std::fs;
use std::path::Path;

#[tokio::test]
async fn test_backup_creation() -> Result<()> {
    // Set up test environment
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test subvolume with some files
    let subvol_path = env.create_test_subvolume("test_subvol").await?;
    
    // Create a backup
    let backup = backup_manager
        .create_backup(&subvol_path, Some("test_backup"), Some("test description"), false, None)
        .await?;
    
    // Verify backup metadata
    assert_eq!(backup.name, "test_backup");
    assert_eq!(backup.description, Some("test description".to_string()));
    assert!(!backup.is_incremental);
    assert!(backup.size > 0);
    
    // Verify the backup exists in storage
    let backup_path = format!("backups/{}/{}.btrfs", &backup.id[..2], backup.id);
    assert!(backup_manager.storage().list("backups/").await?.contains(&backup_path));
    
    Ok(())
}

#[tokio::test]
async fn test_backup_restore() -> Result<()> {
    // Set up test environment
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test subvolume with content
    let subvol_path = env.create_test_subvolume("test_subvol").await?;
    let test_content = "test content for restoration";
    fs::write(subvol_path.join("test.txt"), test_content).await?;
    
    // Create a backup
    let backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Restore to a new location
    let restore_path = env._temp_dir.path().join("restored");
    backup_manager.restore_backup(&backup.id, Some(&restore_path)).await?;
    
    // Verify the restored content
    let restored_content = fs::read_to_string(restore_path.join("test.txt")).await?;
    assert_eq!(restored_content, test_content);
    
    Ok(())
}

#[tokio::test]
async fn test_incremental_backup() -> Result<()> {
    // Set up test environment
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create initial subvolume and backup
    let subvol_path = env.create_test_subvolume("test_subvol").await?;
    fs::write(subvol_path.join("file1.txt"), "initial content").await?;
    
    let full_backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Modify the subvolume
    let new_content = "modified content";
    fs::write(subvol_path.join("file2.txt"), new_content).await?;
    
    // Create incremental backup
    let incremental = backup_manager
        .create_backup(&subvol_path, None, None, true, Some(&full_backup))
        .await?;
    
    // Verify incremental backup properties
    assert!(incremental.is_incremental);
    assert_eq!(incremental.parent_id, Some(full_backup.id));
    
    // Verify both backups exist
    let backups = backup_manager.list_backups().await?;
    assert_eq!(backups.len(), 2);
    
    Ok(())
}

#[tokio::test]
async fn test_backup_deletion() -> Result<()> {
    // Set up test environment
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test subvolume and backup
    let subvol_path = env.create_test_subvolume("test_subvol").await?;
    let backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Verify the backup exists
    assert!(backup_manager.get_backup(&backup.id).await.is_ok());
    
    // Delete the backup
    backup_manager.delete_backup(&backup.id).await?;
    
    // Verify the backup is gone
    assert!(backup_manager.get_backup(&backup.id).await.is_err());
    
    Ok(())
}

#[tokio::test]
async fn test_list_backups() -> Result<()> {
    // Set up test environment
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create multiple backups
    let subvol_path = env.create_test_subvolume("test_subvol").await?;
    
    // First backup
    backup_manager
        .create_backup(&subvol_path, Some("backup1"), None, false, None)
        .await?;
    
    // Second backup
    let backup2 = backup_manager
        .create_backup(&subvol_path, Some("backup2"), None, false, None)
        .await?;
    
    // List backups
    let backups = backup_manager.list_backups().await?;
    
    // Verify we have both backups
    assert_eq!(backups.len(), 2);
    assert!(backups.iter().any(|b| b.name == "backup1"));
    assert!(backups.iter().any(|b| b.name == "backup2"));
    
    // Verify sorting (newest first)
    assert_eq!(backups[0].id, backup2.id);
    
    Ok(())
}

#[tokio::test]
async fn test_backup_verification() -> Result<()> {
    // Set up test environment
    let env = TestEnvironment::new().await?;
    let backup_manager = env.create_backup_manager().await?;
    
    // Create a test backup
    let subvol_path = env.create_test_subvolume("test_subvol").await?;
    let backup = backup_manager
        .create_backup(&subvol_path, None, None, false, None)
        .await?;
    
    // Verify the backup
    let is_valid = backup_manager.verify_backup(&backup.id).await?;
    assert!(is_valid);
    
    // Verify non-existent backup
    let is_valid = backup_manager.verify_backup("nonexistent").await?;
    assert!(!is_valid);
    
    Ok(())
}
