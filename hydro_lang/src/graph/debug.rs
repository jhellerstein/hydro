//! Debugging utilities for Hydro IR graph visualization.
//!
//! Similar to the DFIR debugging utilities, this module provides convenient
//! methods for opening graphs in web browsers and VS Code.

use std::fmt::Write;
use std::io::Result;

use super::render::{HydroWriteConfig, render_hydro_ir_dot, render_hydro_ir_mermaid};
use super::template::get_template;
use crate::ir::HydroLeaf;

/// Debugging extensions for Hydro IR.
impl HydroLeaf {
    /// Opens this Hydro IR graph as a mermaid diagram in the [mermaid.live](https://mermaid.live) browser editor.

    pub fn open_mermaid(&self, config: Option<HydroWriteConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        let mermaid_src = self.to_mermaid(&config);
        open_mermaid_browser(&mermaid_src)
    }

    /// Opens this Hydro IR graph as a DOT/Graphviz diagram in the browser.

    pub fn open_dot(&self, config: Option<HydroWriteConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        let dot_src = self.to_dot(&config);
        open_dot_browser(&dot_src)
    }

    /// Saves this Hydro IR graph as a .mermaid file and opens it in VS Code for preview.
    /// Requires the "Mermaid Preview" extension in VS Code.

    pub fn open_mermaid_vscode(
        &self,
        filename: Option<&str>,
        config: Option<HydroWriteConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();
        let mermaid_src = self.to_mermaid(&config);
        let filename = filename.unwrap_or("hydro_graph.mermaid");
        save_and_open_vscode(&mermaid_src, filename)
    }

    /// Saves this Hydro IR graph as a .dot file and opens it in VS Code for preview.
    /// Requires a Graphviz extension in VS Code.

    pub fn open_dot_vscode(
        &self,
        filename: Option<&str>,
        config: Option<HydroWriteConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();
        let dot_src = self.to_dot(&config);
        let filename = filename.unwrap_or("hydro_graph.dot");
        save_and_open_vscode(&dot_src, filename)
    }

    /// Saves this Hydro IR graph as a ReactFlow.js JSON file and opens it in a browser.
    /// Creates a complete HTML file with ReactFlow.js visualization.

    pub fn open_reactflow_browser(
        &self,
        filename: Option<&str>,
        config: Option<HydroWriteConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();
        let reactflow_json = self.to_reactflow(&config);
        let filename = filename.unwrap_or("hydro_graph.html");
        save_and_open_reactflow_browser(&reactflow_json, filename)
    }

    /// Saves this Hydro IR graph as a ReactFlow.js JSON file.

    pub fn save_reactflow_json(
        &self,
        filename: Option<&str>,
        config: Option<HydroWriteConfig>,
    ) -> Result<()> {
        let config = config.unwrap_or_default();
        let reactflow_json = self.to_reactflow(&config);
        let filename = filename.unwrap_or("hydro_graph.json");
        std::fs::write(filename, reactflow_json)?;
        Ok(())
    }
}

/// Opens multiple Hydro IR leaves as a single mermaid diagram.

pub fn open_hydro_ir_mermaid(leaves: &[HydroLeaf], config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let mermaid_src = render_hydro_ir_mermaid(leaves, &config);
    open_mermaid_browser(&mermaid_src)
}

/// Opens multiple Hydro IR leaves as a single DOT diagram.

pub fn open_hydro_ir_dot(leaves: &[HydroLeaf], config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let dot_src = render_hydro_ir_dot(leaves, &config);
    open_dot_browser(&dot_src)
}

/// Saves multiple Hydro IR leaves as a .mermaid file and opens it in VS Code.

