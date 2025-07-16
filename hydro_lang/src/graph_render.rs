use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;

use auto_impl::auto_impl;
use quote::ToTokens;

use crate::ir::{HydroLeaf, HydroNode, HydroSource};

/// Trait for writing textual representations of Hydro IR graphs, i.e. mermaid or dot graphs.
#[auto_impl(&mut, Box)]
pub trait HydroGraphWrite {
    /// Error type emitted by writing.
    type Err: Error;

    /// Begin the graph. First method called.
    fn write_prologue(&mut self) -> Result<(), Self::Err>;

    /// Write a node definition with styling.
    fn write_node_definition(
        &mut self,
        node_id: usize,
        node_label: &str,
        node_type: HydroNodeType,
    ) -> Result<(), Self::Err>;

    /// Write an edge between nodes with optional labeling.
    fn write_edge(
        &mut self,
        src_id: usize,
        dst_id: usize,
        edge_type: HydroEdgeType,
        label: Option<&str>,
    ) -> Result<(), Self::Err>;

    /// Begin writing a location grouping (process/cluster).
    fn write_location_start(
        &mut self,
        location_id: usize,
        location_type: &str,
    ) -> Result<(), Self::Err>;

    /// Write a node within a location.
    fn write_node(&mut self, node_id: usize) -> Result<(), Self::Err>;

    /// End writing a location grouping.
    fn write_location_end(&mut self) -> Result<(), Self::Err>;

    /// End the graph. Last method called.
    fn write_epilogue(&mut self) -> Result<(), Self::Err>;
}

/// Types of nodes in Hydro IR for styling purposes.
#[derive(Debug, Clone, Copy)]
pub enum HydroNodeType {
    Source,
    Transform,
    Join,
    Aggregation,
    Network,
    Sink,
    Tee,
}

/// Types of edges in Hydro IR.
#[derive(Debug, Clone, Copy)]
pub enum HydroEdgeType {
    Stream,
    Persistent,
    Network,
    Cycle,
}

/// Configuration for graph writing.
#[derive(Debug, Clone)]
pub struct HydroWriteConfig {
    pub show_metadata: bool,
    pub show_location_groups: bool,
    pub include_tee_ids: bool,
}

impl Default for HydroWriteConfig {
    fn default() -> Self {
        Self {
            show_metadata: false,
            show_location_groups: true,
            include_tee_ids: true,
        }
    }
}

/// Escapes a string for use in a mermaid graph label.
pub fn escape_mermaid(string: &str) -> String {
    string
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('#', "&num;")
        .replace('\n', "<br>")
        // Handle code block markers
        .replace("`", "&#96;")
}

/// Mermaid graph writer for Hydro IR.
pub struct HydroMermaid<W> {
    write: W,
    indent: usize,
    link_count: usize,
}

impl<W> HydroMermaid<W> {
    pub fn new(write: W) -> Self {
        Self {
            write,
            indent: 0,
            link_count: 0,
        }
    }
}

