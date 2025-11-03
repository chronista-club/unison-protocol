use unison::prelude::*;

#[test]
fn test_basic_kdl_parsing() {
    let schema_str = r#"
protocol "TestProtocol" version="1.0.0" {
    service "TestService" {
        method "testMethod" {
            request {
                field "test" type="string" required=true
            }
            response {
                field "result" type="bool"
            }
        }
    }
}
"#;

    let parser = SchemaParser::new();
    let result = parser.parse(schema_str);

    assert!(result.is_ok(), "パース失敗: {:?}", result.err());

    let schema = result.unwrap();
    assert!(schema.protocol.is_some(), "プロトコルが見つかりません");

    let protocol = schema.protocol.unwrap();
    assert_eq!(protocol.name, "TestProtocol");
    assert_eq!(protocol.version, "1.0.0");
    assert_eq!(protocol.services.len(), 1);

    let service = &protocol.services[0];
    assert_eq!(service.name, "TestService");
    assert_eq!(service.methods.len(), 1);

    let method = &service.methods[0];
    assert_eq!(method.name, "testMethod");
    assert!(method.request.is_some());
    assert!(method.response.is_some());
}

#[test]
fn test_message_with_fields() {
    let schema_str = r#"
message "User" {
    field "id" type="int" required=true
    field "name" type="string" required=true
    field "email" type="string"
    field "age" type="int" min=0 max=150
}
"#;

    let parser = SchemaParser::new();
    let result = parser.parse(schema_str);

    assert!(result.is_ok());

    let schema = result.unwrap();
    assert_eq!(schema.messages.len(), 1);

    let message = &schema.messages[0];
    assert_eq!(message.name, "User");
    assert_eq!(message.fields.len(), 4);

    let id_field = &message.fields[0];
    assert_eq!(id_field.name, "id");
    assert!(id_field.required);
}

#[test]
fn test_enum_parsing() {
    let schema_str = r#"
enum "Status" {
    values "pending" "active" "completed" "cancelled"
}
"#;

    let parser = SchemaParser::new();
    let result = parser.parse(schema_str);

    assert!(result.is_ok());

    let schema = result.unwrap();
    assert_eq!(schema.enums.len(), 1);

    let enum_def = &schema.enums[0];
    assert_eq!(enum_def.name, "Status");
    assert_eq!(enum_def.values.len(), 4);
    assert_eq!(enum_def.values[0], "pending");
}
