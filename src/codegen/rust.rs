use anyhow::Result;
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use crate::parser::{
    ParsedSchema, TypeRegistry, Protocol, Service, Method, Stream, 
    Message, Field, FieldType, Enum, DefaultValue
};
use super::CodeGenerator;

pub struct RustGenerator;

impl RustGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl CodeGenerator for RustGenerator {
    fn generate(&self, schema: &ParsedSchema, type_registry: &TypeRegistry) -> Result<String> {
        let mut tokens = TokenStream::new();

        // インポート文を追加
        tokens.extend(self.generate_imports());

        // 列挙型を生成
        for enum_def in &schema.enums {
            tokens.extend(self.generate_enum(enum_def));
        }

        // メッセージを生成
        for message in &schema.messages {
            tokens.extend(self.generate_message(message, type_registry));
        }

        // プロトコル固有のコードを生成
        if let Some(protocol) = &schema.protocol {
            tokens.extend(self.generate_protocol(protocol, type_registry));
        }

        // 生成されたコードをフォーマット
        let code = tokens.to_string();
        Ok(self.format_code(&code))
    }
}

impl RustGenerator {
    fn generate_imports(&self) -> TokenStream {
        quote! {
            use serde::{Deserialize, Serialize};
            use anyhow::Result;
            use chrono::{DateTime, Utc};
            use uuid::Uuid;
            use std::collections::HashMap;
            
            #[allow(unused_imports)]
            use crate::network::{ProtocolClient, ProtocolServer};
        }
    }
    
    fn generate_protocol(&self, protocol: &Protocol, type_registry: &TypeRegistry) -> TokenStream {
        let mut tokens = TokenStream::new();

        // プロトコルの列挙型を生成
        for enum_def in &protocol.enums {
            tokens.extend(self.generate_enum(enum_def));
        }

        // プロトコルのメッセージを生成
        for message in &protocol.messages {
            tokens.extend(self.generate_message(message, type_registry));
        }

        // サービスを生成
        for service in &protocol.services {
            tokens.extend(self.generate_service(service, type_registry));
        }

        tokens
    }
    
