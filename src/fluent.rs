use crate::parser::{Comment, Node};
use convert_case::{Case, Casing};
use std::collections::HashMap;

const LICENSE: &'static str =
    "# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

### TODO - Write a description of what this FTL represents.

";

pub fn dtd_to_fluent(nodes: &Vec<Node>) -> String {
    let mut key_to_node: HashMap<&str, usize> = HashMap::new();

    for (index, node) in nodes.iter().enumerate() {
        if let Node::Comment(Comment {
            value: _,
            key: Some(key),
        }) = node
        {
            key_to_node.insert(key, index);
        }
    }

    let mut iter = nodes.iter().peekable();
    // Check for the license, and skip it if it exists.
    if let Some(Node::Comment(comment)) = iter.peek() {
        if comment.value.contains("http://mozilla.org/MPL/2.0/") {
            iter.next();
        }
    }

    let mut text: String = LICENSE.into();
    for node in iter {
        match node {
            Node::Entity(entity) => {
                // Add a comment that belongs to this.
                if let Some(comment_index) = key_to_node.get(entity.key) {
                    if let Some(Node::Comment(comment)) = nodes.get(*comment_index) {
                        text.push_str("\n");
                        for line in comment.value.lines() {
                            let mut line = line.trim();
                            let mut char_iter = line.chars();
                            if let Some('-') = char_iter.next() {
                                line = char_iter.as_str().trim();
                            }

                            text.push_str("# ");
                            text.push_str(line.trim());
                            text.push_str("\n");
                        }
                    }
                }

                // Add the key.
                text.push_str(&entity.key.replace('.', "-").to_case(Case::Kebab));

                // Push on the = part
                if entity.value.contains("\n") {
                    text.push_str(" =");
                    text.push_str("\n");
                    for line in entity.value.trim().lines() {
                        text.push_str("  ");
                        text.push_str(line);
                        text.push_str("\n");
                    }
                } else {
                    text.push_str(" = ");
                    text.push_str(&entity.value);
                    text.push('\n');
                }
            }
            Node::Comment(comment) => {
                if comment.key.is_some() {
                    continue;
                }
                {
                    let mut chars = text.chars();
                    chars.next_back();
                    if chars.next_back() != Some('\n') {
                        text.push('\n');
                    }
                }
                for line in comment.value.lines() {
                    text.push_str("## ");
                    text.push_str(line);
                    text.push('\n');
                }
                text.push('\n');
            }
        }
    }
    return text;
}