impl<W> HydroGraphWrite for HydroMermaid<W>
where
    W: std::fmt::Write,
{
    type Err = std::fmt::Error;

    fn write_prologue(&mut self) -> Result<(), Self::Err> {
        writeln!(
            self.write,
            "{b:i$}%%{{init:{{'theme':'base','themeVariables':{{'clusterBkg':'#f0f0f0','clusterBorder':'#888'}}}}}}%%
{b:i$}flowchart TD
{b:i$}classDef sourceClass fill:#8f8,stroke:#000,text-align:left,white-space:pre
{b:i$}classDef transformClass fill:#88f,stroke:#000,text-align:left,white-space:pre
{b:i$}classDef joinClass fill:#f88,stroke:#000,text-align:left,white-space:pre
{b:i$}classDef aggClass fill:#ff8,stroke:#000,text-align:left,white-space:pre
{b:i$}classDef networkClass fill:#8ff,stroke:#000,text-align:left,white-space:pre
{b:i$}classDef sinkClass fill:#f8f,stroke:#000,text-align:left,white-space:pre
{b:i$}classDef teeClass fill:#ddd,stroke:#000,text-align:left,white-space:pre
{b:i$}linkStyle default stroke:#666",
            b = "",
            i = self.indent
        )?;
        Ok(())
    }

    fn write_node_definition(
        &mut self,
        node_id: usize,
        node_label: &str,
        node_type: HydroNodeType,
    ) -> Result<(), Self::Err> {
        let class_str = match node_type {
            HydroNodeType::Source => "sourceClass",
            HydroNodeType::Transform => "transformClass",
            HydroNodeType::Join => "joinClass",
            HydroNodeType::Aggregation => "aggClass",
            HydroNodeType::Network => "networkClass",
            HydroNodeType::Sink => "sinkClass",
            HydroNodeType::Tee => "teeClass",
        };
        
        let (lbracket, rbracket) = match node_type {
            HydroNodeType::Source => ("((", "))"),
            HydroNodeType::Sink => ("[/", "/]"),
            HydroNodeType::Network => ("[[", "]]"),
            HydroNodeType::Tee => ("(", ")"),
            _ => ("[", "]"),
        };

        let label = format!(
            r#"n{node_id}{lbracket}"{escaped_label}"{rbracket}:::{class}"#,
            escaped_label = escape_mermaid(node_label),
            class = class_str,
        );
        
        writeln!(self.write, "{b:i$}{label}", b = "", i = self.indent)?;
        Ok(())
    }

    fn write_edge(
        &mut self,
        src_id: usize,
        dst_id: usize,
        edge_type: HydroEdgeType,
        label: Option<&str>,
    ) -> Result<(), Self::Err> {
        let arrow_style = match edge_type {
            HydroEdgeType::Stream => "-->",
            HydroEdgeType::Persistent => "==>",
            HydroEdgeType::Network => "-.->",
            HydroEdgeType::Cycle => "--o",
        };

        write!(
            self.write,
            "{b:i$}n{src}{arrow}{label}n{dst}",
            src = src_id,
            arrow = arrow_style,
            label = if let Some(label) = label {
                Cow::Owned(format!("|{}|", escape_mermaid(label)))
            } else {
                Cow::Borrowed("")
            },
            dst = dst_id,
            b = "",
            i = self.indent,
        )?;

        // Add styling for different edge types
        if !matches!(edge_type, HydroEdgeType::Stream) {
            write!(
                self.write,
                "; linkStyle {} stroke:{}",
                self.link_count,
                match edge_type {
                    HydroEdgeType::Persistent => "#080",
                    HydroEdgeType::Network => "#808",
                    HydroEdgeType::Cycle => "#f80",
                    HydroEdgeType::Stream => "#666",
                }
            )?;
        }
        
        writeln!(self.write)?;
        self.link_count += 1;
        Ok(())
    }

    fn write_location_start(
        &mut self,
        location_id: usize,
        location_type: &str,
    ) -> Result<(), Self::Err> {
        writeln!(
            self.write,
            "{b:i$}subgraph loc_{id} [\"{location_type} {id}\"]",
            id = location_id,
            b = "",
            i = self.indent,
        )?;
        self.indent += 4;
        Ok(())
    }

    fn write_node(&mut self, node_id: usize) -> Result<(), Self::Err> {
        writeln!(
            self.write,
            "{b:i$}n{node_id}",
            b = "",
            i = self.indent
        )
    }

    fn write_location_end(&mut self) -> Result<(), Self::Err> {
        self.indent -= 4;
        writeln!(self.write, "{b:i$}end", b = "", i = self.indent)
    }

    fn write_epilogue(&mut self) -> Result<(), Self::Err> {
        Ok(())
    }
}

/// Escapes a string for use in a DOT graph label.
pub fn escape_dot(string: &str, newline: &str) -> String {
    string.replace('"', "\\\"").replace('\n', newline)
}

/// DOT/Graphviz graph writer for Hydro IR.
pub struct HydroDot<W> {
    write: W,
    indent: usize,
}

impl<W> HydroDot<W> {
    pub fn new(write: W) -> Self {
        Self { write, indent: 0 }
    }
}

