use dprint_core::configuration::{
    get_unknown_property_diagnostics, ConfigKeyMap, GlobalConfiguration, ResolveConfigurationResult,
};
use dprint_core::plugins::PluginInfo;
use dprint_core::types::ErrBox;
use itertools::Itertools;
use rowan::{GreenNode, GreenNodeBuilder, GreenToken, NodeOrToken, SmolStr, SyntaxElement};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::Path;
use toml_parse::{parse_it, Formatter, SyntaxNode, SyntaxNodeExtTrait, TomlKind};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use dprint_core::plugins::PluginHandler as _;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
dprint_core::generate_plugin_code!(PluginHandler, PluginHandler);

pub struct PluginHandler;

impl dprint_core::plugins::PluginHandler<Configuration> for PluginHandler {
    fn resolve_config(
        &mut self,
        config: ConfigKeyMap,
        _: &GlobalConfiguration,
    ) -> ResolveConfigurationResult<Configuration> {
        let mut diagnostics = Vec::new();

        diagnostics.extend(get_unknown_property_diagnostics(config));

        ResolveConfigurationResult {
            config: Configuration {},
            diagnostics,
        }
    }

    fn get_plugin_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: "cargo-toml-fmt".to_string(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            config_key: "cargo_toml".to_string(),
            file_extensions: vec![],
            file_names: vec!["Cargo.toml".to_owned()],
            help_url: "".to_string(),
            config_schema_url: "".to_string(),
        }
    }

    fn get_license_text(&mut self) -> String {
        std::str::from_utf8(include_bytes!("../LICENSE"))
            .unwrap()
            .into()
    }

    fn format_text(
        &mut self,
        file_path: &Path,
        file_text: &str,
        _: &Configuration,
        _: impl FnMut(&Path, String, &ConfigKeyMap) -> Result<String, ErrBox>,
    ) -> Result<String, ErrBox> {
        match file_path.file_name() {
            Some(file_name) if file_name == "Cargo.toml" => {}
            _ => return Ok(file_text.to_string()),
        }

        let toml = parse_it(file_text).map_err(|e| e.to_string())?.syntax();

        let mut builder = GreenNodeBuilder::new();
        builder.start_node(TomlKind::Root.into());

        for node in toml.children() {
            match table_heading(&node) {
                Some(heading) if heading.contains("[package]") => {
                    sort_table_entries(&node, &mut builder, |left, right| {
                        match (
                            key_name(left.as_ref()).as_deref(),
                            key_name(right.as_ref()).as_deref(),
                        ) {
                            (Some("name"), Some(_)) => Ordering::Less,
                            (Some(_), Some("name")) => Ordering::Greater,
                            (Some("version"), Some(_)) => Ordering::Less,
                            (Some(_), Some("version")) => Ordering::Greater,
                            (Some("description"), Some("")) => Ordering::Less,
                            (Some(""), Some("description")) => Ordering::Greater,
                            (Some("description"), Some(_)) => Ordering::Greater,
                            (Some(_), Some("description")) => Ordering::Less,
                            (Some(left), Some(right)) => left.cmp(right),
                            _ => Ordering::Equal,
                        }
                    })
                }
                Some(heading)
                    if heading.contains("[dependencies]")
                        || heading.contains("[dev-dependencies]") =>
                {
                    sort_table_entries(&node, &mut builder, |left, right| {
                        match (
                            key_name(left.as_ref()).as_deref(),
                            key_name(right.as_ref()).as_deref(),
                        ) {
                            (Some(left), Some(right)) => left.cmp(right),
                            _ => Ordering::Equal,
                        }
                    });
                }
                _ => {
                    add_node(&node, &mut builder);
                }
            }
        }

        builder.finish_node();
        let formatted_toml = SyntaxNode::new_root(builder.finish());

        Ok(Formatter::new(&formatted_toml).format().to_string())
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {}

enum TableEntry {
    Regular(SyntaxNode),
    WithComment {
        entry: SyntaxNode,
        comment: SyntaxNode,
    },
}

impl AsRef<SyntaxNode> for TableEntry {
    fn as_ref(&self) -> &SyntaxNode {
        match self {
            TableEntry::Regular(i) => i,
            TableEntry::WithComment { entry, .. } => entry,
        }
    }
}

fn sort_table_entries<C>(node: &SyntaxNode, builder: &mut GreenNodeBuilder, cmp: C)
where
    C: Fn(&TableEntry, &TableEntry) -> Ordering,
{
    builder.start_node(TomlKind::Table.into());

    let mut children = node.children();

    let heading = children.next().unwrap();
    add_node(&heading, builder);

    for child in children
        .filter_map(|c| {
            if c.kind() == TomlKind::Comment {
                return None;
            }

            match c.next_sibling() {
                Some(sibling) if sibling.kind() == TomlKind::Comment => {
                    Some(TableEntry::WithComment {
                        entry: c,
                        comment: sibling,
                    })
                }
                _ => Some(TableEntry::Regular(c)),
            }
        })
        .sorted_by(cmp)
        .map(|entry| {
            let key_value = entry.as_ref();

            let last = key_value.last_token().expect("table child to not be empty");

            if last.text() == "\n" {
                return entry;
            }

            let table_entries = key_value
                .children_with_tokens()
                .map(|c| match c {
                    NodeOrToken::Node(node) => NodeOrToken::Node(node.green().clone()),
                    NodeOrToken::Token(token) => {
                        NodeOrToken::Token(if token == last && last.text() == "\n\n" {
                            GreenToken::new(TomlKind::Whitespace.into(), SmolStr::new("\n"))
                        } else {
                            token.green().clone()
                        })
                    }
                })
                .collect::<Vec<_>>();
            let node1 = GreenNode::new(TomlKind::KeyValue.into(), table_entries);

            match entry {
                TableEntry::Regular(_) => TableEntry::Regular(SyntaxNode::new_root(node1)),
                TableEntry::WithComment { comment, .. } => TableEntry::WithComment {
                    entry: SyntaxNode::new_root(node1),
                    comment,
                },
            }
        })
    {
        match child {
            TableEntry::Regular(entry) => add_node(&entry, builder),
            TableEntry::WithComment { entry, comment, .. } => {
                add_node(&entry, builder);
                add_node(&comment, builder);
            }
        }
    }

    builder.finish_node();
}

fn table_heading(node: &SyntaxNode) -> Option<String> {
    let table_heading = match node.kind() {
        TomlKind::Table => node.first_child().unwrap(),
        _ => return None,
    };

    Some(table_heading.token_text())
}

fn key_name(node: &SyntaxNode) -> Option<String> {
    let key_value = match node.kind() {
        TomlKind::KeyValue => node.first_child().unwrap(),
        _ => return None,
    };

    Some(key_value.token_text())
}

fn add_node(node: &SyntaxNode, builder: &mut GreenNodeBuilder) {
    builder.start_node(node.kind().into());

    for kid in node.children_with_tokens() {
        match kid {
            SyntaxElement::Node(n) => add_node(&n, builder),
            SyntaxElement::Token(t) => builder.token(t.kind().into(), t.text().clone()),
        }
    }

    builder.finish_node();
}
