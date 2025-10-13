use kdl::KdlDocument;

fn main() {
    let test_cases = vec![
        r#"field "test" type="string" required=true"#,
        r#"field "test" type="string" required=#true"#,
        r#"field "test" type="string" required="true""#,
    ];

    for (i, test) in test_cases.iter().enumerate() {
        println!("Test case {}: {}", i + 1, test);
        match test.parse::<KdlDocument>() {
            Ok(doc) => {
                if let Some(node) = doc.nodes().first() {
                    if let Some(val) = node.get("required") {
                        println!("  ✓ Parsed! required value: {:?}", val);
                        println!("    as_bool(): {:?}", val.as_bool());
                        println!("    as_string(): {:?}", val.as_string());
                    } else {
                        println!("  ✗ No 'required' property found");
                    }
                }
            }
            Err(e) => {
                println!("  ✗ Parse error: {}", e);
            }
        }
        println!();
    }
}