impl<W> HydroGraphWrite for HydroDot<W>
where
    W: std::fmt::Write,
{
    type Err = std::fmt::Error;

    fn write_prologue(&mut self) -> Result<(), Self::Err> {
        writeln!(self.write, "{b:i$}digraph HydroIR {{", b = "", i = self.indent)?;
        self.indent += 4;

        const FONTS: &str = "\"Monaco,Menlo,Consolas,&quot;Droid Sans Mono&quot;,Inconsolata,&quot;Courier New&quot;,monospace\"";
        writeln!(
            self.write,
            "{b:i$}node [fontname={}, style=filled];",
            FONTS,
            b = "",
            i = self.indent
        )?;
        writeln!(
            self.write,
            "{b:i$}edge [fontname={}];",
            FONTS,
            b = "",
            i = self.indent
        )?;
        Ok(())
    }

    fn write_node_definition(
        &mut self,
        node_id: usize,
        node_label: &str,
        node_type: HydroNodeType,
    ) -> Result<(), Self::Err> {
        let escaped_label = escape_dot(node_label, "\\l");
        let label = format!("n{}", node_id);
        
        let (shape_str, color_str) = match node_type {
            HydroNodeType::Source => ("ellipse", "\"#88ff88\""),
            HydroNodeType::Transform => ("box", "\"#8888ff\""),
            HydroNodeType::Join => ("diamond", "\"#ff8888\""),
            HydroNodeType::Aggregation => ("house", "\"#ffff88\""),
            HydroNodeType::Network => ("doubleoctagon", "\"#88ffff\""),
            HydroNodeType::Sink => ("invhouse", "\"#ff88ff\""),
            HydroNodeType::Tee => ("circle", "\"#dddddd\""),
        };

        write!(
            self.write,
            "{b:i$}{label} [label=\"({node_id}) {escaped_label}{}\"",
            if escaped_label.contains("\\l") { "\\l" } else { "" },
            b = "",
            i = self.indent,
        )?;
        write!(self.write, ", shape={shape_str}, fillcolor={color_str}")?;
        writeln!(self.write, "]")?;
        Ok(())
    }

    fn write_edge(
        &mut self,
        src_id: usize,
        dst_id: usize,
        edge_type: HydroEdgeType,
        label: Option<&str>,
    ) -> Result<(), Self::Err> {
        let mut properties = Vec::<Cow<'static, str>>::new();
        
        if let Some(label) = label {
            properties.push(format!("label=\"{}\"", escape_dot(label, "\\n")).into());
        }

        // Styling based on edge type
        match edge_type {
            HydroEdgeType::Persistent => {
                properties.push("color=\"#008800\"".into());
                properties.push("style=\"bold\"".into());
            }
            HydroEdgeType::Network => {
                properties.push("color=\"#880088\"".into());
                properties.push("style=\"dashed\"".into());
            }
            HydroEdgeType::Cycle => {
                properties.push("color=\"#ff8800\"".into());
                properties.push("style=\"dotted\"".into());
            }
            HydroEdgeType::Stream => {}
        }

        write!(
            self.write,
            "{b:i$}n{} -> n{}",
            src_id,
            dst_id,
            b = "",
            i = self.indent,
        )?;
        
        if !properties.is_empty() {
            write!(self.write, " [")?;
            for prop in itertools::Itertools::intersperse(properties.into_iter(), ", ".into()) {
                write!(self.write, "{}", prop)?;
            }
            write!(self.write, "]")?;
        }
        writeln!(self.write)?;
        Ok(())
    }

    fn write_location_start(
        &mut self,
        location_id: usize,
        location_type: &str,
    ) -> Result<(), Self::Err> {
        writeln!(
            self.write,
            "{b:i$}subgraph cluster_loc_{id} {{",
            id = location_id,
            b = "",
            i = self.indent,
        )?;
        self.indent += 4;
        writeln!(self.write, "{b:i$}label = \"{location_type} {id}\"", id = location_id, b = "", i = self.indent)?;
        writeln!(self.write, "{b:i$}style=filled", b = "", i = self.indent)?;
        writeln!(self.write, "{b:i$}fillcolor=\"#f0f0f0\"", b = "", i = self.indent)?;
        Ok(())
    }

    fn write_node(&mut self, node_id: usize) -> Result<(), Self::Err> {
        writeln!(
            self.write,
            "{b:i$}n{node_id}",
            b = "",
            i = self.indent
        )
    }

    fn write_location_end(&mut self) -> Result<(), Self::Err> {
        self.indent -= 4;
        writeln!(self.write, "{b:i$}}}", b = "", i = self.indent)
    }

    fn write_epilogue(&mut self) -> Result<(), Self::Err> {
        self.indent -= 4;
        writeln!(self.write, "{b:i$}}}", b = "", i = self.indent)
    }
}

