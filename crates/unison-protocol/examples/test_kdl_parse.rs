use kdl::KdlDocument;

fn main() {
    let test_cases = vec![
        (r#"field "test" type="string" required=true"#, "bare true"),
        (
            r#"field "test" type="string" required=#true"#,
            "typed #true",
        ),
        (
            r#"field "test" type="string" required="true""#,
            "string \"true\"",
        ),
    ];

    for (test, desc) in test_cases {
        println!("Testing: {} ({})", desc, test);
        match test.parse::<KdlDocument>() {
            Ok(doc) => {
                if let Some(node) = doc.nodes().first() {
                    if let Some(val) = node.get("required") {
                        println!("  ✓ Parsed! as_bool() = {:?}", val.as_bool());
                    } else {
                        println!("  ✗ No 'required' property");
                    }
                }
            }
            Err(e) => println!("  ✗ Parse error: {}", e),
        }
    }
}
