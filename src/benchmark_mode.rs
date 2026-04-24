use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkTelemetry {
    pub terminal_state: &'static str,
    pub response: String,
    pub token_count: u64,
    pub tool_calls: u32,
    pub elapsed_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_telemetry_serializes() {
        let item = BenchmarkTelemetry {
            terminal_state: "success",
            response: "ok".to_string(),
            token_count: 10,
            tool_calls: 0,
            elapsed_ms: 123,
        };
        let json = serde_json::to_string(&item).expect("serialize telemetry");
        assert!(json.contains("\"terminal_state\":\"success\""));
        assert!(json.contains("\"elapsed_ms\":123"));
    }
}
