pub mod common;

use anyhow::{Context, Result};
use function_name::named;

/// `garden plant` adds an empty repository
#[test]
#[named]
fn plant_empty_repo() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // garden plant in test/tmp/plant_empty_repo
    common::exec_garden(&["--chdir", &fixture.root(), "init", "--empty"])?;
    // Empty garden.yaml should be created
    fixture.path("garden.yaml");

    // Create tests/tmp/plant_empty_repo/repo{1,2}
    let cmd = ["git", "init", "--quiet", "repo1"];
    common::assert_cmd(&cmd, &fixture.root());
    let cmd = ["git", "init", "--quiet", "repo2"];
    common::assert_cmd(&cmd, &fixture.root());

    // repo1 has two remotes: "origin" and "remote-1".
    let worktree_repo1 = fixture.worktree("repo1");
    // git remote add origin repo-1-url
    let cmd = ["git", "remote", "add", "origin", "repo-1-url"];
    common::assert_cmd(&cmd, &worktree_repo1);
    // git remote add remote-1 remote-1-url
    let cmd = ["git", "remote", "add", "remote-1", "remote-1-url"];
    common::assert_cmd(&cmd, &worktree_repo1);
    // garden plant repo1
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repo1"])?;

    // Load the configuration and assert that the remotes are configured.
    let pathbuf = fixture.pathbuf("garden.yaml");
    let app_context = garden::model::ApplicationContext::from_path(&pathbuf)?;
    let cfg = app_context.get_root_config();
    assert_eq!(1, cfg.trees.len());
    assert_eq!("repo1", cfg.trees[0].get_name());
    assert_eq!(2, cfg.trees[0].remotes.len());

    let origin_var = cfg.trees[0].remotes.get("origin").context("origin")?;
    assert_eq!("repo-1-url", origin_var.get_expr());

    let remote1_var = cfg.trees[0].remotes.get("remote-1").context("remote-1")?;
    assert_eq!("remote-1-url", remote1_var.get_expr());

    // repo2 has two remotes: "remote-1" and "remote-2".
    // git remote add remote-1 remote-1-url
    let worktree_repo2 = fixture.worktree("repo2");
    let cmd = ["git", "remote", "add", "remote-1", "remote-1-url"];
    common::assert_cmd(&cmd, &worktree_repo2);

    // git remote add remote-2 remote-2-url
    let cmd = ["git", "remote", "add", "remote-2", "remote-2-url"];
    common::assert_cmd(&cmd, &worktree_repo2);

    // garden add repo2
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repo2"])?;

    // Load the configuration and assert that the remotes are configured.
    let app_context = garden::model::ApplicationContext::from_path(&pathbuf)?;
    let cfg = app_context.get_root_config();
    assert_eq!(2, cfg.trees.len()); // Now we have two trees.
    assert_eq!("repo2", cfg.trees[1].get_name());
    assert_eq!(2, cfg.trees[1].remotes.len());

    let remote1_var = cfg.trees[1].remotes.get("remote-1").context("remote-1")?;
    assert_eq!("remote-1-url", remote1_var.get_expr());

    let remote2_var = cfg.trees[1].remotes.get("remote-2").context("remote-2")?;
    assert_eq!("remote-2-url", remote2_var.get_expr());

    // Verify that "garden plant" will refresh the remote URLs
    // for existing entries.

    // Update repo1's origin url to repo-1-new-url.
    // git config remote.origin.url repo-1-new-url
    let cmd = ["git", "config", "remote.origin.url", "repo-1-new-url"];
    common::assert_cmd(&cmd, &worktree_repo1);

    // Update repo2's remote-2 url to remote-2-new-url.
    // git config remote.remote-2.url remote-2-new-url
    let cmd = ["git", "config", "remote.remote-2.url", "remote-2-new-url"];
    common::assert_cmd(&cmd, &worktree_repo2);

    // garden plant repo1 repo2
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repo1", "repo2"])?;

    // Load the configuration and assert that the remotes are configured.
    let app_context = garden::model::ApplicationContext::from_path(&pathbuf)?;
    let cfg = app_context.get_root_config();
    assert_eq!(2, cfg.trees.len());
    assert_eq!("repo1", cfg.trees[0].get_name());
    assert_eq!(2, cfg.trees[0].remotes.len());

    let origin_var = cfg.trees[0].remotes.get("origin").context("origin")?;
    assert_eq!("repo-1-new-url", origin_var.get_expr()); // New value.

    let remote1_var = cfg.trees[0].remotes.get("remote-1").context("remote-1")?;
    assert_eq!("remote-1-url", remote1_var.get_expr());

    assert_eq!("repo2", cfg.trees[1].get_name());
    assert_eq!(2, cfg.trees[1].remotes.len());

    let remote1_var = cfg.trees[1].remotes.get("remote-1").context("remote-1")?;
    assert_eq!("remote-1-url", remote1_var.get_expr());

    let remote2_var = cfg.trees[1].remotes.get("remote-2").context("remote-2")?;
    assert_eq!("remote-2-new-url", remote2_var.get_expr());

    Ok(())
}