/// Graph structure tracker for Hydro IR rendering.
#[derive(Debug)]
pub struct HydroGraphStructure {
    pub nodes: HashMap<usize, (String, HydroNodeType, Option<usize>)>, // node_id -> (label, type, location)
    pub edges: Vec<(usize, usize, HydroEdgeType, Option<String>)>, // (src, dst, edge_type, label)
    pub locations: HashMap<usize, String>, // location_id -> location_type
    pub next_node_id: usize,
}

impl HydroGraphStructure {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            locations: HashMap::new(),
            next_node_id: 0,
        }
    }

    pub fn add_node(&mut self, label: String, node_type: HydroNodeType, location: Option<usize>) -> usize {
        let node_id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.insert(node_id, (label, node_type, location));
        node_id
    }

    pub fn add_edge(&mut self, src: usize, dst: usize, edge_type: HydroEdgeType, label: Option<String>) {
        self.edges.push((src, dst, edge_type, label));
    }

    pub fn add_location(&mut self, location_id: usize, location_type: String) {
        self.locations.insert(location_id, location_type);
    }
}

impl HydroLeaf {
    /// Generate a mermaid graph representation of this Hydro IR leaf and its subgraph.
    pub fn to_mermaid(&self, config: &HydroWriteConfig) -> String {
        let mut output = String::new();
        self.write_mermaid(&mut output, config).unwrap();
        output
    }

    /// Write mermaid representation to the given writer.
    pub fn write_mermaid(
        &self,
        output: impl std::fmt::Write,
        config: &HydroWriteConfig,
    ) -> std::fmt::Result {
        let mut graph_write = HydroMermaid::new(output);
        self.write_graph(&mut graph_write, config)
    }

    /// Generate a DOT/Graphviz graph representation of this Hydro IR leaf and its subgraph.
    pub fn to_dot(&self, config: &HydroWriteConfig) -> String {
        let mut output = String::new();
        self.write_dot(&mut output, config).unwrap();
        output
    }

    /// Write DOT representation to the given writer.
    pub fn write_dot(
        &self,
        output: impl std::fmt::Write,
        config: &HydroWriteConfig,
    ) -> std::fmt::Result {
        let mut graph_write = HydroDot::new(output);
        self.write_graph(&mut graph_write, config)
    }

    /// Core graph writing logic that works with any GraphWrite implementation.
    pub fn write_graph<W>(
        &self,
        mut graph_write: W,
        config: &HydroWriteConfig,
    ) -> Result<(), W::Err>
    where
        W: HydroGraphWrite,
    {
        let mut structure = HydroGraphStructure::new();
        let mut seen_tees = HashMap::new();
        
        // Build the graph structure by traversing the IR
        let _sink_id = self.build_graph_structure(&mut structure, &mut seen_tees, config);
        
        // Write the graph
        graph_write.write_prologue()?;
        
        // Write node definitions
        for (&node_id, (label, node_type, _location)) in &structure.nodes {
            graph_write.write_node_definition(node_id, label, *node_type)?;
        }
        
        // Group nodes by location if requested
        if config.show_location_groups {
            let mut nodes_by_location: HashMap<usize, Vec<usize>> = HashMap::new();
            for (&node_id, (_, _, location)) in &structure.nodes {
                if let Some(location_id) = location {
                    nodes_by_location.entry(*location_id).or_default().push(node_id);
                }
            }
            
            for (&location_id, node_ids) in &nodes_by_location {
                if let Some(location_type) = structure.locations.get(&location_id) {
                    graph_write.write_location_start(location_id, location_type)?;
                    for &node_id in node_ids {
                        graph_write.write_node(node_id)?;
                    }
                    graph_write.write_location_end()?;
                }
            }
        }
        
        // Write edges
        for (src_id, dst_id, edge_type, label) in &structure.edges {
            graph_write.write_edge(*src_id, *dst_id, *edge_type, label.as_deref())?;
        }
        
        graph_write.write_epilogue()?;
        Ok(())
    }

