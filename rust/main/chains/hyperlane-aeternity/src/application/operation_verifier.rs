use std::io::Cursor;

use async_trait::async_trait;
use derive_new::new;
use tracing::trace;

use hyperlane_core::{Decode, HyperlaneMessage, H256, U256};
use hyperlane_operation_verifier::{
    ApplicationOperationVerifier, ApplicationOperationVerifierReport,
};
use hyperlane_warp_route::TokenMessage;
use ApplicationOperationVerifierReport::MalformedMessage;

const WARP_ROUTE_MARKER: &str = "/";

/// 536M AE * 10^18 (max practical supply in aettos)
fn max_ae_amount() -> U256 {
    U256::from(536_000_000u64) * U256::from(10u64).pow(U256::from(18))
}

/// Application operation verifier for Aeternity
#[derive(new)]
pub struct AeApplicationOperationVerifier {}

#[async_trait]
impl ApplicationOperationVerifier for AeApplicationOperationVerifier {
    async fn verify(
        &self,
        app_context: &Option<String>,
        message: &HyperlaneMessage,
    ) -> Option<ApplicationOperationVerifierReport> {
        trace!(
            ?app_context,
            ?message,
            "Aeternity application operation verifier",
        );

        Self::verify_message(app_context, message)
    }
}

impl AeApplicationOperationVerifier {
    fn verify_message(
        app_context: &Option<String>,
        message: &HyperlaneMessage,
    ) -> Option<ApplicationOperationVerifierReport> {
        let context = match app_context {
            Some(c) => c,
            None => return None,
        };

        if !context.contains(WARP_ROUTE_MARKER) {
            return None;
        }

        // Reject warp route messages with zero recipient (Mailbox rejects on-chain)
        if message.recipient == H256::zero() {
            return Some(MalformedMessage(message.clone()));
        }

        let mut reader = Cursor::new(message.body.as_slice());
        let token_message = match TokenMessage::read_from(&mut reader) {
            Ok(m) => m,
            Err(_) => return Some(MalformedMessage(message.clone())),
        };

        if token_message.amount() > max_ae_amount() {
            return Some(MalformedMessage(message.clone()));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use super::*;
    use hyperlane_core::Encode;
    use hyperlane_core::H256;

    fn encode(token_message: TokenMessage) -> Vec<u8> {
        let mut encoded = vec![];
        token_message.write_to(&mut encoded).unwrap();
        encoded
    }

    #[test]
    fn test_amount_not_too_big() {
        let app_context = Some("H/warp-route".to_string());
        let amount = max_ae_amount();

        let token_message = TokenMessage::new(H256::repeat_byte(0x01), amount, vec![]);
        let encoded = encode(token_message);
        let message = HyperlaneMessage {
            body: encoded,
            recipient: H256::repeat_byte(0x01),
            ..Default::default()
        };

        let report = AeApplicationOperationVerifier::verify_message(&app_context, &message);
        assert_eq!(report, None);
    }

    #[test]
    fn test_amount_too_big() {
        let app_context = Some("H/warp-route".to_string());
        let amount = max_ae_amount().add(1);

        let token_message = TokenMessage::new(H256::zero(), amount, vec![]);
        let encoded = encode(token_message);
        let message = HyperlaneMessage {
            body: encoded,
            ..Default::default()
        };

        let report = AeApplicationOperationVerifier::verify_message(&app_context, &message);
        assert_eq!(report.unwrap(), MalformedMessage(message));
    }

    #[test]
    fn test_malformed_body_rejected() {
        let app_context = Some("H/warp-route".to_string());
        let message = HyperlaneMessage {
            body: vec![0xFF, 0xFE],
            ..Default::default()
        };

        let report = AeApplicationOperationVerifier::verify_message(&app_context, &message);
        assert_eq!(report.unwrap(), MalformedMessage(message));
    }

    #[test]
    fn test_non_warp_route_skips() {
        let app_context = Some("plain-context".to_string());
        let message = HyperlaneMessage::default();

        let report = AeApplicationOperationVerifier::verify_message(&app_context, &message);
        assert_eq!(report, None);
    }

    #[test]
    fn test_no_context_skips() {
        let message = HyperlaneMessage::default();
        let report = AeApplicationOperationVerifier::verify_message(&None, &message);
        assert_eq!(report, None);
    }

    #[test]
    fn test_zero_recipient_rejected_in_warp_context() {
        let app_context = Some("H/warp-route".to_string());
        let message = HyperlaneMessage {
            recipient: H256::zero(),
            body: vec![0u8; 64],
            ..Default::default()
        };
        let report = AeApplicationOperationVerifier::verify_message(&app_context, &message);
        assert_eq!(report.unwrap(), MalformedMessage(message));
    }

    #[test]
    fn test_nonzero_recipient_passes_warp_context() {
        let app_context = Some("H/warp-route".to_string());
        let amount = U256::from(100u64);
        let token_message = TokenMessage::new(H256::repeat_byte(0xAA), amount, vec![]);
        let encoded = encode(token_message);
        let message = HyperlaneMessage {
            recipient: H256::repeat_byte(0x01),
            body: encoded,
            ..Default::default()
        };
        let report = AeApplicationOperationVerifier::verify_message(&app_context, &message);
        assert_eq!(report, None);
    }
}