    fn generate_enum(&self, enum_def: &Enum) -> TokenStream {
        let name = format_ident!("{}", enum_def.name);
        let variants: Vec<_> = enum_def.values.iter()
            .map(|v| {
                let variant = format_ident!("{}", v.to_case(Case::Pascal));
                let value = v;
                quote! {
                    #[serde(rename = #value)]
                    #variant
                }
            })
            .collect();
        
        quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
            #[serde(rename_all = "snake_case")]
            pub enum #name {
                #(#variants),*
            }
        }
    }
    
    fn generate_message(&self, message: &Message, type_registry: &TypeRegistry) -> TokenStream {
        let name = format_ident!("{}", message.name.trim_start_matches("_inline_"));

        // インラインメッセージはスキップ
        if message.name.starts_with("_inline_") {
            return TokenStream::new();
        }

        let fields: Vec<_> = message.fields.iter()
            .map(|f| self.generate_field(f, type_registry))
            .collect();

        quote! {
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct #name {
                #(#fields),*
            }
        }
    }
    
    fn generate_field(&self, field: &Field, type_registry: &TypeRegistry) -> TokenStream {
        let name = format_ident!("{}", field.name);
        let rust_type = self.field_type_to_rust(&field.field_type(), type_registry);

        let mut attributes = vec![];

        // 必要に応じてserdeのrenameを追加
        if field.name != field.name.to_case(Case::Snake) {
            let rename = &field.name;
            attributes.push(quote! { #[serde(rename = #rename)] });
        }

        // オプショナルフィールドの処理
        let (field_type, extra_attrs) = if !field.required {
            (
                quote! { Option<#rust_type> },
                quote! { #[serde(skip_serializing_if = "Option::is_none")] }
            )
        } else {
            (rust_type, TokenStream::new())
        };

        // デフォルト値の処理
        let default_attr = if let Some(default) = &field.default() {
            self.generate_default_attr(default)
        } else {
            TokenStream::new()
        };

        quote! {
            #(#attributes)*
            #extra_attrs
            #default_attr
            pub #name: #field_type
        }
    }
    
    fn field_type_to_rust(&self, field_type: &FieldType, type_registry: &TypeRegistry) -> TokenStream {
        match field_type {
            FieldType::String => quote! { String },
            FieldType::Int => quote! { i64 },
            FieldType::Float => quote! { f64 },
            FieldType::Bool => quote! { bool },
            FieldType::Json | FieldType::Object => quote! { serde_json::Value },
            FieldType::Array(inner) => {
                let inner_type = self.field_type_to_rust(inner, type_registry);
                quote! { Vec<#inner_type> }
            }
            FieldType::Map(key, value) => {
                let key_type = self.field_type_to_rust(key, type_registry);
                let value_type = self.field_type_to_rust(value, type_registry);
                quote! { HashMap<#key_type, #value_type> }
            }
            FieldType::Enum(_) => {
                // 実際の列挙型に解決される必要がある
                quote! { String }
            }
            FieldType::Custom(name) => {
                if let Some(rust_type) = type_registry.get_rust_type(name) {
                    let tokens: TokenStream = rust_type.parse().unwrap_or_else(|_| quote! { String });
                    tokens
                } else {
                    let ident = format_ident!("{}", name);
                    quote! { #ident }
                }
            }
        }
    }
    
    fn generate_default_attr(&self, default: &DefaultValue) -> TokenStream {
        match default {
            DefaultValue::String(s) => {
                quote! { #[serde(default = #s)] }
            }
            DefaultValue::Int(i) => {
                let default_fn = format!("default_{}", i);
                quote! { #[serde(default = #default_fn)] }
            }
            DefaultValue::Float(f) => {
                let default_fn = format!("default_{}", f);
                quote! { #[serde(default = #default_fn)] }
            }
            DefaultValue::Bool(b) => {
                if *b {
                    quote! { #[serde(default = "default_true")] }
                } else {
                    quote! { #[serde(default)] }
                }
            }
            _ => TokenStream::new(),
        }
    }
    
    fn generate_service(&self, service: &Service, type_registry: &TypeRegistry) -> TokenStream {
        let service_name = format_ident!("{}Service", service.name);
        let client_name = format_ident!("{}Client", service.name);
        
        let methods: Vec<_> = service.methods.iter()
            .map(|m| self.generate_service_method(m, type_registry))
            .collect();
        
        let streams: Vec<_> = service.streams.iter()
            .map(|s| self.generate_service_stream(s, type_registry))
            .collect();
        
        let client_methods: Vec<_> = service.methods.iter()
            .map(|m| self.generate_client_method(m, type_registry))
            .collect();
        
        let client_streams: Vec<_> = service.streams.iter()
            .map(|s| self.generate_client_stream(s, type_registry))
            .collect();
        
        quote! {
            // サービストレイト
            pub trait #service_name: Send + Sync {
                #(#methods)*
                #(#streams)*
            }

            // クライアント実装
            pub struct #client_name {
                inner: Box<dyn ProtocolClient>,
            }

            impl #client_name {
                pub fn new(client: Box<dyn ProtocolClient>) -> Self {
                    Self { inner: client }
                }

                #(#client_methods)*
                #(#client_streams)*
            }
        }
    }
    
    fn generate_service_method(&self, method: &Method, _type_registry: &TypeRegistry) -> TokenStream {
        let name = format_ident!("{}", method.name.to_case(Case::Snake));
        let request_type = self.method_type_name(&method.request, "Request");
        let response_type = self.method_type_name(&method.response, "Response");
        
        quote! {
            async fn #name(&self, request: #request_type) -> Result<#response_type>;
        }
    }
    
    fn generate_service_stream(&self, stream: &Stream, _type_registry: &TypeRegistry) -> TokenStream {
        let name = format_ident!("{}", stream.name.to_case(Case::Snake));
        let request_type = self.method_type_name(&stream.request, "Request");
        let response_type = self.method_type_name(&stream.response, "Response");
        
        quote! {
            async fn #name(
                &self, 
                request: #request_type
            ) -> Result<Box<dyn futures_util::Stream<Item = Result<#response_type>> + Send + Unpin>>;
        }
    }
    
    fn generate_client_method(&self, method: &Method, _type_registry: &TypeRegistry) -> TokenStream {
        let name = format_ident!("{}", method.name.to_case(Case::Snake));
        let request_type = self.method_type_name(&method.request, "Request");
        let response_type = self.method_type_name(&method.response, "Response");
        let method_name = &method.name;
        
        quote! {
            pub async fn #name(&self, request: #request_type) -> Result<#response_type> {
                self.inner.call(#method_name, request).await
            }
        }
    }
    
    fn generate_client_stream(&self, stream: &Stream, _type_registry: &TypeRegistry) -> TokenStream {
        let name = format_ident!("{}", stream.name.to_case(Case::Snake));
        let request_type = self.method_type_name(&stream.request, "Request");
        let response_type = self.method_type_name(&stream.response, "Response");
        let stream_name = &stream.name;
        
        quote! {
            pub async fn #name(
                &self,
                request: #request_type
            ) -> Result<Box<dyn futures_util::Stream<Item = Result<#response_type>> + Send + Unpin>> {
                self.inner.stream(#stream_name, request).await
            }
        }
    }
    
    fn method_type_name(&self, message: &Option<Message>, suffix: &str) -> TokenStream {
        if let Some(msg) = message {
            if msg.name.starts_with("_inline_") {
                // インライン型を生成
                let fields: Vec<_> = msg.fields.iter()
                    .map(|f| {
                        let name = format_ident!("{}", f.name);
                        let ty = self.field_type_to_rust(&f.field_type(), &TypeRegistry::new());
                        quote! { pub #name: #ty }
                    })
                    .collect();

                quote! {
                    {
                        #[derive(Debug, Clone, Serialize, Deserialize)]
                        struct #suffix {
                            #(#fields),*
                        }
                        #suffix
                    }
                }
            } else {
                let ident = format_ident!("{}", msg.name);
                quote! { #ident }
            }
        } else {
            quote! { () }
        }
    }

    fn format_code(&self, code: &str) -> String {
        // 基本的なフォーマット - 本番環境ではrustfmtを使用
        code.replace(" ;", ";")
            .replace("  ", " ")
            .replace("{ ", "{\n    ")
            .replace(" }", "\n}")
            .replace(", ", ",\n    ")
    }
}