pub fn open_hydro_ir_mermaid_vscode(leaves: &[HydroLeaf], filename: &str) -> Result<()> {
    let config = HydroWriteConfig::default();
    let mermaid_content = render_hydro_ir_mermaid(leaves, &config);

    // Write to file
    std::fs::write(filename, mermaid_content)?;

    // Open the file in VS Code
    let vscode_result = std::process::Command::new("code").arg(filename).status();

    match vscode_result {
        Ok(status) if status.success() => {
            println!("Opened {} in VS Code", filename);
            println!("To view the Mermaid diagram:");
            println!("  1. Press Ctrl+Shift+P (or Cmd+Shift+P on Mac)");
            println!("  2. Search for 'Mermaid Preview'");
            println!("  3. Select 'Mermaid Preview: Open Preview to the Side'");
            println!(
                "Or simply click the 'Open Preview to the Side' button in the top right of the editor."
            );
            Ok(())
        }
        _ => {
            // Fallback to browser
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let url = format!("https://mermaid.live/edit#{}", timestamp);
            let result = std::process::Command::new("open").arg(&url).status();

            match result {
                Ok(status) if status.success() => {
                    println!(
                        "VS Code not available. Opened {} in browser at {}",
                        filename, url
                    );
                    println!(
                        "Please copy the contents of {} and paste into the editor",
                        filename
                    );
                    Ok(())
                }
                Ok(_) => {
                    println!("Failed to open browser");
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to open browser",
                    ))
                }
                Err(e) => {
                    println!("Failed to open Mermaid viewer: {}", e);
                    Err(e)
                }
            }
        }
    }
}
/// Saves multiple Hydro IR leaves as a .dot file and opens it in VS Code.

pub fn open_hydro_ir_dot_vscode(
    leaves: &[HydroLeaf],
    filename: Option<&str>,
    config: Option<HydroWriteConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let dot_src = render_hydro_ir_dot(leaves, &config);
    let filename = filename.unwrap_or("hydro_graph.dot");
    save_and_open_vscode(&dot_src, filename)
}

/// Opens a Mermaid diagram in VS Code's Simple Browser using mermaid.live.

pub fn open_hydro_ir_mermaid_simple_browser(
    leaves: &[HydroLeaf],
    config: Option<HydroWriteConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let mermaid_src = render_hydro_ir_mermaid(leaves, &config);
    open_mermaid_vscode_browser(&mermaid_src)
}

/// Opens a DOT diagram in VS Code's Simple Browser.

pub fn open_hydro_ir_dot_simple_browser(
    leaves: &[HydroLeaf],
    config: Option<HydroWriteConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let dot_src = render_hydro_ir_dot(leaves, &config);
    open_dot_vscode_browser(&dot_src)
}

/// Opens multiple Hydro IR leaves as a ReactFlow.js visualization in a browser.
/// Creates a complete HTML file with ReactFlow.js interactive graph visualization.

pub fn open_hydro_ir_reactflow_browser(
    leaves: &[HydroLeaf],
    filename: Option<&str>,
    config: Option<HydroWriteConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let reactflow_json = render_hydro_ir_reactflow(leaves, &config);
    let filename = filename.unwrap_or("hydro_graph.html");
    save_and_open_reactflow_browser(&reactflow_json, filename)
}

/// Saves multiple Hydro IR leaves as a ReactFlow.js JSON file.

pub fn save_hydro_ir_reactflow_json(
    leaves: &[HydroLeaf],
    filename: Option<&str>,
    config: Option<HydroWriteConfig>,
) -> Result<()> {
    let config = config.unwrap_or_default();
    let reactflow_json = render_hydro_ir_reactflow(leaves, &config);
    let filename = filename.unwrap_or("hydro_graph.json");
    std::fs::write(filename, reactflow_json)?;
    println!("Saved ReactFlow.js JSON to {}", filename);
    Ok(())
}

fn open_mermaid_browser(mermaid_src: &str) -> Result<()> {
    let state = serde_json::json!({
        "code": mermaid_src,
        "mermaid": serde_json::json!({
            "theme": "default"
        }),
        "autoSync": true,
        "updateDiagram": true
    });
    let state_json = serde_json::to_vec(&state)?;
    let state_base64 = data_encoding::BASE64URL.encode(&state_json);
    webbrowser::open(&format!(
        "https://mermaid.live/edit#base64:{}",
        state_base64
    ))
}

