use crate::server::app;
use crate::test_utils::post_and_read;
use axum::http::StatusCode;
use num_bigint::BigInt;
use num_traits::Signed;

#[tokio::test]
async fn v1_data_usdg_total_rewards() {
    let app = app();
    let body = serde_json::json!({
        "solarFarms": []
    });

    let (status, text) =
        post_and_read(app, "/api/rewards-simulator?preloadGlowV1=true", body).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "API call failed with body: {}",
        text
    );

    let output: serde_json::Value = serde_json::from_str(&text).expect("Failed to parse JSON");
    let weekly_outputs = output.as_object().expect("Expected JSON object");

    let mut total_usdg: BigInt = BigInt::from(0);

    for (_week, week_data) in weekly_outputs {
        if let Some(wallet_dists) = week_data
            .get("walletDistributions")
            .and_then(|v| v.as_array())
        {
            for dist in wallet_dists {
                if let Some(assets_earned) = dist.get("assetsEarned").and_then(|v| v.as_object()) {
                    // v1 data uses "USDG"
                    if let Some(usdg_val) = assets_earned.get("USDG") {
                        if let Some(usdg_str) = usdg_val.as_str() {
                            let usdg_amount: BigInt =
                                usdg_str.parse().expect("Failed to parse USDG amount");
                            total_usdg += usdg_amount;
                        }
                    }
                }
            }
        }
    }

    let expected_total_usdg: BigInt = "20369901143593".parse().unwrap();
    let tolerance = BigInt::from(1_000_000i64);
    let diff = (&total_usdg - &expected_total_usdg).abs();
    assert!(
        diff <= tolerance,
        "Total USDG rewards mismatch: got {}, expected {}, diff is {}, which is more than tolerance {}",
        total_usdg,
        expected_total_usdg,
        diff,
        tolerance
    );
}
