//! Debugging utilities for Hydro IR graph visualization.
//!
//! Similar to the DFIR debugging utilities, this module provides convenient
//! methods for opening graphs in web browsers and VS Code.

use std::fmt::Write;
use std::io::Result;

use super::graph_render::{HydroWriteConfig, render_hydro_ir_dot, render_hydro_ir_mermaid};
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
    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hydro IR Graph - ReactFlow.js</title>
    <script crossorigin src="https://unpkg.com/react@17/umd/react.development.js"></script>
    <script crossorigin src="https://unpkg.com/react-dom@17/umd/react-dom.development.js"></script>
    <script src="https://unpkg.com/@babel/standalone/babel.min.js"></script>
    <script src="https://unpkg.com/reactflow@11.11.0/dist/umd/index.js"></script>
    <script src="https://unpkg.com/elkjs@0.8.2/lib/elk.bundled.js"></script>
    <link rel="stylesheet" href="https://unpkg.com/reactflow@11.11.0/dist/style.css" />
    <style>
        body {{
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
                'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
                sans-serif;
            -webkit-font-smoothing: antialiased;
            -moz-osx-font-smoothing: grayscale;
        }}
        .reactflow-wrapper {{
            width: 100vw;
            height: 100vh;
        }}
        /* Compact unified legend in upper right */
        .unified-legend {{
            position: absolute;
            top: 60px;
            right: 10px;
            z-index: 10;
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            padding: 8px;
            border-radius: 6px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.15);
            max-width: 220px;
            font-size: 11px;
        }}
        /* Layout controls above legend */
        .layout-controls {{
            position: absolute;
            top: 10px;
            right: 10px;
            z-index: 10;
            background: rgba(255, 255, 255, 0.95);
            backdrop-filter: blur(10px);
            border-radius: 6px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.15);
            padding: 6px;
            display: flex;
            align-items: center;
            gap: 4px;
        }}
        .unified-legend h4 {{
            margin: 0 0 6px 0;
            font-size: 12px;
            font-weight: 600;
            color: #333;
            border-bottom: 1px solid #eee;
            padding-bottom: 3px;
        }}
        .legend-section {{
            margin-bottom: 8px;
        }}
        .legend-section:last-child {{
            margin-bottom: 0;
        }}
        .legend-item {{
            display: flex;
            align-items: center;
            margin: 3px 0;
            font-size: 10px;
        }}
        .legend-color {{
            width: 12px;
            height: 12px;
            border-radius: 2px;
            margin-right: 6px;
            border: 1px solid #666;
            flex-shrink: 0;
        }}
        .location-legend-color {{
            width: 16px;
            height: 10px;
            border-radius: 2px;
            margin-right: 6px;
            border: 1px solid;
            flex-shrink: 0;
        }}
        .icon-button {{
            width: 28px;
            height: 28px;
            border: none;
            border-radius: 4px;
            background: #f8f9fa;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 14px;
            color: #495057;
            transition: all 0.2s ease;
            position: relative;
        }}
        .icon-button:hover {{
            background: #e9ecef;
            color: #212529;
            transform: translateY(-1px);
        }}
        .icon-button:active {{
            transform: translateY(0);
        }}
        .layout-select {{
            background: #f8f9fa;
            border: 1px solid #dee2e6;
            border-radius: 4px;
            font-size: 11px;
            padding: 4px 6px;
            width: 80px;
            color: #495057;
        }}
        .layout-select:hover {{
            border-color: #adb5bd;
        }}
        /* Tooltip styles */
        .tooltip {{
            position: absolute;
            bottom: 100%;
            left: 50%;
            transform: translateX(-50%);
            background: #333;
            color: white;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 10px;
            white-space: nowrap;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s ease;
            margin-bottom: 4px;
        }}
        .tooltip::after {{
            content: '';
            position: absolute;
            top: 100%;
            left: 50%;
            transform: translateX(-50%);
            border: 4px solid transparent;
            border-top-color: #333;
        }}
        .icon-button:hover .tooltip {{
            opacity: 1;
        }}
        .react-flow__node {{
            cursor: pointer;
        }}
        .react-flow__node:hover {{
            transform: scale(1.02);
            transition: transform 0.2s ease;
        }}
    </style>