    /// Build the graph structure by traversing the IR tree.
    pub fn build_graph_structure(
        &self,
        structure: &mut HydroGraphStructure,
        seen_tees: &mut HashMap<*const std::cell::RefCell<HydroNode>, usize>,
        config: &HydroWriteConfig,
    ) -> usize {
        use crate::location::LocationId;
        
        // Helper function to extract location without capturing structure
        fn extract_location_id(metadata: &crate::ir::HydroIrMetadata) -> (Option<usize>, Option<String>) {
            match &metadata.location_kind {
                LocationId::Process(id) => (Some(*id), Some("Process".to_string())),
                LocationId::Cluster(id) => (Some(*id), Some("Cluster".to_string())),
                LocationId::ExternalProcess(id) => (Some(*id), Some("External".to_string())),
                LocationId::Tick(_, inner) => {
                    match inner.as_ref() {
                        LocationId::Process(id) => (Some(*id), Some("Process".to_string())),
                        LocationId::Cluster(id) => (Some(*id), Some("Cluster".to_string())),
                        _ => (None, None),
                    }
                }
            }
        }

        match self {
            HydroLeaf::ForEach { f, input, metadata } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let sink_id = structure.add_node(
                    format!("for_each({})", f.to_token_stream()),
                    HydroNodeType::Sink,
                    location_id,
                );
                structure.add_edge(input_id, sink_id, HydroEdgeType::Stream, None);
                sink_id
            }
            HydroLeaf::DestSink { sink, input, metadata } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let sink_id = structure.add_node(
                    format!("dest_sink({})", sink.to_token_stream()),
                    HydroNodeType::Sink,
                    location_id,
                );
                structure.add_edge(input_id, sink_id, HydroEdgeType::Stream, None);
                sink_id
            }
            HydroLeaf::CycleSink { ident, input, metadata, .. } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let sink_id = structure.add_node(
                    format!("cycle_sink({})", ident),
                    HydroNodeType::Sink,
                    location_id,
                );
                structure.add_edge(input_id, sink_id, HydroEdgeType::Cycle, None);
                sink_id
            }
        }
    }
}

impl HydroNode {
    /// Build the graph structure recursively for this node.
    pub fn build_graph_structure(
        &self,
        structure: &mut HydroGraphStructure,
        seen_tees: &mut HashMap<*const std::cell::RefCell<HydroNode>, usize>,
        config: &HydroWriteConfig,
    ) -> usize {
        use crate::location::LocationId;
        
        // Helper function to extract location without capturing structure
        fn extract_location_id(metadata: &crate::ir::HydroIrMetadata) -> (Option<usize>, Option<String>) {
            match &metadata.location_kind {
                LocationId::Process(id) => (Some(*id), Some("Process".to_string())),
                LocationId::Cluster(id) => (Some(*id), Some("Cluster".to_string())),
                LocationId::ExternalProcess(id) => (Some(*id), Some("External".to_string())),
                LocationId::Tick(_, inner) => {
                    match inner.as_ref() {
                        LocationId::Process(id) => (Some(*id), Some("Process".to_string())),
                        LocationId::Cluster(id) => (Some(*id), Some("Cluster".to_string())),
                        _ => (None, None),
                    }
                }
            }
        }

        match self {
            HydroNode::Placeholder => {
                structure.add_node("PLACEHOLDER".to_string(), HydroNodeType::Transform, None)
            }
            
            HydroNode::Source { source, metadata, .. } => {
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let label = match source {
                    HydroSource::Stream(expr) => format!("source_stream({})", expr.to_token_stream()),
                    HydroSource::ExternalNetwork() => "external_network()".to_string(),
                    HydroSource::Iter(expr) => format!("source_iter({})", expr.to_token_stream()),
                    HydroSource::Spin() => "spin()".to_string(),
                };
                structure.add_node(label, HydroNodeType::Source, location_id)
            }
            
            HydroNode::CycleSource { ident, metadata, .. } => {
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                structure.add_node(
                    format!("cycle_source({})", ident),
                    HydroNodeType::Source,
                    location_id,
                )
            }
            
            HydroNode::Tee { inner, metadata } => {
                let ptr = inner.as_ptr();
                if let Some(&existing_id) = seen_tees.get(&ptr) {
                    return existing_id;
                }
                
                let input_id = inner.0.borrow().build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let tee_id = if config.include_tee_ids {
                    structure.add_node("tee()".to_string(), HydroNodeType::Tee, location_id)
                } else {
                    input_id // If not showing tee nodes, just return the input
                };
                
                seen_tees.insert(ptr, tee_id);
                
                if config.include_tee_ids {
                    structure.add_edge(input_id, tee_id, HydroEdgeType::Stream, None);
                }
                tee_id
            }
            
            HydroNode::Persist { inner, metadata } => {
                let input_id = inner.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let persist_id = structure.add_node("persist()".to_string(), HydroNodeType::Transform, location_id);
                structure.add_edge(input_id, persist_id, HydroEdgeType::Persistent, None);
                persist_id
            }
            
            HydroNode::Delta { inner, metadata } => {
                let input_id = inner.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let delta_id = structure.add_node("delta()".to_string(), HydroNodeType::Transform, location_id);
                structure.add_edge(input_id, delta_id, HydroEdgeType::Stream, None);
                delta_id
            }
            
            HydroNode::Map { f, input, metadata } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let map_id = structure.add_node(
                    format!("map({})", f.to_token_stream()),
                    HydroNodeType::Transform,
                    location_id,
                );
                structure.add_edge(input_id, map_id, HydroEdgeType::Stream, None);
                map_id
            }
            
            HydroNode::Filter { f, input, metadata } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let filter_id = structure.add_node(
                    format!("filter({})", f.to_token_stream()),
                    HydroNodeType::Transform,
                    location_id,
                );
                structure.add_edge(input_id, filter_id, HydroEdgeType::Stream, None);
                filter_id
            }
            
