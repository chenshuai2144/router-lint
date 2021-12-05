fn gen_diagnostic_repeat(
    node: &RoutePathObj,
    repeat_node: &RoutePathObj,
    source_file_name: String,
) -> RouteDiagnostic {
    let mut line_text = Vec::new();
    line_text.push(node.node_source.to_string());
    line_text.push(repeat_node.node_source.to_string());

    let mut display_position = Vec::new();
    display_position.push(node.display_position.clone());
    display_position.push(repeat_node.display_position.clone());
    let route_diagnostic = RouteDiagnostic {
        specifier: node.path.clone(),
        display_position: display_position,
        kind: RouteSyntaxError::Repeat,
        source_file_name: source_file_name,
    };
    route_diagnostic
}

fn print_diagnostic_repeat(diagnostic: &RouteDiagnostic) {
    println!("🚨 {} 重复声明，发现于以下行：", diagnostic.specifier);
    for line_and_column in &diagnostic.display_position {
        println!(
            "   ---> {}:{}:{} 的 {}",
            diagnostic.source_file_name,
            line_and_column.line,
            line_and_column.column,
            line_and_column.line_text[0]
        );
    }
    println!("");
    println!("如果是父子路由，请使用 ./ 来代替",);
    let message = "\
    💡  更改方案：
    {
        path: '/user',
        layout: false,
        routes: [
            {
                path: '/user',
                component: './user/Login',
            },
        ],
    },

    可以转化为 ======>

    {
        path: '/user',
        layout: false,
        routes: [
            {
                path: './',
                component: './user/Login',
            },
        ],
    },
    
";
    println!("{}", message);
}
