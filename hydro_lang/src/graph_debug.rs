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
        const initialEdges = graphData.edges || [];

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

        const elk = new ELK();

        // Function to apply ELK layout with hierarchical grouping
        const applyElkLayout = async (nodes, edges, layoutType = 'layered') => {{
            const elkOptions = elkLayouts[layoutType] || elkLayouts.layered;
            
            const locationGroups = {{}};
            
            // Group nodes by location
            nodes.forEach(node => {{
                const locationId = node.data.locationId;
                const groupKey = locationId !== null && locationId !== undefined ? locationId.toString() : 'unassigned';
                
                if (!locationGroups[groupKey]) {{
                    locationGroups[groupKey] = [];
                }}
                locationGroups[groupKey].push(node);
            }});

            // Create elk nodes (flat structure but grouped)
            const elkNodes = nodes.map(node => ({{
                id: node.id,
                width: 200,
                height: 60,
            }}));

            const elkEdges = edges.map((edge) => ({{
                id: edge.id,
                sources: [edge.source],
                targets: [edge.target],
            }}));

            const elkGraph = {{
                id: 'root',
                layoutOptions: elkOptions,
                children: elkNodes,
                edges: elkEdges,
            }};

            try {{
                const layoutedGraph = await elk.layout(elkGraph);
                
                // Apply positions from ELK layout
                const layoutedNodes = nodes.map((node) => {{
                    const elkNode = layoutedGraph.children?.find((n) => n.id === node.id);
                    return {{
                        ...node,
                        position: {{
                            x: elkNode ? elkNode.x || 0 : 0,
                            y: elkNode ? elkNode.y || 0 : 0,
                        }},
                    }};
                }});

                // Now group nodes by location for visual arrangement
                const groupedNodes = [];
                const locationOffsets = {{}};
                let currentX = 0;
                
                Object.keys(locationGroups).forEach((locationId, groupIndex) => {{
                    const groupNodes = locationGroups[locationId];
                    const nodesInGroup = layoutedNodes.filter(node => {{
                        const nodeLocationId = node.data.locationId;
                        const nodeGroupKey = nodeLocationId !== null && nodeLocationId !== undefined ? nodeLocationId.toString() : 'unassigned';
                        return nodeGroupKey === locationId;
                    }});
                    
                    // Calculate group dimensions
                    const groupWidth = Math.max(300, nodesInGroup.length * 250);
                    const groupHeight = 200;
                    
                    // Position nodes within their group
                    nodesInGroup.forEach((node, index) => {{
                        const nodeX = currentX + 50 + (index % 3) * 220;
                        const nodeY = 80 + Math.floor(index / 3) * 100;
                        
                        groupedNodes.push({{
                            ...node,
                            position: {{ x: nodeX, y: nodeY }}
                        }});
                    }});
                    
                    locationOffsets[locationId] = {{ x: currentX, y: 20, width: groupWidth, height: groupHeight }};
                    currentX += groupWidth + 50;
                }});

                return groupedNodes;
            }} catch (error) {{
                console.error('ELK layout failed:', error);
                // Fallback to simple grid layout
                return nodes.map((node, index) => ({{
                    ...node,
                    position: {{ x: (index % 3) * 250, y: Math.floor(index / 3) * 100 }}
                }}));
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
                
                // Apply ELK layout
                const layoutedNodes = await applyElkLayout(initialNodes, initialEdges, currentLayout);
                
                // Create location container elements (background rectangles)
                const locationContainers = [];
                if (graphData.locations) {{
                    graphData.locations.forEach(location => {{
                        // Find nodes in this location
                        const locationNodes = layoutedNodes.filter(node => 
                            node.data.locationId !== null && 
                            node.data.locationId !== undefined && 
                            node.data.locationId.toString() === location.id.toString()
                        );
                        
                        if (locationNodes.length > 0) {{
                            // Calculate bounding box for this location
                            const minX = Math.min(...locationNodes.map(n => n.position.x));
                            const minY = Math.min(...locationNodes.map(n => n.position.y));
                            const maxX = Math.max(...locationNodes.map(n => n.position.x + 200));
                            const maxY = Math.max(...locationNodes.map(n => n.position.y + 60));
                            
                            // Add padding
                            const padding = 30;
                            locationContainers.push({{
                                id: `container_${{location.id}}`,
                                type: 'group',
                                position: {{ 
                                    x: minX - padding, 
                                    y: minY - padding - 30 // Extra space for label
                                }},
                                style: {{
                                    width: maxX - minX + 2 * padding,
                                    height: maxY - minY + 2 * padding + 30,
                                    backgroundColor: 'rgba(200, 220, 240, 0.3)',
                                    border: '2px solid #4A90E2',
                                    borderRadius: '8px',
                                    zIndex: -1,
                                }},
                                data: {{ 
                                    label: location.label,
                                    isContainer: true
                                }},
                                selectable: false,
                                draggable: false,
                            }});
                        }}
                    }});
                }}
                
                // Combine containers and nodes
                const allElements = [...locationContainers, ...layoutedNodes];
                console.log('Location containers created:', locationContainers.length);
                console.log('Total elements:', allElements.length);
                setNodes(allElements);
                
                // Fit view after layout is applied
                setTimeout(() => {{
                    reactFlowInstance.fitView();
                }}, 100);
            }}, [setNodes, currentLayout]);

            // Apply elk layout with selected algorithm
            const applyLayout = useCallback(async (layoutType = currentLayout) => {{
                // Only layout the actual graph nodes, not containers
                const actualNodes = nodes.filter(node => !node.data.isContainer);
                const layoutedNodes = await applyElkLayout(actualNodes, edges, layoutType);
                
                // Create location container elements
                const locationContainers = [];
                if (graphData.locations) {{
                    graphData.locations.forEach(location => {{
                        // Find nodes in this location
                        const locationNodes = layoutedNodes.filter(node => 
                            node.data.locationId !== null && 
                            node.data.locationId !== undefined && 
                            node.data.locationId.toString() === location.id.toString()
                        );
                        
                        if (locationNodes.length > 0) {{
                            // Calculate bounding box for this location
                            const minX = Math.min(...locationNodes.map(n => n.position.x));
                            const minY = Math.min(...locationNodes.map(n => n.position.y));
                            const maxX = Math.max(...locationNodes.map(n => n.position.x + 200));
                            const maxY = Math.max(...locationNodes.map(n => n.position.y + 60));
                            
                            // Add padding
                            const padding = 30;
                            locationContainers.push({{
                                id: `container_${{location.id}}`,
                                type: 'group',
                                position: {{ 
                                    x: minX - padding, 
                                    y: minY - padding - 30 // Extra space for label
                                }},
                                style: {{
                                    width: maxX - minX + 2 * padding,
                                    height: maxY - minY + 2 * padding + 30,
                                    backgroundColor: 'rgba(200, 220, 240, 0.3)',
                                    border: '2px solid #4A90E2',
                                    borderRadius: '8px',
                                    zIndex: -1,
                                }},
                                data: {{ 
                                    label: location.label,
                                    isContainer: true
                                }},
                                selectable: false,
                                draggable: false,
                            }});
                        }}
                    }});
                }}
                
                // Combine containers and nodes
                const allElements = [...locationContainers, ...layoutedNodes];
                setNodes(allElements);
                setTimeout(() => {{
                    if (window.reactFlowInstance) {{
                        window.reactFlowInstance.fitView();
                    }}
                }}, 100);
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
                            <div className="legend-item">
                                <div className="location-legend-color" style={{{{backgroundColor: '#fff0e6', borderColor: '#ffccb3'}}}}></div>
                                <span>Cluster</span>
                            </div>
                            <div className="legend-item">
                                <div className="location-legend-color" style={{{{backgroundColor: '#e6fff3', borderColor: '#b3ffcc'}}}}></div>
                                <span>Process</span>
                            </div>
                            <div className="legend-item">
                                <div className="location-legend-color" style={{{{backgroundColor: '#f5f5f5', borderColor: '#cccccc'}}}}></div>
                                <span>External</span>
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
