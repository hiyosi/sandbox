use glob::glob;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protos = vec![];

    let patterns = [
      "proto/spire-api-sdk/proto/spire/api/types/**/*.proto",
      "proto/spire-api-sdk/proto/spire/api/server/**/*.proto",
    ];

    for pattern in &patterns {
        for entry in glob(pattern)? {
            protos.push(entry?.to_string_lossy().to_string());
        }
    }

    for entry in glob("proto/spire-api-sdk/proto/spire/api/types/*.proto")? {
        protos.push(entry?.to_string_lossy().to_string());
    }

    tonic_build::configure()
        .build_server(false)
        .compile_protos(&protos, &["proto/spire-api-sdk/proto"])?;
    Ok(())
  }
