use crate::MfrError;
use hickory_resolver::{TokioAsyncResolver, config::*};

/// Verify DNS TXT record for domain ownership.
/// Checks that `_actrix-verify.<domain>` has a TXT record containing `expected_token`.
pub async fn verify_txt_record(domain: &str, expected_token: &str) -> Result<bool, MfrError> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let host = format!("_actrix-verify.{}.", domain);
    match resolver.txt_lookup(&host).await {
        Ok(records) => {
            for record in records.iter() {
                let txt = record.to_string();
                if txt.contains(expected_token) {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        Err(e) => {
            tracing::warn!(domain = %domain, error = %e, "DNS TXT lookup failed");
            Err(MfrError::Dns(e.to_string()))
        }
    }
}
