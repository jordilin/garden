use ::model;
use ::query;


/// Resolve a tree query into a `Vec<garden::model::TreeContext>`.
///
/// Parameters:
/// - `config`: `&garden::model::Configuration`.
/// - `query`: Tree query `&str`.
/// Returns:
/// - `Vec<garden::model::TreeContext>`

pub fn resolve_trees(config: &model::Configuration, query: &str)
-> Vec<model::TreeContext> {
    let mut result = Vec::new();
    let tree_query = model::TreeQuery::new(query);
    let ref pattern = tree_query.pattern;

    if tree_query.include_gardens {
        result = garden_trees(config, pattern);
        if result.len() > 0 {
            return result;
        }
    }

    if tree_query.include_groups {
        for group in &config.groups {
            // Find the matching group
            if !pattern.matches(group.name.as_ref()) {
                continue;
            }
            // Matching group found, collect its trees
            result.append(&mut trees_from_group(config, None, group));
        }
        if result.len() > 0 {
            return result;
        }
    }

    // No matching gardens or groups were found.
    // Search for matching trees.
    if tree_query.include_trees {
        result.append(&mut trees(config, pattern));
        if result.len() > 0 {
            return result;
        }
    }

    // Lowest precedence: match paths on the filesystem.
    // The pattern is a default non-special pattern, and its value points to an
    // existing tree on the filesystem.  Look up the tree context for this
    // entry and use the matching tree.
    if tree_query.is_default {
        if let Some(ctx) = tree_from_path(config, &tree_query.query) {
            result.push(ctx);
        }
    }

    result
}


/// Return tree contexts for every garden matching the specified pattern.
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - pattern: `&glob::Pattern`

pub fn garden_trees(
    config: &model::Configuration,
    pattern: &glob::Pattern,
) -> Vec<model::TreeContext> {

    let mut result = Vec::new();

    for garden in &config.gardens {
        if !pattern.matches(garden.name.as_ref()) {
            continue;
        }
        result.append(&mut trees_from_garden(config, &garden));
    }

    result
}


/// Return the tree contexts for a garden
pub fn trees_from_garden(
    config: &model::Configuration,
    garden: &model::Garden,
) -> Vec<model::TreeContext> {

    let mut result = Vec::new();

    // Loop over the garden's groups.
    for group in &garden.groups {
        // Create a glob pattern for the group entry
        let pattern_res = glob::Pattern::new(&group);
        if pattern_res.is_err() {
            continue;
        }
        let pattern = pattern_res.unwrap();
        // Loop over configured groups to find the matching groups
        for cfg_group in &config.groups {
            if !pattern.matches(&cfg_group.name) {
                continue;
            }
            // Match found -- take all of the discovered trees.
            result.append(
                &mut trees_from_group(config, Some(garden.index), cfg_group));
        }
    }

    // Collect indexes for each tree in this garden
    for tree in &garden.trees {
        result.append(&mut trees_from_pattern(
                config, tree, Some(garden.index), None));
    }

    result
}

/// Return the tree contexts for a garden
pub fn trees_from_group(
    config: &model::Configuration,
    garden: Option<model::GardenIndex>,
    group: &model::Group,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();

    // Collect indexes for each tree in this group
    for tree in &group.members {
        result.append(
            &mut trees_from_pattern(config, tree, garden, Some(group.index)));
    }

    result
}


/// Find a tree by name
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name `&str`
/// - garden_idx: `Option<garden::model::GardenIndex>`

pub fn tree_from_name(
    config: &model::Configuration,
    tree: &str,
    garden_idx: Option<model::GardenIndex>,
    group_idx: Option<model::GroupIndex>,
) -> Option<model::TreeContext> {


    // Collect tree indexes for the configured trees
    for (tree_idx, cfg_tree) in config.trees.iter().enumerate() {
        if *tree == cfg_tree.name {
            // Tree found
            return Some(model::TreeContext {
                tree: tree_idx,
                garden: garden_idx,
                group: group_idx,
            });
        }
    }

    // Try to find the specified name on the filesystem if no tree was found
    // that matched the specified name.  Matching trees are found by matching
    // tree paths against the specified name.

    if let Some(ctx) = tree_from_path(config, tree) {
        return Some(ctx);
    }

    None
}