fn open_dot_browser(dot_src: &str) -> Result<()> {
    let mut url = "https://dreampuf.github.io/GraphvizOnline/#".to_owned();
    for byte in dot_src.bytes() {
        // Lazy percent encoding: https://en.wikipedia.org/wiki/Percent-encoding
        write!(url, "%{:02x}", byte).unwrap();
    }
    webbrowser::open(&url)
}

/// Saves content to a file and opens it in VS Code.

fn save_and_open_vscode(content: &str, filename: &str) -> Result<()> {
    use std::fs;
    use std::process::Command;

    // Save the content to a file
    fs::write(filename, content)?;

    // Try to open in VS Code
    if Command::new("code").arg(filename).status().is_ok() {
        println!("Opened {} in VS Code", filename);
        println!("For Mermaid files: Install 'Mermaid Preview' extension");
        println!("For DOT files: Install 'Graphviz (dot) language support' extension");
        Ok(())
    } else {
        // Fall back to system default
        webbrowser::open(&format!(
            "file://{}",
            std::env::current_dir()?.join(filename).display()
        ))?;
        println!(
            "VS Code not found, opened {} with default application",
            filename
        );
        Ok(())
    }
}

/// Opens a Mermaid diagram in VS Code's Simple Browser using mermaid.live.

fn open_mermaid_vscode_browser(mermaid_src: &str) -> Result<()> {
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create the standard mermaid.live state
    let state = serde_json::json!({
        "code": mermaid_src,
        "mermaid": {
            "theme": "default"
        }
    });
    let state_str = state.to_string();
    let encoded = data_encoding::BASE64URL_NOPAD.encode(state_str.as_bytes());

    // Add timestamp as query parameter to avoid browser caching
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let url = format!("https://mermaid.live/edit#{}?v={}", encoded, timestamp);

    // Try to open in VS Code Simple Browser first
    if Command::new("code")
        .arg("--command")
        .arg("simpleBrowser.show")
        .arg(&url)
        .status()
        .is_ok()
    {
        println!("Opened Mermaid diagram in VS Code Simple Browser");
        Ok(())
    } else {
        // Fall back to regular browser
        webbrowser::open(&url)?;
        println!("VS Code not found, opened in default browser");
        Ok(())
    }
}

/// Opens a DOT diagram in VS Code's Simple Browser using a DOT viewer.

fn open_dot_vscode_browser(dot_src: &str) -> Result<()> {
    use std::process::Command;

    let encoded = data_encoding::BASE64.encode(dot_src.as_bytes());
    let url = format!("https://dreampuf.github.io/GraphvizOnline/#{}", encoded);

    // Try to open in VS Code Simple Browser first
    if Command::new("code")
        .arg("--command")
        .arg("simpleBrowser.show")
        .arg(&url)
        .status()
        .is_ok()
    {
        println!("Opened DOT diagram in VS Code Simple Browser");
        Ok(())
    } else {
        // Fall back to regular browser
        webbrowser::open(&url)?;
        println!("VS Code not found, opened in default browser");
        Ok(())
    }
}

/// Helper function to create a complete HTML file with ReactFlow.js visualization and open it in browser.

fn save_and_open_reactflow_browser(reactflow_json: &str, filename: &str) -> Result<()> {
    let template = get_template();
    let html_content = template.replace("{{GRAPH_DATA}}", reactflow_json);

    std::fs::write(filename, html_content)?;
    println!("Saved Enhanced ReactFlow.js visualization to {}", filename);

    // Open the HTML file in VS Code editor and in browser
    use std::process::Command;

    let file_path = std::env::current_dir()?.join(filename);
    let file_url = format!("file://{}", file_path.display());

    // Open the file in VS Code editor
    let _ = Command::new("code").arg(&file_path).status();

    // Also open in default browser for immediate viewing
    webbrowser::open(&file_url)?;

    println!("Opened Enhanced ReactFlow.js visualization:");
    println!("  • File opened in VS Code editor: {}", filename);
    println!("  • Visualization opened in default browser");
    println!("  • Enhanced with: directed edges, ColorBrewer palettes, CSS gradients");
    println!(
        "  • To view in VS Code Simple Browser: Cmd+Shift+P → 'Simple Browser: Show' → paste: {}",
        file_url
    );
    Ok(())
}