            HydroNode::Join { left, right, metadata } => {
                let left_id = left.build_graph_structure(structure, seen_tees, config);
                let right_id = right.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let join_id = structure.add_node("join()".to_string(), HydroNodeType::Join, location_id);
                structure.add_edge(left_id, join_id, HydroEdgeType::Stream, Some("left".to_string()));
                structure.add_edge(right_id, join_id, HydroEdgeType::Stream, Some("right".to_string()));
                join_id
            }
            
            HydroNode::Fold { init, acc, input, metadata } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (location_id, location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                let fold_id = structure.add_node(
                    format!("fold({}, {})", init.to_token_stream(), acc.to_token_stream()),
                    HydroNodeType::Aggregation,
                    location_id,
                );
                structure.add_edge(input_id, fold_id, HydroEdgeType::Stream, None);
                fold_id
            }
            
            HydroNode::Network { 
                to_location, 
                serialize_fn, 
                deserialize_fn, 
                input, 
                metadata, 
                .. 
            } => {
                let input_id = input.build_graph_structure(structure, seen_tees, config);
                let (from_location_id, from_location_type) = extract_location_id(metadata);
                if let (Some(loc_id), Some(loc_type)) = (from_location_id, from_location_type) {
                    structure.add_location(loc_id, loc_type);
                }
                
                let to_location_id = match to_location {
                    LocationId::Process(id) => {
                        structure.add_location(*id, "Process".to_string());
                        Some(*id)
                    }
                    LocationId::Cluster(id) => {
                        structure.add_location(*id, "Cluster".to_string());
                        Some(*id)
                    }
                    LocationId::ExternalProcess(id) => {
                        structure.add_location(*id, "External".to_string());
                        Some(*id)
                    }
                    _ => None,
                };
                
                let mut label = "network(".to_string();
                if serialize_fn.is_some() {
                    label.push_str("ser");
                }
                if deserialize_fn.is_some() {
                    if serialize_fn.is_some() { label.push_str(" + "); }
                    label.push_str("deser");
                }
                label.push(')');
                
                let network_id = structure.add_node(label, HydroNodeType::Network, to_location_id);
                structure.add_edge(
                    input_id, 
                    network_id, 
                    HydroEdgeType::Network, 
                    Some(format!("to {:?}", to_location_id))
                );
                network_id
            }
            