/// Find trees matching a pattern
/// Parameters:
/// - config: `&garden::model::Configuration`
/// - tree: Tree name pattern `&str`
/// - garden_idx: `Option<garden::model::GardenIndex>`

pub fn trees_from_pattern(
    config: &model::Configuration,
    tree: &str,
    garden_idx: Option<model::GardenIndex>,
    group_idx: Option<model::GroupIndex>,
) -> Vec<model::TreeContext> {
    let mut result = Vec::new();

    let pattern_res = glob::Pattern::new(tree);
    if pattern_res.is_err() {
        return result;
    }
    let pattern = pattern_res.unwrap();

    // Collect tree indexes for the configured trees
    for (tree_idx, cfg_tree) in config.trees.iter().enumerate() {
        if pattern.matches(&cfg_tree.name) {
            // Tree found
            result.push(model::TreeContext {
                tree: tree_idx,
                garden: garden_idx,
                group: group_idx,
            });
        }
    }

    // Try to find the specified name on the filesystem if no tree was found
    // that matched the specified name.  Matching trees are found by matching
    // tree paths against the specified name.
    if result.is_empty() {
        if let Some(ctx) = tree_from_path(config, tree) {
            result.push(ctx);
        }
    }


    result
}


/// Return a tree context for the specified filesystem path.

pub fn tree_from_path(
    config: &model::Configuration,
    path: &str,
) -> Option<model::TreeContext> {

    let canon = std::path::PathBuf::from(path).canonicalize();
    if canon.is_err() {
        return None;
    }

    let pathbuf = canon.unwrap().to_path_buf();

    for (idx, tree) in config.trees.iter().enumerate() {
        let tree_path = tree.path.value.as_ref().unwrap();
        let tree_canon = std::path::PathBuf::from(tree_path).canonicalize();
        if tree_canon.is_err() {
            continue;
        }
        if pathbuf == tree_canon.unwrap() {
            return Some(
                model::TreeContext {
                    tree: idx as model::TreeIndex,
                    garden: None,
                    group: None,
                }
            );
        }
    }

    None
}

/// Returns tree contexts matching the specified pattern

fn trees(config: &model::Configuration, pattern: &glob::Pattern)
    -> Vec<model::TreeContext> {

    let mut result = Vec::new();
    for (tree_idx, tree) in config.trees.iter().enumerate() {
        if pattern.matches(tree.name.as_ref()) {
            result.push(
                model::TreeContext {
                    tree: tree_idx,
                    garden: None,
                    group: None,
                }
            );
        }
    }

    result
}


/// Return a Result<garden::model::TreeContext, String> when the tree and
/// optional garden are present.  Err is a String.

pub fn tree_context(config: &model::Configuration,
                    tree: &str, garden: Option<String>)
-> Result<model::TreeContext, String> {

    let mut ctx = model::TreeContext {
        tree: 0,
        garden: None,
        group: None,
    };
    if let Some(context) = tree_from_name(&config, tree, None, None) {
        ctx.tree = context.tree;
    } else {
        return Err(format!(
                "unable to find '{}': No tree exists with that name", tree));
    }

    if garden.is_some() {
        let pattern = glob::Pattern::new(garden.as_ref().unwrap()).unwrap();
        let contexts = query::garden_trees(config, &pattern);

        if contexts.is_empty() {
            return Err(format!(
                "unable to find '{}': No garden exists with that name",
                garden.unwrap()));
        }

        let mut found = false;
        for current_ctx in &contexts {
            if current_ctx.tree == ctx.tree {
                ctx.garden = current_ctx.garden;
                found = true;
                break;
            }
        }

        if !found {
            return Err(format!(
                "invalid arguments: '{}' is not part of the '{}' garden",
                tree, garden.unwrap()));
        }
    }

    Ok(ctx)
}
