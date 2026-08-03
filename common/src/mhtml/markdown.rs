use ego_tree::NodeRef;
use scraper::{Html, Node};

/// Convert cleaned HTML body to Markdown text.
pub fn html_to_markdown(html: &str) -> String {
    let document = Html::parse_fragment(html);
    let mut output = String::new();
    render_node(&document.tree.root(), &mut output);
    // Collapse triple+ newlines to double.
    let re = regex::Regex::new(r"\n{3,}").unwrap();
    re.replace_all(output.trim(), "\n\n").to_string() + "\n"
}

fn render_node(node: &NodeRef<Node>, out: &mut String) {
    match node.value() {
        Node::Text(text) => {
            out.push_str(text);
        }
        Node::Element(elem) => {
            let tag = elem.name();
            match tag {
                "h1" => {
                    out.push_str("\n\n# ");
                    render_children(node, out);
                    out.push_str("\n\n");
                }
                "h2" => {
                    out.push_str("\n\n## ");
                    render_children(node, out);
                    out.push_str("\n\n");
                }
                "h3" => {
                    out.push_str("\n\n### ");
                    render_children(node, out);
                    out.push_str("\n\n");
                }
                "h4" | "h5" | "h6" => {
                    out.push_str("\n\n#### ");
                    render_children(node, out);
                    out.push_str("\n\n");
                }
                "p" | "div" => {
                    out.push_str("\n\n");
                    render_children(node, out);
                    out.push_str("\n\n");
                }
                "br" => {
                    out.push('\n');
                }
                "hr" => {
                    out.push_str("\n\n---\n\n");
                }
                "strong" | "b" => {
                    out.push_str("**");
                    render_children(node, out);
                    out.push_str("**");
                }
                "em" | "i" => {
                    out.push('*');
                    render_children(node, out);
                    out.push('*');
                }
                "a" => {
                    let href = elem.attr("href").unwrap_or("");
                    out.push('[');
                    render_children(node, out);
                    out.push_str("](");
                    out.push_str(href);
                    out.push(')');
                }
                "img" => {
                    let src = elem.attr("src").unwrap_or("");
                    let alt = elem.attr("alt").unwrap_or("");
                    if !src.is_empty() && !src.starts_with("data:") {
                        out.push_str(&format!("![{}]({})", alt, src));
                    }
                }
                "blockquote" => {
                    out.push_str("\n\n");
                    let mut inner = String::new();
                    render_children(node, &mut inner);
                    for line in inner.trim().lines() {
                        out.push_str("> ");
                        out.push_str(line.trim());
                        out.push('\n');
                    }
                    out.push('\n');
                }
                "ul" | "ol" => {
                    out.push_str("\n\n");
                    render_children(node, out);
                    out.push_str("\n\n");
                }
                "li" => {
                    out.push_str("- ");
                    render_children(node, out);
                    out.push('\n');
                }
                "code" => {
                    out.push('`');
                    render_children(node, out);
                    out.push('`');
                }
                "pre" => {
                    out.push_str("\n\n```\n");
                    render_children(node, out);
                    out.push_str("\n```\n\n");
                }
                // Skip these entirely.
                "script" | "style" | "button" | "svg" | "nav" => {}
                // Everything else: just render children.
                _ => {
                    render_children(node, out);
                }
            }
        }
        // Document / Fragment root — recurse into children.
        _ => {
            render_children(node, out);
        }
    }
}

fn render_children(node: &NodeRef<Node>, out: &mut String) {
    for child in node.children() {
        render_node(&child, out);
    }
}
