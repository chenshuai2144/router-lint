fn gen_diagnostic(node: &RoutePathObj, source_file_name: String) -> RouteDiagnostic {
    let mut line_text = Vec::new();
    line_text.push(node.node_source.to_string());

    let mut display_position = Vec::new();
    display_position.push(node.display_position.clone());
    let route_diagnostic = RouteDiagnostic {
        specifier: node.path.clone(),
        display_position: display_position,
        kind: RouteSyntaxError::RedirectRedundancy,
        source_file_name: source_file_name,
    };
    route_diagnostic
}

fn print_diagnostic(diagnostic: &RouteDiagnostic) {
    println!("🚨 redirect 的冗余配置，发现于以下行：",);
    for line_and_column in &diagnostic.display_position {
        println!(
            "   ---> {}:{}:{} 的 {}",
            diagnostic.source_file_name,
            line_and_column.line,
            line_and_column.column,
            line_and_column.router_source_code
        );
    }
    println!("");
    println!("redirect 路由中应该只配置 redirect 和 path 两个属性！",);
}
