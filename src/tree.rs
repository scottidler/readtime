use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::walker::FileInfo;

/// A node in the file tree
#[derive(Debug)]
pub enum Node {
    File {
        name: String,
        read_time_minutes: usize,
    },
    Directory {
        name: String,
        children: Vec<Node>,
        total_read_time_minutes: usize,
    },
}

impl Node {
    pub fn name(&self) -> &str {
        match self {
            Node::File { name, .. } => name,
            Node::Directory { name, .. } => name,
        }
    }

    pub fn read_time(&self) -> usize {
        match self {
            Node::File {
                read_time_minutes, ..
            } => *read_time_minutes,
            Node::Directory {
                total_read_time_minutes,
                ..
            } => *total_read_time_minutes,
        }
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, Node::Directory { .. })
    }
}

/// Build a tree structure from a list of files
pub fn build_tree(files: &[FileInfo], base_path: &Path) -> Node {
    if files.len() == 1 && files[0].path == base_path {
        // Single file mode
        return Node::File {
            name: base_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            read_time_minutes: files[0].read_time_minutes,
        };
    }

    // Build a nested tree structure
    let mut root_children: BTreeMap<PathBuf, Vec<FileInfo>> = BTreeMap::new();

    for file in files {
        if let Ok(relative) = file.path.strip_prefix(base_path) {
            let mut current_path = base_path.to_path_buf();

            // Get the immediate child under base_path
            if let Some(first_component) = relative.components().next() {
                current_path.push(first_component);
                root_children
                    .entry(current_path)
                    .or_default()
                    .push(file.clone());
            }
        }
    }

    let mut children = Vec::new();
    let mut total_time = 0;

    for (path, child_files) in root_children {
        if path.is_file() {
            // It's a file directly under base_path
            if let Some(file_info) = child_files.first() {
                total_time += file_info.read_time_minutes;
                children.push(Node::File {
                    name: path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    read_time_minutes: file_info.read_time_minutes,
                });
            }
        } else {
            // It's a directory, recurse
            let subnode = build_tree(&child_files, &path);
            total_time += subnode.read_time();
            children.push(subnode);
        }
    }

    // Sort children: directories first, then files, alphabetically within each group
    children.sort_by(|a, b| match (a.is_directory(), b.is_directory()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name().cmp(b.name()),
    });

    Node::Directory {
        name: base_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(".")
            .to_string(),
        children,
        total_read_time_minutes: total_time,
    }
}

/// Render the tree to a string
pub fn render_tree(node: &Node, base_name: Option<&str>) -> String {
    let mut output = String::new();

    match node {
        Node::File {
            name,
            read_time_minutes,
        } => {
            let display_name = base_name.unwrap_or(name);
            output.push_str(&format!(
                "{:50} {:>6} min\n",
                display_name, read_time_minutes
            ));
        }
        Node::Directory {
            name,
            children,
            total_read_time_minutes,
        } => {
            let display_name = base_name.unwrap_or(name);
            output.push_str(&format!("{}/\n", display_name));

            for (i, child) in children.iter().enumerate() {
                let is_last = i == children.len() - 1;
                render_node(child, "", is_last, &mut output);
            }

            output.push_str(&format!("\nTotal: {} min\n", total_read_time_minutes));
        }
    }

    output
}

fn render_node(node: &Node, prefix: &str, is_last: bool, output: &mut String) {
    let connector = if is_last { "└── " } else { "├── " };
    let extension = if is_last { "    " } else { "│   " };

    match node {
        Node::File {
            name,
            read_time_minutes,
        } => {
            output.push_str(&format!(
                "{}{}{:40} {:>6} min\n",
                prefix, connector, name, read_time_minutes
            ));
        }
        Node::Directory {
            name,
            children,
            total_read_time_minutes,
        } => {
            output.push_str(&format!(
                "{}{}{:40} {:>6} min\n",
                prefix,
                connector,
                format!("{}/", name),
                total_read_time_minutes
            ));

            let new_prefix = format!("{}{}", prefix, extension);
            for (i, child) in children.iter().enumerate() {
                let child_is_last = i == children.len() - 1;
                render_node(child, &new_prefix, child_is_last, output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_file() {
        let files = vec![FileInfo {
            path: PathBuf::from("/tmp/test.md"),
            read_time_minutes: 1,
        }];

        let tree = build_tree(&files, Path::new("/tmp/test.md"));
        assert!(matches!(tree, Node::File { .. }));
    }

    #[test]
    fn test_directory_with_files() {
        let files = vec![
            FileInfo {
                path: PathBuf::from("/tmp/dir/file1.md"),
                read_time_minutes: 1,
            },
            FileInfo {
                path: PathBuf::from("/tmp/dir/file2.md"),
                read_time_minutes: 2,
            },
        ];

        let tree = build_tree(&files, Path::new("/tmp/dir"));

        if let Node::Directory {
            total_read_time_minutes,
            children,
            ..
        } = tree
        {
            assert_eq!(total_read_time_minutes, 3);
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected directory node");
        }
    }

    #[test]
    fn test_render_single_file() {
        let node = Node::File {
            name: "test.md".to_string(),
            read_time_minutes: 5,
        };

        let output = render_tree(&node, None);
        assert!(output.contains("test.md"));
        assert!(output.contains("5 min"));
    }

    #[test]
    fn test_render_directory() {
        let node = Node::Directory {
            name: "docs".to_string(),
            children: vec![
                Node::File {
                    name: "readme.md".to_string(),
                    read_time_minutes: 2,
                },
                Node::File {
                    name: "guide.md".to_string(),
                    read_time_minutes: 5,
                },
            ],
            total_read_time_minutes: 7,
        };

        let output = render_tree(&node, None);
        assert!(output.contains("docs/"));
        assert!(output.contains("readme.md"));
        assert!(output.contains("guide.md"));
        assert!(output.contains("Total: 7 min"));
    }
}
