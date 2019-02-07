extern crate garden;


fn from_yaml_string(string: &String) -> garden::model::Configuration {
    let mut config = garden::model::Configuration::new();
    let file_format = garden::config::FileFormat::YAML;
    garden::config::parse(string, file_format, false, &mut config);

    return config;
}


fn from_json_string(string: &String) -> garden::model::Configuration {
    let mut config = garden::model::Configuration::new();
    let file_format = garden::config::FileFormat::JSON;
    garden::config::parse(string, file_format, false, &mut config);

    return config;
}


/// Defaults
#[test]
fn config_default() {
    let config = garden::model::Configuration::new();
    assert_eq!(config.shell.to_string_lossy(), "zsh");
    assert_eq!(config.environment_variables, true);
    assert_eq!(config.verbose, false);
}


/// Core garden settings
#[test]
fn core() {
    let string = r#"
    garden:
        root: /tmp
        environment_variables: false
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.root_path, std::path::PathBuf::from("/tmp"));
    assert_eq!(config.environment_variables, false);
}

/// Variables
#[test]
fn variables() {
    let string = r#"
    variables:
        foo: foo_value
        bar: ${foo}
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.variables.len(), 2);

    assert_eq!(config.variables[0].name, "foo");
    assert_eq!(config.variables[0].expr, "foo_value");
    assert_eq!(config.variables[0].value, None);

    assert_eq!(config.variables[1].name, "bar");
    assert_eq!(config.variables[1].expr, "${foo}");
    assert_eq!(config.variables[1].value, None);
}

/// Commands
#[test]
fn commands() {
    let string = r#"
    commands:
        test_cmd: echo cmd
        test_cmd_vec:
            - echo first
            - echo second
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.commands.len(), 2);

    assert_eq!(config.commands[0].name, "test_cmd");
    assert_eq!(config.commands[0].values[0].expr, "echo cmd");

    assert_eq!(config.commands[1].name, "test_cmd_vec");
    assert_eq!(config.commands[1].values[0].expr, "echo first");
    assert_eq!(config.commands[1].values[1].expr, "echo second");
}


/// Templates
#[test]
fn templates() {
    let string = r#"
    templates:
        template1:
            variables:
                foo: bar
            environment:
                ENV=: ${foo}env
                THEPATH:
                    - ${foo}
                    - ${ENV}
        template2:
            extend: template1
            variables:
                baz: zax
                zee: ${foo}
        template3:
            extend: [template1, template2]
            variables:
                foo: boo
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.templates.len(), 3);
    assert_eq!(config.templates[0].name, "template1");
    assert_eq!(config.templates[0].variables.len(), 1);
    assert_eq!(config.templates[0].variables[0].name, "foo");
    assert_eq!(config.templates[0].variables[0].expr, "bar");

    assert_eq!(config.templates[0].environment.len(), 2);
    assert_eq!(config.templates[0].environment[0].name, "ENV=");
    assert_eq!(config.templates[0].environment[0].values.len(), 1);
    assert_eq!(config.templates[0].environment[0].values[0].expr, "${foo}env");

    assert_eq!(config.templates[0].environment[1].name, "THEPATH");
    assert_eq!(config.templates[0].environment[1].values.len(), 2);
    assert_eq!(config.templates[0].environment[1].values[0].expr, "${foo}");
    assert_eq!(config.templates[0].environment[1].values[1].expr, "${ENV}");

    assert_eq!(config.templates[1].name, "template2");
    assert_eq!(config.templates[1].extend, ["template1"]);
    assert_eq!(config.templates[1].variables.len(), 2);
    assert_eq!(config.templates[1].variables[0].name, "baz");
    assert_eq!(config.templates[1].variables[0].expr, "zax");
    assert_eq!(config.templates[1].variables[1].name, "zee");
    assert_eq!(config.templates[1].variables[1].expr, "${foo}");

    assert_eq!(config.templates[2].name, "template3");
    assert_eq!(config.templates[2].extend, ["template1", "template2"]);
    assert_eq!(config.templates[2].variables.len(), 1);
    assert_eq!(config.templates[2].variables[0].name, "foo");
    assert_eq!(config.templates[2].variables[0].expr, "boo");
}


