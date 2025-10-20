use rcgen;
use std::fs;
use std::path::Path;

/// Generate development certificates for embedding at build time
pub fn generate_dev_certs() -> Result<(), Box<dyn std::error::Error>> {
    let certs_dir = Path::new("assets/certs");

    // Create assets/certs directory if it doesn't exist
    fs::create_dir_all(certs_dir)?;

    // Skip generation if certificates already exist
    if certs_dir.join("cert.pem").exists() && certs_dir.join("private_key.der").exists() {
        println!("cargo:warning=Development certificates already exist, skipping generation");
        return Ok(());
    }

    println!("cargo:warning=Generating development certificates for embedding...");

    // Generate simple self-signed certificate
    let subject_alt_names = vec![
        "localhost".to_string(),
        "*.unison.svc.cluster.local".to_string(),
        "dev.chronista.club".to_string(),
    ];

    let cert_key = rcgen::generate_simple_self_signed(subject_alt_names)?;

    // Save certificate in PEM format
    let cert_pem = cert_key.cert.pem();
    fs::write(certs_dir.join("cert.pem"), cert_pem)?;

    // Save private key in DER format (Rust/quinn optimized)
    let private_key_der = cert_key.key_pair.serialize_der();
    fs::write(certs_dir.join("private_key.der"), private_key_der)?;

    // Save private key in PEM format (for compatibility)
    let private_key_pem = cert_key.key_pair.serialize_pem();
    fs::write(certs_dir.join("private_key.pem"), private_key_pem)?;

    // Save certificate in DER format (for compatibility)
    let cert_der = cert_key.cert.der();
    fs::write(certs_dir.join("cert.der"), cert_der)?;

    println!("cargo:warning=Development certificates generated successfully");
    println!("cargo:rerun-if-changed=build_certs.rs");
    println!("cargo:rerun-if-changed=assets/certs/cert.pem");
    println!("cargo:rerun-if-changed=assets/certs/private_key.der");

    Ok(())
}
