//! Graph visualization utilities for Hydro IR

pub mod debug;
pub mod graphviz;
pub mod mermaid;
pub mod reactflow;
pub mod render;
pub mod template;

// Re-export for convenience
pub use render::{HydroDot, HydroMermaid, HydroReactFlow, escape_dot, escape_mermaid};