/// Groups
#[test]
fn groups() {
    let string = r#"
    groups:
        cola: [git, qtpy, cola]
        test: [a, b, c]
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.groups.len(), 2);
    assert_eq!(config.groups[0].name, "cola");
    assert_eq!(config.groups[0].members, ["git", "qtpy", "cola"]);

    assert_eq!(config.groups[1].name, "test");
    assert_eq!(config.groups[1].members, ["a", "b", "c"]);
}

/// Trees
#[test]
fn trees() {
    let string = r#"
    templates:
        makefile:
            commands:
                install: make -j prefix=${prefix} install
                test: make test
        python:
            environment:
                PYTHONPATH: ${TREE_PATH}
    trees:
        git:
            url: https://github.com/git/git
            templates: makefile
            variables:
                prefix: ~/.local
            gitconfig:
                user.name: A U Thor
                user.email: author@example.com
        cola:
            url: https://github.com/git-cola/git-cola
            path: git-cola
            templates: [makefile, python]
            variables:
                prefix: ${TREE_PATH}/local
            environment:
                PATH:
                    - ${TREE_PATH}/bin
                    - ${prefix}
                PYTHONPATH: ${TREE_PATH}
            commands:
                test:
                    - git status --short
                    - make test
            remotes:
                davvid: git@github.com:davvid/git-cola.git
    "#.to_string();

    let config = from_yaml_string(&string);
    assert_eq!(config.trees.len(), 2);

    // git
    let ref tree0 = config.trees[0];
    assert!(tree0.environment.is_empty());
    assert!(tree0.commands.is_empty());

    assert_eq!(tree0.name, "git");
    assert_eq!(tree0.path, "git");  // picks up default value
    assert_eq!(tree0.templates, ["makefile"]);

    assert_eq!(tree0.remotes.len(), 1);
    assert_eq!(tree0.remotes[0].name, "origin");
    assert_eq!(tree0.remotes[0].url, "https://github.com/git/git");

    assert_eq!(tree0.variables.len(), 1);
    assert_eq!(tree0.variables[0].name, "prefix");
    assert_eq!(tree0.variables[0].expr, "~/.local");

    assert_eq!(tree0.gitconfig.len(), 2);
    assert_eq!(tree0.gitconfig[0].name, "user.name");
    assert_eq!(tree0.gitconfig[0].expr, "A U Thor");
    assert_eq!(tree0.gitconfig[0].value, None);
    assert_eq!(tree0.gitconfig[1].name, "user.email");
    assert_eq!(tree0.gitconfig[1].expr, "author@example.com");
    assert_eq!(tree0.gitconfig[1].value, None);

    // cola
    let ref tree1 = config.trees[1];
    assert!(tree1.gitconfig.is_empty());

    assert_eq!(tree1.name, "cola");
    assert_eq!(tree1.path, "git-cola");
    assert_eq!(tree1.templates, ["makefile", "python"]);

    assert_eq!(tree1.remotes.len(), 2);
    assert_eq!(tree1.remotes[0].name, "origin");
    assert_eq!(tree1.remotes[0].url, "https://github.com/git-cola/git-cola");
    assert_eq!(tree1.remotes[1].name, "davvid");
    assert_eq!(tree1.remotes[1].url, "git@github.com:davvid/git-cola.git");

    assert_eq!(tree1.environment.len(), 2);
    assert_eq!(tree1.environment[0].name, "PATH");
    assert_eq!(tree1.environment[0].values.len(), 2);
    assert_eq!(tree1.environment[0].values[0].expr, "${TREE_PATH}/bin");
    assert_eq!(tree1.environment[0].values[1].expr, "${prefix}");

    assert_eq!(tree1.environment[1].name, "PYTHONPATH");
    assert_eq!(tree1.environment[1].values.len(), 1);
    assert_eq!(tree1.environment[1].values[0].expr, "${TREE_PATH}");

    assert_eq!(tree1.commands.len(), 1);
    assert_eq!(tree1.commands[0].name, "test");
    assert_eq!(tree1.commands[0].values.len(), 2);
    assert_eq!(tree1.commands[0].values[0].expr, "git status --short");
    assert_eq!(tree1.commands[0].values[1].expr, "make test");
}