</head>
<body>
    <div id="root"></div>
    <script type="text/babel">
        const {{ useState, useCallback, useRef, useEffect }} = React;
        const ReactFlowLib = window.ReactFlow;
        const {{ ReactFlow, Controls, MiniMap, Background, useNodesState, useEdgesState, addEdge, applyNodeChanges, applyEdgeChanges }} = ReactFlowLib;
        
        const graphData = {reactflow_json};
        
        const initialNodes = graphData.nodes || [];
        const initialEdges = (graphData.edges || []).map(edge => ({{
            ...edge,
            zIndex: 1000, // Ensure edges render above containers
            style: {{
                ...edge.style,
                strokeWidth: 2,
                stroke: '#666666'
            }},
            interactionWidth: 20 // Make edges easier to interact with
        }}));

        // elk.js layout configuration with hierarchical support
        const elkLayouts = {{
            layered: {{
                'elk.algorithm': 'layered',
                'elk.layered.spacing.nodeNodeBetweenLayers': 100,
                'elk.spacing.nodeNode': 80,
                'elk.direction': 'RIGHT',
                'elk.layered.thoroughness': 7,
                'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
            }},
            force: {{
                'elk.algorithm': 'force',
                'elk.force.repulsivePower': 0.5,
                'elk.spacing.nodeNode': 100,
                'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
            }},
            stress: {{
                'elk.algorithm': 'stress',
                'elk.stress.desiredEdgeLength': 100,
                'elk.spacing.nodeNode': 80,
                'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
            }},
            mrtree: {{
                'elk.algorithm': 'mrtree',
                'elk.mrtree.searchOrder': 'DFS',
                'elk.spacing.nodeNode': 80,
                'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
            }},
            radial: {{
                'elk.algorithm': 'radial',
                'elk.radial.radius': 200,
                'elk.spacing.nodeNode': 80,
                'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
            }},
            disco: {{
                'elk.algorithm': 'disco',
                'elk.disco.componentCompaction.strategy': 'POLYOMINO',
                'elk.spacing.nodeNode': 80,
                'elk.hierarchyHandling': 'INCLUDE_CHILDREN'
            }}
        }};

        // Generate unique colors for each location ID
        const generateLocationColor = (locationId, totalLocations) => {{
            const hue = (locationId * 137.508) % 360; // Golden angle for good distribution
            const saturation = 40 + (locationId * 13) % 30; // 40-70% saturation
            const lightness = 85 + (locationId * 7) % 10; // 85-95% lightness
            return `hsl(${{hue}}, ${{saturation}}%, ${{lightness}}%)`;
        }};

        const generateLocationBorderColor = (locationId, totalLocations) => {{
            const hue = (locationId * 137.508) % 360;
            const saturation = 60 + (locationId * 13) % 30; // 60-90% saturation
            const lightness = 50 + (locationId * 7) % 20; // 50-70% lightness
            return `hsl(${{hue}}, ${{saturation}}%, ${{lightness}}%)`;
        }};

        const elk = new ELK();

        // Function to apply ELK layout with hierarchical grouping
        const applyElkLayout = async (nodes, edges, layoutType = 'layered') => {{
            const elkOptions = elkLayouts[layoutType] || elkLayouts.layered;
            
            console.log('Applying ELK layout:', layoutType, 'with options:', elkOptions);
            console.log('Input nodes:', nodes.length, 'edges:', edges.length);
            
            // Group nodes by location first
            const locationGroups = new Map();
            const orphanNodes = [];
            
            nodes.forEach(node => {{
                const nodeLocationId = node.data?.locationId;
                if (nodeLocationId !== null && nodeLocationId !== undefined) {{
                    if (!locationGroups.has(nodeLocationId)) {{
                        locationGroups.set(nodeLocationId, []);
                    }}
                    locationGroups.get(nodeLocationId).push(node);
                }} else {{
                    orphanNodes.push(node);
                }}
            }});
            
            console.log('Found', locationGroups.size, 'location groups and', orphanNodes.length, 'orphan nodes');
            
            // Create hierarchical ELK structure with proper container spacing
            const elkChildren = [];
            let containerIndex = 0;
            
            // Process each location as a separate container
            for (const [locationId, locationNodes] of locationGroups) {{
                const location = graphData.locations?.find(loc => loc.id.toString() === locationId.toString());
                
                const elkNodes = locationNodes.map(node => ({{
                    id: node.id,
                    width: node.style?.width || 200,
                    height: node.style?.height || 60,
                }}));

                const elkEdgesInLocation = edges.filter(edge => {{
                    const sourceInLocation = locationNodes.some(n => n.id === edge.source);
                    const targetInLocation = locationNodes.some(n => n.id === edge.target);
                    return sourceInLocation && targetInLocation;
                }}).map(edge => ({{
                    id: edge.id,
                    sources: [edge.source],
                    targets: [edge.target],
                }}));
                
                console.log(`Location ${{locationId}} has ${{elkEdgesInLocation.length}} internal edges`);

                elkChildren.push({{
                    id: `container_${{locationId}}`,
                    width: 400,
                    height: 300,
                    layoutOptions: {{
                        ...elkOptions,
                        'elk.padding': '[top=40,left=20,bottom=20,right=20]',
                        'elk.spacing.nodeNode': 60,
                    }},
                    children: elkNodes,
                    edges: elkEdgesInLocation,
                }});
                
                containerIndex++;
            }}
            
            // Add orphan nodes as top-level nodes
            orphanNodes.forEach(node => {{
                elkChildren.push({{
                    id: node.id,
                    width: node.style?.width || 200,
                    height: node.style?.height || 60,
                }});
            }});

            // Cross-container edges
            const crossContainerEdges = edges.filter(edge => {{
                const sourceLocation = nodes.find(n => n.id === edge.source)?.data?.locationId;
                const targetLocation = nodes.find(n => n.id === edge.target)?.data?.locationId;
                return sourceLocation !== targetLocation;
            }}).map(edge => ({{
                id: edge.id,
                sources: [edge.source],
                targets: [edge.target],
            }}));

            const elkGraph = {{
                id: 'root',
                layoutOptions: {{
                    ...elkOptions,
                    'elk.spacing.nodeNode': 150, // More space between containers
                    'elk.spacing.componentComponent': 100,
                    'elk.layered.spacing.nodeNodeBetweenLayers': 150,
                }},
                children: elkChildren,
                edges: crossContainerEdges,
            }};

            try {{
                console.log('Running ELK layout with hierarchical structure...');
                const layoutedGraph = await elk.layout(elkGraph);
                console.log('ELK layout complete:', layoutedGraph);
                
                // Apply positions from ELK layout
                const layoutedNodes = nodes.map((node) => {{
                    // Find the node in the layout result
                    let elkNode = null;
                    let containerOffset = {{ x: 0, y: 0 }};
                    
                    // Look for the node in containers first
                    for (const container of layoutedGraph.children || []) {{
                        if (container.children) {{
                            const foundNode = container.children.find(n => n.id === node.id);
                            if (foundNode) {{
                                elkNode = foundNode;
                                containerOffset = {{ x: container.x || 0, y: container.y || 0 }};
                                break;
                            }}
                        }}
                    }}
                    
                    // If not found in containers, look at top level
                    if (!elkNode) {{
                        elkNode = layoutedGraph.children?.find(n => n.id === node.id);
                    }}
                    
                    const newNode = {{
                        ...node,
                        position: {{
                            x: elkNode ? (elkNode.x || 0) + containerOffset.x : Math.random() * 500,
                            y: elkNode ? (elkNode.y || 0) + containerOffset.y : Math.random() * 500,
                        }},
                    }};
                    return newNode;
                }});

                console.log('Layouted nodes:', layoutedNodes.length);
                return layoutedNodes;
            }} catch (error) {{
                console.error('ELK layout failed:', error);
                // Fallback to simple grid layout with better spacing for containers
                console.log('Using fallback grid layout');
                const locationGroups = new Map();
                const orphanNodes = [];
                
                nodes.forEach(node => {{
                    const nodeLocationId = node.data?.locationId;
                    if (nodeLocationId !== null && nodeLocationId !== undefined) {{
                        if (!locationGroups.has(nodeLocationId)) {{
                            locationGroups.set(nodeLocationId, []);
                        }}
                        locationGroups.get(nodeLocationId).push(node);
                    }} else {{
                        orphanNodes.push(node);
                    }}
                }});
                
                let currentX = 0;
                let currentY = 0;
                const containerWidth = 500;
                const containerHeight = 400;
                const containerSpacing = 100;
                const containersPerRow = 3;
                
                const result = [];
                let containerIndex = 0;
                
                // Layout location groups
                for (const [locationId, locationNodes] of locationGroups) {{
                    const containerX = currentX;
                    const containerY = currentY;
                    
                    // Layout nodes within container
                    locationNodes.forEach((node, nodeIndex) => {{
                        const nodesPerRow = Math.ceil(Math.sqrt(locationNodes.length));
                        const nodeX = containerX + 50 + (nodeIndex % nodesPerRow) * 220;
                        const nodeY = containerY + 80 + Math.floor(nodeIndex / nodesPerRow) * 80;
                        
                        result.push({{
                            ...node,
                            position: {{ x: nodeX, y: nodeY }}
                        }});
                    }});
                    
                    // Move to next container position
                    containerIndex++;
                    if (containerIndex % containersPerRow === 0) {{
                        currentX = 0;
                        currentY += containerHeight + containerSpacing;
                    }} else {{
                        currentX += containerWidth + containerSpacing;
                    }}
                }}
                
                // Layout orphan nodes
                orphanNodes.forEach((node, index) => {{
                    result.push({{
                        ...node,
                        position: {{ 
                            x: currentX + (index % 5) * 250, 
                            y: currentY + Math.floor(index / 5) * 100 
                        }}
                    }});
                }});
                
                return result;
            }}
        }};
        
        function HydroGraph() {{
            const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
            const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
            const [currentLayout, setCurrentLayout] = React.useState('layered');
            
            const onConnect = useCallback((params) => setEdges((eds) => addEdge(params, eds)), [setEdges]);
            
            const fitView = useCallback(() => {{
                const reactFlowInstance = window.reactFlowInstance;
                if (reactFlowInstance) {{
                    reactFlowInstance.fitView();
                }}
            }}, []);
            
            const onInit = useCallback(async (reactFlowInstance) => {{
                window.reactFlowInstance = reactFlowInstance;
                
                console.log('Initializing graph with', initialNodes.length, 'nodes');
                
                // Apply ELK layout to get proper positions
                const layoutedNodes = await applyElkLayout(initialNodes, initialEdges, currentLayout);
                
                // Group nodes by location for hierarchical display
                const locationContainers = [];
                const childNodes = [];
                
                if (graphData.locations && graphData.locations.length > 0) {{
                    console.log('Creating location containers for', graphData.locations.length, 'locations');
                    
                    graphData.locations.forEach(location => {{
                        // Find nodes in this location
                        const locationNodes = layoutedNodes.filter(node => {{
                            const nodeLocationId = node.data?.locationId;
                            return nodeLocationId !== null && 
                                   nodeLocationId !== undefined && 
                                   nodeLocationId.toString() === location.id.toString();
                        }});
                        
                        console.log(`Location ${{location.id}} (${{location.label}}) has ${{locationNodes.length}} nodes`);
                        
                        if (locationNodes.length > 0) {{
                            // Calculate bounding box for this location
                            const minX = Math.min(...locationNodes.map(n => n.position.x));
                            const minY = Math.min(...locationNodes.map(n => n.position.y));
                            const maxX = Math.max(...locationNodes.map(n => n.position.x + 200));
                            const maxY = Math.max(...locationNodes.map(n => n.position.y + 60));
                            
                            // Add padding
                            const padding = 30;
                            const containerX = minX - padding;
                            const containerY = minY - padding - 30;
                            
                            // Generate unique colors for this location
                            const backgroundColor = generateLocationColor(location.id, graphData.locations.length);
                            const borderColor = generateLocationBorderColor(location.id, graphData.locations.length);
                            
                            // Create parent container
                            locationContainers.push({{
                                id: `container_${{location.id}}`,
                                type: 'group',
                                position: {{ x: containerX, y: containerY }},
                                style: {{
                                    width: maxX - minX + 2 * padding,
                                    height: maxY - minY + 2 * padding + 30,
                                    backgroundColor: backgroundColor,
                                    border: `2px solid ${{borderColor}}`,
                                    borderRadius: '8px',
                                }},
                                data: {{ 
                                    label: location.label,
                                    isContainer: true
                                }},
                                draggable: true,
                            }});
                            
                            // Make nodes children of the container
                            locationNodes.forEach(node => {{
                                childNodes.push({{
                                    ...node,
                                    parentNode: `container_${{location.id}}`,
                                    extent: 'parent',
                                    position: {{
                                        x: node.position.x - containerX,
                                        y: node.position.y - containerY
                                    }}
                                }});
                            }});
                        }}
                    }});
                    
                    // Handle orphan nodes (not in any location) - group them into a grey container
                    const orphanNodes = layoutedNodes.filter(node => {{
                        const nodeLocationId = node.data?.locationId;
                        if (nodeLocationId === null || nodeLocationId === undefined) return true;
                        
                        return !graphData.locations.some(loc => 
                            loc.id.toString() === nodeLocationId.toString()
                        );
                    }});
                    
                    console.log('Found', orphanNodes.length, 'orphan nodes');
                    
                    if (orphanNodes.length > 0) {{
                        // Calculate bounding box for orphan nodes
                        const minX = Math.min(...orphanNodes.map(n => n.position.x));
                        const minY = Math.min(...orphanNodes.map(n => n.position.y));
                        const maxX = Math.max(...orphanNodes.map(n => n.position.x + 200));
                        const maxY = Math.max(...orphanNodes.map(n => n.position.y + 60));
                        
                        // Add padding
                        const padding = 30;
                        const containerX = minX - padding;
                        const containerY = minY - padding - 30;
                        
                        // Create grey container for orphan nodes
                        locationContainers.push({{
                            id: 'container_null',
                            type: 'group',
                            position: {{ x: containerX, y: containerY }},
                            style: {{
                                width: maxX - minX + 2 * padding,
                                height: maxY - minY + 2 * padding + 30,
                                backgroundColor: 'rgba(200, 200, 200, 0.2)',
                                border: '2px solid #999999',
                                borderRadius: '8px',
                            }},
                            data: {{ 
                                label: 'Internal/Unassigned',
                                isContainer: true
                            }},
                            draggable: true,
                        }});
                        
                        // Make orphan nodes children of the grey container
                        orphanNodes.forEach(node => {{
                            childNodes.push({{
                                ...node,
                                parentNode: 'container_null',
                                extent: 'parent',
                                position: {{
                                    x: node.position.x - containerX,
                                    y: node.position.y - containerY
                                }}
                            }});
                        }});
                    }}
                }} else {{
                    // No locations defined, use all nodes as-is
                    console.log('No locations defined, using flat layout');
                    childNodes.push(...layoutedNodes);
                }}
                
                // Combine containers and child nodes
                const allElements = [...locationContainers, ...childNodes];
                console.log('Setting', allElements.length, 'total elements:', {{
                    containers: locationContainers.length,
                    nodes: childNodes.length
                }});
                setNodes(allElements);
                
                // Ensure all edges are properly set
                console.log('Setting', initialEdges.length, 'edges');
                setEdges(initialEdges);
                
                // Fit view after layout is applied
                setTimeout(() => {{
                    console.log('Fitting view...');
                    reactFlowInstance.fitView({{ padding: 0.1 }});
                }}, 200);
            }}, [setNodes, currentLayout]);

            // Apply elk layout with selected algorithm
            const applyLayout = useCallback(async (layoutType = currentLayout) => {{
                console.log('Applying layout:', layoutType);
                
                // Only layout the actual graph nodes, not containers
                const actualNodes = nodes.filter(node => !node.data?.isContainer);
                console.log('Laying out', actualNodes.length, 'actual nodes');
                
                if (actualNodes.length === 0) {{
                    console.log('No nodes to layout');
                    return;
                }}
                
                const layoutedNodes = await applyElkLayout(actualNodes, edges, layoutType);
                
                // Create new containers and child relationships
                const locationContainers = [];
                const childNodes = [];
                
                if (graphData.locations && graphData.locations.length > 0) {{
                    graphData.locations.forEach(location => {{
                        const locationNodes = layoutedNodes.filter(node => {{
                            const nodeLocationId = node.data?.locationId;
                            return nodeLocationId !== null && 
                                   nodeLocationId !== undefined && 
                                   nodeLocationId.toString() === location.id.toString();
                        }});
                        
                        if (locationNodes.length > 0) {{
                            // Calculate bounding box
                            const minX = Math.min(...locationNodes.map(n => n.position.x));
                            const minY = Math.min(...locationNodes.map(n => n.position.y));
                            const maxX = Math.max(...locationNodes.map(n => n.position.x + 200));
                            const maxY = Math.max(...locationNodes.map(n => n.position.y + 60));
                            
                            const padding = 30;
                            const containerX = minX - padding;
                            const containerY = minY - padding - 30;
                            
                            // Generate unique colors for this location
                            const backgroundColor = generateLocationColor(location.id, graphData.locations.length);
                            const borderColor = generateLocationBorderColor(location.id, graphData.locations.length);
                            
                            // Create container
                            locationContainers.push({{
                                id: `container_${{location.id}}`,
                                type: 'group',
                                position: {{ x: containerX, y: containerY }},
                                style: {{
                                    width: maxX - minX + 2 * padding,
                                    height: maxY - minY + 2 * padding + 30,
                                    backgroundColor: backgroundColor,
                                    border: `2px solid ${{borderColor}}`,
                                    borderRadius: '8px',
                                }},
                                data: {{ 
                                    label: location.label,
                                    isContainer: true
                                }},
                                draggable: true,
                            }});
                            
                            // Add child nodes
                            locationNodes.forEach(node => {{
                                childNodes.push({{
                                    ...node,
                                    parentNode: `container_${{location.id}}`,
                                    extent: 'parent',
                                    position: {{
                                        x: node.position.x - containerX,
                                        y: node.position.y - containerY
                                    }}
                                }});
                            }});
                        }}
                    }});
                    
                    // Handle orphan nodes - group them into a grey container
                    const orphanNodes = layoutedNodes.filter(node => {{
                        const nodeLocationId = node.data?.locationId;
                        if (nodeLocationId === null || nodeLocationId === undefined) return true;
                        return !graphData.locations.some(loc => 
                            loc.id.toString() === nodeLocationId.toString()
                        );
                    }});
                    
                    if (orphanNodes.length > 0) {{
                        // Calculate bounding box for orphan nodes
                        const minX = Math.min(...orphanNodes.map(n => n.position.x));
                        const minY = Math.min(...orphanNodes.map(n => n.position.y));
                        const maxX = Math.max(...orphanNodes.map(n => n.position.x + 200));
                        const maxY = Math.max(...orphanNodes.map(n => n.position.y + 60));
                        
                        const padding = 30;
                        const containerX = minX - padding;
                        const containerY = minY - padding - 30;
                        
                        // Create grey container for orphan nodes
                        locationContainers.push({{
                            id: 'container_null',
                            type: 'group',
                            position: {{ x: containerX, y: containerY }},
                            style: {{
                                width: maxX - minX + 2 * padding,
                                height: maxY - minY + 2 * padding + 30,
                                backgroundColor: 'rgba(200, 200, 200, 0.2)',
                                border: '2px solid #999999',
                                borderRadius: '8px',
                            }},
                            data: {{ 
                                label: 'Internal/Unassigned',
                                isContainer: true
                            }},
                            draggable: true,
                        }});
                        
                        // Make orphan nodes children of the grey container
                        orphanNodes.forEach(node => {{
                            childNodes.push({{
                                ...node,
                                parentNode: 'container_null',
                                extent: 'parent',
                                position: {{
                                    x: node.position.x - containerX,
                                    y: node.position.y - containerY
                                }}
                            }});
                        }});
                    }}
                }} else {{
                    childNodes.push(...layoutedNodes);
                }}
                
                const allElements = [...locationContainers, ...childNodes];
                console.log('Updating to', allElements.length, 'elements');
                setNodes(allElements);
                
                // Ensure all edges are properly maintained
                console.log('Maintaining', initialEdges.length, 'edges');
                setEdges(initialEdges);
                
                setTimeout(() => {{
                    if (window.reactFlowInstance) {{
                        window.reactFlowInstance.fitView({{ padding: 0.1 }});
                    }}
                }}, 200);
            }}, [nodes, edges, setNodes, currentLayout]);

            // Handle layout change
            const handleLayoutChange = useCallback(async (event) => {{
                const newLayout = event.target.value;
                setCurrentLayout(newLayout);
                await applyLayout(newLayout);
            }}, [applyLayout]);

            // Handle node clicks to toggle expansion
            const onNodeClick = useCallback((event, node) => {{
                setNodes((nodes) => 
                    nodes.map((n) => {{
                        if (n.id === node.id) {{
                            const expanded = !n.data.expanded;
                            const label = expanded ? n.data.fullLabel : n.data.shortLabel;
                            return {{
                                ...n,
                                data: {{
                                    ...n.data,
                                    label: label,
                                    expanded: expanded
                                }}
                            }};
                        }}
                        return n;
                    }})
                );
            }}, [setNodes]);

            // Toggle all nodes
            const toggleAllNodes = useCallback(() => {{
                setNodes((nodes) => {{
                    const allExpanded = nodes.every(n => n.data.expanded);
                    return nodes.map((n) => ({{
                        ...n,
                        data: {{
                            ...n.data,
                            label: allExpanded ? n.data.shortLabel : n.data.fullLabel,
                            expanded: !allExpanded
                        }}
                    }}));
                }});
            }}, [setNodes]);
            
            return (
                <div className="reactflow-wrapper">
                    {{/* Layout controls above legend in upper right */}}
                    <div className="layout-controls">
                        <select 
                            className="layout-select"
                            value={{currentLayout}} 
                            onChange={{handleLayoutChange}}
                            title="Layout Algorithm"
                        >
                            <option value="layered">Layered</option>
                            <option value="force">Force</option>
                            <option value="stress">Stress</option>
                            <option value="mrtree">Tree</option>
                            <option value="radial">Radial</option>
                            <option value="disco">Disco</option>
                        </select>
                        <button className="icon-button" onClick={{() => applyLayout()}} title="Re-apply Layout">
                            🔄
                            <div className="tooltip">Re-apply Layout</div>
                        </button>
                        <button className="icon-button" onClick={{toggleAllNodes}} title="Toggle Details">
                            📝
                            <div className="tooltip">Toggle Details</div>
                        </button>
                    </div>

                    {{/* Unified compact legend in upper right */}}
                    <div className="unified-legend">
                        <div className="legend-section">
                            <h4>Node Types</h4>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#88ff88'}}}}></div>
                                <span>Source</span>
                            </div>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#8888ff'}}}}></div>
                                <span>Transform</span>
                            </div>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#ff8888'}}}}></div>
                                <span>Join</span>
                            </div>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#ffff88'}}}}></div>
                                <span>Aggregation</span>
                            </div>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#88ffff'}}}}></div>
                                <span>Network</span>
                            </div>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#ff88ff'}}}}></div>
                                <span>Sink</span>
                            </div>
                            <div className="legend-item">
                                <div className="legend-color" style={{{{backgroundColor: '#dddddd'}}}}></div>
                                <span>Tee</span>
                            </div>
                        </div>
                        <div className="legend-section">
                            <h4>Locations</h4>
                            {{graphData.locations && graphData.locations.map((location, index) => (
                                <div key={{location.id}} className="legend-item">
                                    <div 
                                        className="location-legend-color" 
                                        style={{{{
                                            backgroundColor: generateLocationColor(location.id, graphData.locations.length),
                                            borderColor: generateLocationBorderColor(location.id, graphData.locations.length),
                                            border: '1px solid'
                                        }}}}
                                    ></div>
                                    <span>{{location.label}}</span>
                                </div>
                            ))}}
                            <div className="legend-item">
                                <div 
                                    className="location-legend-color" 
                                    style={{{{
                                        backgroundColor: 'rgba(200, 200, 200, 0.2)',
                                        borderColor: '#999999',
                                        border: '1px solid'
                                    }}}}
                                ></div>
                                <span>Internal/Unassigned</span>
                            </div>
                        </div>
                    </div>

                    <ReactFlow
                        nodes={{nodes}}
                        edges={{edges}}
                        onNodesChange={{onNodesChange}}
                        onEdgesChange={{onEdgesChange}}
                        onConnect={{onConnect}}
                        onNodeClick={{onNodeClick}}
                        onInit={{onInit}}
                        connectionMode="loose"
                        elevateEdgesOnSelect={{true}}
                        edgesReconnectable={{false}}
                        fitView
                    >
                        <Controls />
                        <MiniMap />
                        <Background />
                    </ReactFlow>
                </div>
            );
        }}
        
        ReactDOM.render(<HydroGraph />, document.getElementById('root'));
    </script>
</body>
</html>"#,
        reactflow_json = reactflow_json
    );

    std::fs::write(filename, html_content)?;
    println!("Saved ReactFlow.js visualization to {}", filename);

    // Open the HTML file in VS Code editor and in browser
    use std::process::Command;

    let file_path = std::env::current_dir()?.join(filename);
    let file_url = format!("file://{}", file_path.display());

    // Open the file in VS Code editor
    let _ = Command::new("code").arg(&file_path).status();

    // Also open in default browser for immediate viewing
    webbrowser::open(&file_url)?;

    println!("Opened ReactFlow.js visualization:");
    println!("  • File opened in VS Code editor: {}", filename);
    println!("  • Visualization opened in default browser");
    println!(
        "  • To view in VS Code Simple Browser: Cmd+Shift+P → 'Simple Browser: Show' → paste: {}",
        file_url
    );
    Ok(())
}

/// Helper function to render multiple Hydro IR leaves as ReactFlow.js JSON.

fn render_hydro_ir_reactflow(leaves: &[HydroLeaf], config: &HydroWriteConfig) -> String {
    super::graph_render::render_hydro_ir_reactflow(leaves, config)
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