/// `garden plant` detects bare repositories.
#[test]
#[named]
fn plant_bare_repo() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // Create an empty garden.yaml using "garden init".
    common::exec_garden(&["--chdir", &fixture.root(), "init", "--empty"])?;

    // garden plant repo.git
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "repos/example.git"])?;

    // Load the configuration and assert that the remotes are configured.
    let pathbuf = fixture.pathbuf("garden.yaml");
    let app_context = garden::model::ApplicationContext::from_path(&pathbuf)?;
    let cfg = app_context.get_root_config();
    assert_eq!(1, cfg.trees.len());
    assert_eq!("repos/example.git", cfg.trees[0].get_name());

    // The generated config must have "bare: true" configured.
    assert!(cfg.trees[0].is_bare_repository);

    Ok(())
}

/// `garden plant` detects "git worktree" repositories.
#[cfg(not(any(
    target_arch = "aarch64",
    target_arch = "powerpc64",
    target_arch = "s390x",
    target_arch = "x86"
)))]
#[test]
#[named]
fn plant_git_worktree() -> Result<()> {
    let fixture = common::BareRepoFixture::new(function_name!());
    // Create an empty garden.yaml using "garden init".
    common::exec_garden(&["--chdir", &fixture.root(), "init", "--empty"])?;

    // Create a parent worktree called "parent" branched off of "default".
    let cmd = ["git", "clone", "--quiet", "repos/example.git", "parent"];
    common::assert_cmd(&cmd, &fixture.root());

    // Create a child worktree called "child" branched off of "dev".
    let cmd = [
        "git",
        "worktree",
        "add",
        "--track",
        "-B",
        "dev",
        "../child",
        "origin/dev",
    ];
    common::assert_cmd(&cmd, &fixture.path("parent"));

    common::exec_garden(&["--chdir", &fixture.root(), "plant", "parent"])?;
    common::exec_garden(&["--chdir", &fixture.root(), "plant", "child"])?;

    let pathbuf = fixture.pathbuf("garden.yaml");
    let app_context = garden::model::ApplicationContext::from_path_and_root(
        &pathbuf,
        Some(&fixture.root_pathbuf()),
    )?;
    let cfg = app_context.get_root_config();
    assert_eq!(2, cfg.trees.len());
    assert_eq!("parent", cfg.trees[0].get_name());

    // The "child" repository is a child worktree of the "parent" tree.
    assert!(cfg.trees[1].is_worktree);
    assert_eq!(cfg.trees[1].worktree.get_expr(), "parent");
    assert_eq!(cfg.trees[1].branch.get_expr(), "dev");

    Ok(())
}

/// `garden plant` retains "${variable}" references for a oneline tree.
#[test]
#[named]
fn plant_keep_variables_oneline() -> Result<()> {
    let tree_name = "example/oneline";
    let config_path = "tests/data/plant.yaml";
    let fixture = common::BareRepoFixture::new(function_name!());
    let fixture_root = fixture.root();
    // Copy plant.yaml to "garden.yaml" in the temporary root.
    let mut fixture_config_pathbuf = fixture.root_pathbuf();
    fixture_config_pathbuf.push("garden.yaml");
    std::fs::copy(config_path, &fixture_config_pathbuf)?;

    // Check that the config uses ${storage} variables for url and remotes.
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
    }
    // Grow the tree.
    common::exec_garden(&["--chdir", &fixture_root, "grow", tree_name])?;
    assert!(fixture.pathbuf(tree_name).exists());
    // Re-plant the tree and ensure that the variables are retained.
    common::exec_garden(&["--chdir", &fixture_root, "plant", tree_name])?;
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
    }

    Ok(())
}