            // Handle all other node types with similar pattern
            _ => {
                // This is a temporary catch-all for remaining node types
                // In a real implementation, you'd want to handle each specifically
                match self {
                    HydroNode::FlatMap { f, input, metadata } => {
                        let input_id = input.build_graph_structure(structure, seen_tees, config);
                        let (location_id, location_type) = extract_location_id(metadata);
                        if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                            structure.add_location(loc_id, loc_type);
                        }
                        let node_id = structure.add_node(
                            format!("flat_map({})", f.to_token_stream()),
                            HydroNodeType::Transform,
                            location_id,
                        );
                        structure.add_edge(input_id, node_id, HydroEdgeType::Stream, None);
                        node_id
                    }
                    
                    HydroNode::FilterMap { f, input, metadata } => {
                        let input_id = input.build_graph_structure(structure, seen_tees, config);
                        let (location_id, location_type) = extract_location_id(metadata);
                        if let (Some(loc_id), Some(loc_type)) = (location_id, location_type) {
                            structure.add_location(loc_id, loc_type);
                        }
                        let node_id = structure.add_node(
                            format!("filter_map({})", f.to_token_stream()),
                            HydroNodeType::Transform,
                            location_id,
                        );
                        structure.add_edge(input_id, node_id, HydroEdgeType::Stream, None);
                        node_id
                    }
                    
                    HydroNode::Unpersist { inner, .. } => {
                        // Unpersist is typically optimized away, just pass through
                        inner.build_graph_structure(structure, seen_tees, config)
                    }
                    
                    _ => {
                        // For truly unhandled cases, create a basic node
                        structure.add_node(
                            format!("{:?}", self).split(' ').next().unwrap_or("unknown").to_string(),
                            HydroNodeType::Transform,
                            None,
                        )
                    }
                }
            }
        }
    }
}

/// Utility functions for rendering multiple leaves as a single graph.
pub fn render_hydro_ir_mermaid(leaves: &[HydroLeaf], config: &HydroWriteConfig) -> String {
    let mut output = String::new();
    write_hydro_ir_mermaid(&mut output, leaves, config).unwrap();
    output
}

pub fn write_hydro_ir_mermaid(
    output: impl std::fmt::Write,
    leaves: &[HydroLeaf],
    config: &HydroWriteConfig,
) -> std::fmt::Result {
    let mut graph_write = HydroMermaid::new(output);
    write_hydro_ir_graph(&mut graph_write, leaves, config)
}

pub fn render_hydro_ir_dot(leaves: &[HydroLeaf], config: &HydroWriteConfig) -> String {
    let mut output = String::new();
    write_hydro_ir_dot(&mut output, leaves, config).unwrap();
    output
}

pub fn write_hydro_ir_dot(
    output: impl std::fmt::Write,
    leaves: &[HydroLeaf],
    config: &HydroWriteConfig,
) -> std::fmt::Result {
    let mut graph_write = HydroDot::new(output);
    write_hydro_ir_graph(&mut graph_write, leaves, config)
}

fn write_hydro_ir_graph<W>(
    mut graph_write: W,
    leaves: &[HydroLeaf],
    config: &HydroWriteConfig,
) -> Result<(), W::Err>
where
    W: HydroGraphWrite,
{
    let mut structure = HydroGraphStructure::new();
    let mut seen_tees = HashMap::new();
    
    // Build the graph structure for all leaves
    for leaf in leaves {
        leaf.build_graph_structure(&mut structure, &mut seen_tees, config);
    }
    
    // Write the graph using the same logic as individual leaves
    graph_write.write_prologue()?;
    
    for (&node_id, (label, node_type, _)) in &structure.nodes {
        graph_write.write_node_definition(node_id, label, *node_type)?;
    }
    
    if config.show_location_groups {
        let mut nodes_by_location: HashMap<usize, Vec<usize>> = HashMap::new();
        for (&node_id, (_, _, location)) in &structure.nodes {
            if let Some(location_id) = location {
                nodes_by_location.entry(*location_id).or_default().push(node_id);
            }
        }
        
        for (&location_id, node_ids) in &nodes_by_location {
            if let Some(location_type) = structure.locations.get(&location_id) {
                graph_write.write_location_start(location_id, location_type)?;
                for &node_id in node_ids {
                    graph_write.write_node(node_id)?;
                }
                graph_write.write_location_end()?;
            }
        }
    }
    
    for (src_id, dst_id, edge_type, label) in &structure.edges {
        graph_write.write_edge(*src_id, *dst_id, *edge_type, label.as_deref())?;
    }
    
    graph_write.write_epilogue()?;
    Ok(())
}
