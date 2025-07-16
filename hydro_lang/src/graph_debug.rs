//! Debugging utilities for Hydro IR graph visualization.
//! 
//! Similar to the DFIR debugging utilities, this module provides convenient
//! methods for opening graphs in web browsers and VS Code.

use std::fmt::Write;
use std::io::Result;

use super::graph_render::{HydroWriteConfig, render_hydro_ir_mermaid, render_hydro_ir_dot};
use crate::ir::HydroLeaf;

/// Debugging extensions for Hydro IR.
impl HydroLeaf {
    /// Opens this Hydro IR graph as a mermaid diagram in the [mermaid.live](https://mermaid.live) browser editor.
    #[cfg(feature = "debugging")]
    pub fn open_mermaid(&self, config: Option<HydroWriteConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        let mermaid_src = self.to_mermaid(&config);
        open_mermaid_browser(&mermaid_src)
    }

    /// Opens this Hydro IR graph as a DOT/Graphviz diagram in the browser.
    #[cfg(feature = "debugging")]
    pub fn open_dot(&self, config: Option<HydroWriteConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        let dot_src = self.to_dot(&config);
        open_dot_browser(&dot_src)
    }

    /// Saves this Hydro IR graph as a .mermaid file and opens it in VS Code for preview.
    /// Requires the "Mermaid Preview" extension in VS Code.
    #[cfg(feature = "debugging")]
    pub fn open_mermaid_vscode(&self, filename: Option<&str>, config: Option<HydroWriteConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        let mermaid_src = self.to_mermaid(&config);
        let filename = filename.unwrap_or("hydro_graph.mermaid");
        save_and_open_vscode(&mermaid_src, filename)
    }

    /// Saves this Hydro IR graph as a .dot file and opens it in VS Code for preview.
    /// Requires a Graphviz extension in VS Code.
    #[cfg(feature = "debugging")]
    pub fn open_dot_vscode(&self, filename: Option<&str>, config: Option<HydroWriteConfig>) -> Result<()> {
        let config = config.unwrap_or_default();
        let dot_src = self.to_dot(&config);
        let filename = filename.unwrap_or("hydro_graph.dot");
        save_and_open_vscode(&dot_src, filename)
    }
}

/// Opens multiple Hydro IR leaves as a single mermaid diagram.
#[cfg(feature = "debugging")]
pub fn open_hydro_ir_mermaid(leaves: &[HydroLeaf], config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let mermaid_src = render_hydro_ir_mermaid(leaves, &config);
    open_mermaid_browser(&mermaid_src)
}

/// Opens multiple Hydro IR leaves as a single DOT diagram.
#[cfg(feature = "debugging")]
pub fn open_hydro_ir_dot(leaves: &[HydroLeaf], config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let dot_src = render_hydro_ir_dot(leaves, &config);
    open_dot_browser(&dot_src)
}

/// Saves multiple Hydro IR leaves as a .mermaid file and opens it in VS Code.
#[cfg(feature = "debugging")]
pub fn open_hydro_ir_mermaid_vscode(leaves: &[HydroLeaf], filename: Option<&str>, config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let mermaid_src = render_hydro_ir_mermaid(leaves, &config);
    let filename = filename.unwrap_or("hydro_graph.mermaid");
    save_and_open_vscode(&mermaid_src, filename)
}

/// Saves multiple Hydro IR leaves as a .dot file and opens it in VS Code.
#[cfg(feature = "debugging")]
pub fn open_hydro_ir_dot_vscode(leaves: &[HydroLeaf], filename: Option<&str>, config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let dot_src = render_hydro_ir_dot(leaves, &config);
    let filename = filename.unwrap_or("hydro_graph.dot");
    save_and_open_vscode(&dot_src, filename)
}

/// Opens a Mermaid diagram in VS Code's Simple Browser using mermaid.live.
#[cfg(feature = "debugging")]
pub fn open_hydro_ir_mermaid_simple_browser(leaves: &[HydroLeaf], config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let mermaid_src = render_hydro_ir_mermaid(leaves, &config);
    open_mermaid_vscode_browser(&mermaid_src)
}

/// Opens a DOT diagram in VS Code's Simple Browser.
#[cfg(feature = "debugging")]
pub fn open_hydro_ir_dot_simple_browser(leaves: &[HydroLeaf], config: Option<HydroWriteConfig>) -> Result<()> {
    let config = config.unwrap_or_default();
    let dot_src = render_hydro_ir_dot(leaves, &config);
    open_dot_vscode_browser(&dot_src)
}

#[cfg(feature = "debugging")]
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

#[cfg(feature = "debugging")]
fn open_dot_browser(dot_src: &str) -> Result<()> {
    let mut url = "https://dreampuf.github.io/GraphvizOnline/#".to_owned();
    for byte in dot_src.bytes() {
        // Lazy percent encoding: https://en.wikipedia.org/wiki/Percent-encoding
        write!(url, "%{:02x}", byte).unwrap();
    }
    webbrowser::open(&url)
}

/// Saves content to a file and opens it in VS Code.
#[cfg(feature = "debugging")]
fn save_and_open_vscode(content: &str, filename: &str) -> Result<()> {
    use std::fs;
    use std::process::Command;

    // Save the content to a file
    fs::write(filename, content)?;
    
    // Try to open in VS Code
    if let Ok(_) = Command::new("code")
        .arg(filename)
        .status() 
    {
        println!("Opened {} in VS Code", filename);
        println!("For Mermaid files: Install 'Mermaid Preview' extension");
        println!("For DOT files: Install 'Graphviz (dot) language support' extension");
        Ok(())
    } else {
        // Fall back to system default
        webbrowser::open(&format!("file://{}", std::env::current_dir()?.join(filename).display()))?;
        println!("VS Code not found, opened {} with default application", filename);
        Ok(())
    }
}

/// Opens a Mermaid diagram in VS Code's Simple Browser using mermaid.live.
#[cfg(feature = "debugging")]
fn open_mermaid_vscode_browser(mermaid_src: &str) -> Result<()> {
    use std::process::Command;

    let state = serde_json::json!({
        "code": mermaid_src,
        "mermaid": {
            "theme": "default"
        }
    });
    let state_str = state.to_string();
    let encoded = data_encoding::BASE64URL_NOPAD.encode(state_str.as_bytes());
    let url = format!("https://mermaid.live/edit#{}", encoded);

    // Try to open in VS Code Simple Browser first
    if let Ok(_) = Command::new("code")
        .arg("--command")
        .arg("simpleBrowser.show")
        .arg(&url)
        .status()
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
#[cfg(feature = "debugging")]
fn open_dot_vscode_browser(dot_src: &str) -> Result<()> {
    use std::process::Command;

    let encoded = data_encoding::BASE64.encode(dot_src.as_bytes());
    let url = format!("https://dreampuf.github.io/GraphvizOnline/#{}", encoded);

    // Try to open in VS Code Simple Browser first
    if let Ok(_) = Command::new("code")
        .arg("--command")
        .arg("simpleBrowser.show")
        .arg(&url)
        .status()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::*;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn test_mermaid_generation() {
        // Create a simple Hydro IR graph for testing
        let leaf = HydroLeaf::ForEach {
            f: DebugExpr::from(parse_quote!(|x| println!("{}", x))),
            input: Box::new(HydroNode::Map {
                f: DebugExpr::from(parse_quote!(|x| x * 2)),
                input: Box::new(HydroNode::Source {
                    source: HydroSource::Iter(DebugExpr::from(parse_quote!(vec![1, 2, 3].into_iter()))),
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
