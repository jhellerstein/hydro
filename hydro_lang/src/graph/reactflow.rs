use std::collections::HashMap;
use std::fmt::Write;

use serde_json;

use super::render::{HydroEdgeType, HydroGraphWrite, HydroNodeType};

/// ReactFlow.js graph writer for Hydro IR.
/// Outputs JSON that can be directly used with ReactFlow.js for interactive graph visualization.
pub struct HydroReactFlow<W> {
    write: W,
    nodes: Vec<serde_json::Value>,
    edges: Vec<serde_json::Value>,
    locations: HashMap<usize, (String, Vec<usize>)>, // location_id -> (label, node_ids)
    edge_count: usize,
}

impl<W> HydroReactFlow<W> {
    pub fn new(write: W) -> Self {
        Self {
            write,
            nodes: Vec::new(),
            edges: Vec::new(),
            locations: HashMap::new(),
            edge_count: 0,
        }
    }

    fn node_type_to_style(&self, node_type: HydroNodeType) -> serde_json::Value {
        // Base template for all nodes
        let base_style = serde_json::json!({
            "color": "#000000",
            "border": "2px solid #000000",
            "borderRadius": "8px",
            "padding": "8px",
            "fontSize": "12px",
            "fontFamily": "monospace"
        });

        // Only specify the background colors that differ
        let background_color = match node_type {
            HydroNodeType::Source => "#88ff88",
            HydroNodeType::Transform => "#8888ff",
            HydroNodeType::Join => "#ff8888",
            HydroNodeType::Aggregation => "#ffff88",
            HydroNodeType::Network => "#88ffff",
            HydroNodeType::Sink => "#ff88ff",
            HydroNodeType::Tee => "#dddddd",
        };

        // Merge the background color with the base template
        let mut style = base_style;
        style["backgroundColor"] = serde_json::Value::String(background_color.to_string());
        style
    }

    fn edge_type_to_style(&self, edge_type: HydroEdgeType) -> serde_json::Value {
        // Base template for all edges
        let mut style = serde_json::json!({
            "strokeWidth": 2,
            "animated": false
        });

        // Apply type-specific overrides
        match edge_type {
            HydroEdgeType::Stream => {
                style["stroke"] = serde_json::Value::String("#666666".to_string());
            }
            HydroEdgeType::Persistent => {
                style["stroke"] = serde_json::Value::String("#008800".to_string());
                style["strokeWidth"] = serde_json::Value::Number(serde_json::Number::from(3));
            }
            HydroEdgeType::Network => {
                style["stroke"] = serde_json::Value::String("#880088".to_string());
                style["strokeDasharray"] = serde_json::Value::String("5,5".to_string());
                style["animated"] = serde_json::Value::Bool(true);
            }
            HydroEdgeType::Cycle => {
                style["stroke"] = serde_json::Value::String("#ff0000".to_string());
                style["animated"] = serde_json::Value::Bool(true);
            }
        }

        style
    }

    /// Apply elk.js layout via browser - nodes start at origin for elk.js to position
    fn apply_layout(&mut self) {
        // Set all nodes to default position - elk.js will handle layout in browser
        for node in &mut self.nodes {
            node["position"]["x"] = serde_json::Value::Number(serde_json::Number::from(0));
            node["position"]["y"] = serde_json::Value::Number(serde_json::Number::from(0));
        }
    }

    /// Extract a short, readable label from the full token stream label
    fn extract_short_label(&self, full_label: &str) -> String {
        // Look for common patterns and extract just the operation name
        if let Some(op_name) = full_label.split('(').next() {
            match op_name.to_lowercase().as_str() {
                "map" => "map".to_string(),
                "filter" => "filter".to_string(),
                "flat_map" => "flat_map".to_string(),
                "filter_map" => "filter_map".to_string(),
                "for_each" => "for_each".to_string(),
                "fold" => "fold".to_string(),
                "reduce" => "reduce".to_string(),
                "join" => "join".to_string(),
                "persist" => "persist".to_string(),
                "delta" => "delta".to_string(),
                "tee" => "tee".to_string(),
                "source_iter" => "source_iter".to_string(),
                "dest_sink" => "dest_sink".to_string(),
                "cycle_sink" => "cycle_sink".to_string(),
                "external_network" => "network".to_string(),
                "spin" => "spin".to_string(),
                "inspect" => "inspect".to_string(),
                _ if full_label.contains("network") => {
                    if full_label.contains("deser") {
                        "network(recv)".to_string()
                    } else if full_label.contains("ser") {
                        "network(send)".to_string()
                    } else {
                        "network".to_string()
                    }
                }
                _ if full_label.contains("send_bincode") => "send_bincode".to_string(),
                _ if full_label.contains("broadcast_bincode") => "broadcast_bincode".to_string(),
                _ if full_label.contains("dest_sink") => "dest_sink".to_string(),
                _ if full_label.contains("source_stream") => "source_stream".to_string(),
                _ => {
                    // For other cases, try to get a reasonable short name
                    if full_label.len() > 20 {
                        format!("{}...", &full_label[..17])
                    } else {
                        full_label.to_string()
                    }
                }
            }
        } else {
            // Fallback for labels that don't follow the pattern
            if full_label.len() > 20 {
                format!("{}...", &full_label[..17])
            } else {
                full_label.to_string()
            }
        }
    }
}