/// Helper function to render multiple Hydro IR leaves as ReactFlow.js JSON.

fn render_hydro_ir_reactflow(leaves: &[HydroLeaf], config: &HydroWriteConfig) -> String {
    super::render::render_hydro_ir_reactflow(leaves, config)
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::parse_quote;

    use super::*;
    use crate::ir::*;

    #[test]
    fn test_mermaid_generation() {
        // Create a simple Hydro IR graph for testing
        let leaf = HydroLeaf::ForEach {
            f: DebugExpr::from(parse_quote!(|x: i32| println!("{}", x))),
            input: Box::new(HydroNode::Map {
                f: DebugExpr::from(parse_quote!(|x| x * 2)),
                input: Box::new(HydroNode::Source {
                    source: HydroSource::Iter(DebugExpr::from(parse_quote!(
                        vec![1, 2, 3].into_iter()
                    ))),
                    location_kind: crate::location::LocationId::Process(0),
                    metadata: HydroIrMetadata {
                        location_kind: crate::location::LocationId::Process(0),
                        output_type: None,
                        cardinality: None,
                        cpu_usage: None,
                    },
                }),
                metadata: HydroIrMetadata {
                    location_kind: crate::location::LocationId::Process(0),
                    output_type: None,
                    cardinality: None,
                    cpu_usage: None,
                },
            }),
            metadata: HydroIrMetadata {
                location_kind: crate::location::LocationId::Process(0),
                output_type: None,
                cardinality: None,
                cpu_usage: None,
            },
        };

        let config = HydroWriteConfig::default();
        let mermaid = leaf.to_mermaid(&config);

        // Basic checks that the output contains expected elements
        assert!(mermaid.contains("flowchart TD"));
        assert!(mermaid.contains("source_iter"));
        assert!(mermaid.contains("map"));
        assert!(mermaid.contains("for_each"));
    }

    #[test]
    fn test_dot_generation() {
        // Create a simple join example
        let leaf = HydroLeaf::DestSink {
            sink: DebugExpr::from(parse_quote!(output_sink)),
            input: Box::new(HydroNode::Join {
                left: Box::new(HydroNode::Source {
                    source: HydroSource::Stream(DebugExpr::from(parse_quote!(left_stream))),
                    location_kind: crate::location::LocationId::Process(0),
                    metadata: HydroIrMetadata {
                        location_kind: crate::location::LocationId::Process(0),
                        output_type: None,
                        cardinality: None,
                        cpu_usage: None,
                    },
                }),
                right: Box::new(HydroNode::Source {
                    source: HydroSource::Stream(DebugExpr::from(parse_quote!(right_stream))),
                    location_kind: crate::location::LocationId::Process(0),
                    metadata: HydroIrMetadata {
                        location_kind: crate::location::LocationId::Process(0),
                        output_type: None,
                        cardinality: None,
                        cpu_usage: None,
                    },
                }),
                metadata: HydroIrMetadata {
                    location_kind: crate::location::LocationId::Process(0),
                    output_type: None,
                    cardinality: None,
                    cpu_usage: None,
                },
            }),
            metadata: HydroIrMetadata {
                location_kind: crate::location::LocationId::Process(0),
                output_type: None,
                cardinality: None,
                cpu_usage: None,
            },
        };

        let config = HydroWriteConfig::default();
        let dot = leaf.to_dot(&config);

        // Basic checks that the output contains expected elements
        assert!(dot.contains("digraph HydroIR"));
        assert!(dot.contains("source_stream"));
        assert!(dot.contains("join"));
        assert!(dot.contains("dest_sink"));
    }
}