/// Gardens
#[test]
fn gardens() {
    let string = r#"
    gardens:
        cola:
            groups: cola
            variables:
                prefix: ~/src/git-cola/local/git-cola
            environment:
                GIT_COLA_TRACE=: full
                PATH+: ${prefix}
            commands:
                summary:
                    - git branch -vv
                    - git status --short
        git:
            groups: cola
            trees: gitk
            gitconfig:
                user.name: A U Thor
                user.email: author@example.com
    "#.to_string();

    let config = from_yaml_string(&string);
    test_gardens(&config);
}

#[test]
fn gardens_json() {
    let string = r#"
{
    "gardens": {
        "cola": {
            "groups": "cola",
            "variables": {
                "prefix": "~/src/git-cola/local/git-cola"
            },
            "environment": {
                "GIT_COLA_TRACE=": "full",
                "PATH+": "${prefix}"
            },
            "commands": {
                "summary": [
                    "git branch -vv",
                    "git status --short"
                ]
            }
        },
        "git": {
            "groups": "cola",
            "trees": "gitk",
            "gitconfig": {
                "user.name": "A U Thor",
                "user.email": "author@example.com"
            }
        }
    }
}
    "#.to_string();

    let config = from_json_string(&string);
    test_gardens(&config);
}

fn test_gardens(config: &garden::model::Configuration) {
    assert_eq!(config.gardens.len(), 2);

    // "cola" garden
    assert_eq!(config.gardens[0].name, "cola");

    assert!(config.gardens[0].trees.is_empty());
    assert!(config.gardens[0].gitconfig.is_empty());

    assert_eq!(config.gardens[0].groups.len(), 1);
    assert_eq!(config.gardens[0].groups[0], "cola");

    assert_eq!(config.gardens[0].commands.len(), 1);
    assert_eq!(config.gardens[0].commands[0].name, "summary");
    assert_eq!(config.gardens[0].commands[0].values.len(), 2);
    assert_eq!(config.gardens[0].commands[0].values[0].expr,
               "git branch -vv");
    assert_eq!(config.gardens[0].commands[0].values[1].expr,
               "git status --short");

    assert_eq!(config.gardens[0].variables.len(), 1);
    assert_eq!(config.gardens[0].variables[0].name, "prefix");
    assert_eq!(config.gardens[0].variables[0].expr,
               "~/src/git-cola/local/git-cola");

    assert_eq!(config.gardens[0].environment.len(), 2);
    assert_eq!(config.gardens[0].environment[0].name, "GIT_COLA_TRACE=");
    assert_eq!(config.gardens[0].environment[0].values.len(), 1);
    assert_eq!(config.gardens[0].environment[0].values[0].expr, "full");

    assert_eq!(config.gardens[0].environment[1].name, "PATH+");
    assert_eq!(config.gardens[0].environment[1].values.len(), 1);
    assert_eq!(config.gardens[0].environment[1].values[0].expr, "${prefix}");

    // "git" garden
    assert_eq!(config.gardens[1].name, "git");

    assert!(config.gardens[1].environment.is_empty());
    assert!(config.gardens[1].variables.is_empty());
    assert!(config.gardens[1].commands.is_empty());

    assert_eq!(config.gardens[1].groups, ["cola"]);
    assert_eq!(config.gardens[1].trees, ["gitk"]);

    assert_eq!(config.gardens[1].gitconfig.len(), 2);
    assert_eq!(config.gardens[1].gitconfig[0].name, "user.name");
    assert_eq!(config.gardens[1].gitconfig[0].expr, "A U Thor");
    assert_eq!(config.gardens[1].gitconfig[1].name, "user.email");
    assert_eq!(config.gardens[1].gitconfig[1].expr, "author@example.com");
}