impl<W> HydroGraphWrite for HydroReactFlow<W>
where
    W: Write,
{
    type Err = std::fmt::Error;

    fn write_prologue(&mut self) -> Result<(), Self::Err> {
        // Clear any existing data
        self.nodes.clear();
        self.edges.clear();
        self.locations.clear();
        self.edge_count = 0;
        Ok(())
    }

    fn write_node_definition(
        &mut self,
        node_id: usize,
        node_label: &str,
        node_type: HydroNodeType,
        location_id: Option<usize>,
        location_type: Option<&str>,
    ) -> Result<(), Self::Err> {
        let style = self.node_type_to_style(node_type);

        // Extract short label from full label
        let short_label = self.extract_short_label(node_label);

        // If short and full labels are the same or very similar, enhance the full label
        let enhanced_full_label = if short_label.len() >= node_label.len() - 2 {
            // If they're nearly the same length, add more context to full label
            match short_label.as_str() {
                "inspect" => "inspect [debug output]".to_string(),
                "persist" => "persist [state storage]".to_string(),
                "tee" => "tee [branch dataflow]".to_string(),
                "delta" => "delta [change detection]".to_string(),
                "spin" => "spin [delay/buffer]".to_string(),
                "send_bincode" => "send_bincode [send data to process/cluster]".to_string(),
                "broadcast_bincode" => {
                    "broadcast_bincode [send data to all cluster members]".to_string()
                }
                "source_iter" => "source_iter [iterate over collection]".to_string(),
                "source_stream" => "source_stream [receive external data stream]".to_string(),
                "network(recv)" => "network(recv) [receive from network]".to_string(),
                "network(send)" => "network(send) [send to network]".to_string(),
                "dest_sink" => "dest_sink [output destination]".to_string(),
                _ => {
                    if node_label.len() < 15 {
                        format!("{} [{}]", node_label, "hydro operator")
                    } else {
                        node_label.to_string()
                    }
                }
            }
        } else {
            node_label.to_string()
        };

        let node = serde_json::json!({
            "id": node_id.to_string(),
            "type": "default",
            "data": {
                "label": short_label,
                "shortLabel": short_label,
                "fullLabel": enhanced_full_label,
                "expanded": false,
                "locationId": location_id,
                "locationType": location_type
            },
            "position": {
                "x": 0,
                "y": 0
            },
            "style": style
        });
        self.nodes.push(node);
        Ok(())
    }

    fn write_edge(
        &mut self,
        src_id: usize,
        dst_id: usize,
        edge_type: HydroEdgeType,
        label: Option<&str>,
    ) -> Result<(), Self::Err> {
        let style = self.edge_type_to_style(edge_type);
        let edge_id = format!("e{}", self.edge_count);
        self.edge_count += 1;

        let mut edge = serde_json::json!({
            "id": edge_id,
            "source": src_id.to_string(),
            "target": dst_id.to_string(),
            "style": style,
            // Use smart edge type for better routing and flexible connection points
            "type": "smoothstep",
            // Let ReactFlow choose optimal connection points
            // Remove fixed sourceHandle/targetHandle to enable flexible connections
            "animated": false
        });

        // Add animation for certain edge types
        if matches!(edge_type, HydroEdgeType::Network | HydroEdgeType::Cycle) {
            edge["animated"] = serde_json::Value::Bool(true);
        }

        if let Some(label_text) = label {
            edge["label"] = serde_json::Value::String(label_text.to_string());
            edge["labelStyle"] = serde_json::json!({
                "fontSize": "10px",
                "fontFamily": "monospace",
                "fill": "#333333",
                "backgroundColor": "rgba(255, 255, 255, 0.8)",
                "padding": "2px 4px",
                "borderRadius": "3px"
            });
            // Center the label on the edge
            edge["labelShowBg"] = serde_json::Value::Bool(true);
            edge["labelBgStyle"] = serde_json::json!({
                "fill": "rgba(255, 255, 255, 0.8)",
                "fillOpacity": 0.8
            });
        }

        self.edges.push(edge);
        Ok(())
    }

    fn write_location_start(
        &mut self,
        location_id: usize,
        location_type: &str,
    ) -> Result<(), Self::Err> {
        let location_label = format!("{} {}", location_type, location_id);
        self.locations
            .insert(location_id, (location_label, Vec::new()));
        Ok(())
    }

    fn write_node(&mut self, node_id: usize) -> Result<(), Self::Err> {
        // Find the current location being written and add this node to it
        if let Some((_, node_ids)) = self.locations.values_mut().last() {
            node_ids.push(node_id);
        }
        Ok(())
    }

    fn write_location_end(&mut self) -> Result<(), Self::Err> {
        // Location grouping complete - nothing to do for ReactFlow
        Ok(())
    }

    fn write_epilogue(&mut self) -> Result<(), Self::Err> {
        // Apply automatic layout using a simple algorithm
        self.apply_layout();

        // Create the final JSON structure
        let output = serde_json::json!({
            "nodes": self.nodes,
            "edges": self.edges,
            "locations": self.locations.iter().map(|(id, (label, nodes))| {
                serde_json::json!({
                    "id": id.to_string(),
                    "label": label,
                    "nodes": nodes
                })
            }).collect::<Vec<_>>()
        });

        write!(
            self.write,
            "{}",
            serde_json::to_string_pretty(&output).unwrap()
        )
    }
}