/// `garden plant` is able to add remotes to a oneline tree
#[test]
#[named]
fn plant_add_remote_to_oneline_tree() -> Result<()> {
    let tree_name = "example/oneline";
    let fixture = common::BareRepoFixture::new(function_name!());
    let fixture_root = fixture.root();
    let config_path = "tests/data/plant.yaml";
    // Copy plant.yaml to "garden.yaml" in the temporary root.
    let mut fixture_config_pathbuf = fixture.root_pathbuf();
    fixture_config_pathbuf.push("garden.yaml");
    std::fs::copy(config_path, &fixture_config_pathbuf)?;
    // Check that the config uses ${storage} variables for url and remotes.
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
    }
    // Grow the tree.
    common::exec_garden(&["--chdir", &fixture_root, "grow", tree_name])?;
    assert!(fixture.pathbuf(tree_name).exists());

    // Add a remote called "new-remote".
    let oneline_path = fixture.path(tree_name);
    let cmd = ["git", "remote", "add", "new-remote", "new-url"];
    common::assert_cmd(&cmd, &oneline_path);

    // Re-plant the tree and ensure that variables are retained and the new remote is recorded.
    common::exec_garden(&["--chdir", &fixture_root, "plant", tree_name])?;
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
        assert_eq!(
            tree.remotes.get("new-remote").unwrap().get_expr(),
            "new-url"
        );
    }

    Ok(())
}

/// `garden plant` retains "${variable}" references for a simple tree.
#[test]
#[named]
fn plant_keep_variables_simple() -> Result<()> {
    let tree_name = "example/main";
    let config_path = "tests/data/plant.yaml";
    let fixture = common::BareRepoFixture::new(function_name!());
    let fixture_root = fixture.root();
    // Copy plant.yaml to "garden.yaml" in the temporary root.
    let mut fixture_config_pathbuf = fixture.root_pathbuf();
    fixture_config_pathbuf.push("garden.yaml");
    std::fs::copy(config_path, &fixture_config_pathbuf)?;
    // Check that the config uses ${storage} variables for url and remotes.
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
    }
    // Grow the tree.
    common::exec_garden(&["--chdir", &fixture_root, "grow", tree_name])?;
    assert!(fixture.pathbuf(tree_name).exists());
    // Re-plant the tree and ensure that the variables are retained.
    common::exec_garden(&["--chdir", &fixture_root, "plant", tree_name])?;
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
    }

    Ok(())
}

/// `garden plant` retains "${variable}" references for a tree with remotes.
#[test]
#[named]
fn plant_keep_variables_with_remotes() -> Result<()> {
    let tree_name = "example/variables";
    let config_path = "tests/data/plant.yaml";
    let fixture = common::BareRepoFixture::new(function_name!());
    let fixture_root = fixture.root();
    // Copy plant.yaml to "garden.yaml" in the temporary root.
    let mut fixture_config_pathbuf = fixture.root_pathbuf();
    fixture_config_pathbuf.push("garden.yaml");
    std::fs::copy(config_path, &fixture_config_pathbuf)?;
    // Check that the config uses ${storage} variables for url and remotes.
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
        assert_eq!(
            tree.remotes.get("example").unwrap().get_expr(),
            "${storage}"
        );
    }
    // Grow the tree.
    common::exec_garden(&["--chdir", &fixture_root, "grow", tree_name])?;

    assert!(fixture.pathbuf(tree_name).exists());
    // Re-plant the tree and ensure that the variables are retained.
    common::exec_garden(&["--chdir", &fixture_root, "plant", tree_name])?;
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
        assert_eq!(
            tree.remotes.get("example").unwrap().get_expr(),
            "${storage}"
        );
    }

    Ok(())
}

/// `garden plant` retains "${variable}" references for a templated tree.
#[test]
#[named]
fn plant_keep_variables_with_templates() -> Result<()> {
    let tree_name = "example/template";
    let config_path = "tests/data/plant.yaml";
    let fixture = common::BareRepoFixture::new(function_name!());
    let fixture_root = fixture.root();
    let mut fixture_config_pathbuf = fixture.root_pathbuf();
    fixture_config_pathbuf.push("garden.yaml");
    std::fs::copy(config_path, &fixture_config_pathbuf)?;
    // Check that the config uses ${storage} variables for url and remotes.
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
        assert_eq!(
            tree.remotes.get("example").unwrap().get_expr(),
            "${storage}"
        );
    }
    // Grow the tree.
    common::exec_garden(&["--chdir", &fixture_root, "grow", tree_name])?;
    assert!(fixture.pathbuf(tree_name).exists());
    // Re-plant the tree and ensure that the variables are retained.
    common::exec_garden(&["--chdir", &fixture_root, "plant", tree_name])?;
    {
        let app_context = garden::model::ApplicationContext::from_path_and_root(
            &fixture_config_pathbuf,
            Some(&fixture.root_pathbuf()),
        )?;
        let cfg = app_context.get_root_config();
        let tree = cfg.get_tree(tree_name).unwrap();
        assert_eq!(tree.remotes.get("origin").unwrap().get_expr(), "${storage}");
        assert_eq!(
            tree.remotes.get("example").unwrap().get_expr(),
            "${storage}"
        );
    }

    Ok(())
}
