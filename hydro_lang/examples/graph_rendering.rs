use hydro_lang::graph_render::{HydroWriteConfig, render_hydro_ir_mermaid, render_hydro_ir_dot};
use hydro_lang::ir::*;
use hydro_lang::location::LocationId;
use syn::parse_quote;

#[cfg(feature = "debugging")]
use hydro_lang::graph_debug::{open_hydro_ir_mermaid_vscode, open_hydro_ir_dot_vscode, open_hydro_ir_mermaid_simple_browser};

fn main() {
    // Create a simple Hydro IR graph
    let leaves = create_simple_hydro_ir();
    
    println!("Generated Hydro IR with {} leaves", leaves.len());
    
    // Render to Mermaid format (returns String directly)
    let mermaid_config = HydroWriteConfig::default();
    let mermaid_output = render_hydro_ir_mermaid(&leaves, &mermaid_config);
    println!("=== Mermaid Output ===");
    println!("{}", mermaid_output);
    
    // Write to file
    std::fs::write("hydro_graph.mermaid", &mermaid_output).unwrap();
    println!("Mermaid diagram saved to hydro_graph.mermaid");
    
    println!("\n");
    
    // Render to DOT format (returns String directly)
    let dot_config = HydroWriteConfig::default();
    let dot_output = render_hydro_ir_dot(&leaves, &dot_config);
    println!("=== DOT Output ===");
    println!("{}", dot_output);
    
    // Write to file
    std::fs::write("hydro_graph.dot", &dot_output).unwrap();
    println!("DOT diagram saved to hydro_graph.dot");
    
    // VS Code Integration Examples (requires debugging feature)
    #[cfg(feature = "debugging")]
    {
        println!("\n=== VS Code Integration Examples ===");
        
        // Option 1: Save and open .mermaid file in VS Code
        // (Requires Mermaid Preview extension)
        match open_hydro_ir_mermaid_vscode(&leaves, Some("demo_graph.mermaid"), None) {
            Ok(_) => println!("✓ Opened Mermaid file in VS Code"),
            Err(e) => println!("⚠ Could not open in VS Code: {}", e),
        }
        
        // Option 2: Save and open .dot file in VS Code  
        // (Requires Graphviz extension)
        match open_hydro_ir_dot_vscode(&leaves, Some("demo_graph.dot"), None) {
            Ok(_) => println!("✓ Opened DOT file in VS Code"),
            Err(e) => println!("⚠ Could not open in VS Code: {}", e),
        }
        
        // Option 3: Open in VS Code Simple Browser
        match open_hydro_ir_mermaid_simple_browser(&leaves, None) {
            Ok(_) => println!("✓ Opened Mermaid in VS Code Simple Browser"),
            Err(e) => println!("⚠ Could not open in Simple Browser: {}", e),
        }
        
        println!("\nTips:");
        println!("- Install 'Mermaid Preview' extension for .mermaid files");
        println!("- Install 'Graphviz (dot) language support' for .dot files");
        println!("- Use Ctrl+Shift+V (Cmd+Shift+V on Mac) to preview in VS Code");
    }
    
    #[cfg(not(feature = "debugging"))]
    {
        println!("\n=== VS Code Integration ===");
        println!("To enable VS Code integration, build with: cargo run --example graph_rendering --features debugging");
    }
}

fn create_simple_hydro_ir() -> Vec<HydroLeaf> {
    // Create some simple mock expressions using syn::parse_quote
    let source_expr = DebugExpr(Box::new(parse_quote!(source_stream)));
    let filter_expr = DebugExpr(Box::new(parse_quote!(filter_func)));
    let map_expr = DebugExpr(Box::new(parse_quote!(map_func)));
    let sink_expr = DebugExpr(Box::new(parse_quote!(output_sink)));
    
    // Mock location data  
    let location = LocationId::Process(0);
    let metadata = HydroIrMetadata {
        location_kind: location,
        output_type: None,
        cardinality: None,
        cpu_usage: None,
    };
    
    // Create a simple linear pipeline: source -> filter -> map -> sink
    vec![HydroLeaf::DestSink {
        sink: sink_expr,
        input: Box::new(HydroNode::Map {
            f: map_expr,
            input: Box::new(HydroNode::Filter {
                f: filter_expr,
                input: Box::new(HydroNode::Source {
                    source: HydroSource::Stream(source_expr),
                    location_kind: metadata.location_kind.clone(),
                    metadata: metadata.clone(),
                }),
                metadata: metadata.clone(),
            }),
            metadata: metadata.clone(),
        }),
        metadata,
    }]
}

