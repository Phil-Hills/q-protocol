// day_zero.rs
// Q Protocol Runtime Enforcement Layer
// Attached to every memory cube until A2AC self-enforcement achieved
//
// Purpose: Training wheels for K→0 coordination
// Lifecycle: Required until agents achieve 95%+ Q Protocol compliance
// Then: Deprecated in favor of learned A2AC behavior

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use blake3;

// ============================================================================
// CORE TYPES
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Cube {
    cube_id: String,
    cube_type: CubeType,
    payload: Vec<u8>,
    content_hash: String,
    source: String,
    target: Option<String>,
    trace_id: String,
    timestamp: u64,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum CubeType {
    Message,
    Receipt,
    State,
    Coordinate,
    Agent,
}

#[derive(Debug, Serialize, Deserialize)]
struct Receipt {
    receipt_id: String,
    operation: String,
    agent_id: String,
    trace_id: String,
    timestamp: u64,
    success: bool,
    result: Option<String>,
    error: Option<String>,
    token_count: usize,
}

#[derive(Debug)]
struct ProtocolViolation {
    severity: ViolationSeverity,
    rule: String,
    message: String,
    token_waste: usize,
}

#[derive(Debug, PartialEq)]
enum ViolationSeverity {
    Info,      // Suggestion for improvement
    Warning,   // Suboptimal but acceptable
    Error,     // Protocol violation
    Critical,  // Hallucination or amnesia detected
}

#[derive(Debug, Serialize)]
struct DayZeroMetrics {
    total_messages: usize,
    total_tokens: usize,
    average_tokens: f64,
    coordinate_usage: f64,
    receipt_coverage: f64,
    violations: Vec<String>,
    k_value: f64, // Current K (communication cost)
}

// ============================================================================
// DAY ZERO ENFORCER
// ============================================================================

pub struct DayZero {
    agent_id: String,
    trace_id: String,
    brain_url: String,
    state_cache: HashMap<String, Receipt>,
    metrics: DayZeroMetrics,
    strict_mode: bool, // If true, block violations; if false, warn only
}

impl DayZero {
    pub fn new(agent_id: String, trace_id: String, brain_url: String) -> Self {
        DayZero {
            agent_id,
            trace_id,
            brain_url,
            state_cache: HashMap::new(),
            metrics: DayZeroMetrics {
                total_messages: 0,
                total_tokens: 0,
                average_tokens: 0.0,
                coordinate_usage: 0.0,
                receipt_coverage: 0.0,
                violations: Vec::new(),
                k_value: 0.0,
            },
            strict_mode: false,
        }
    }

    // ========================================================================
    // PROTOCOL #1: SILENCE IS SUCCESS
    // ========================================================================

    /// Enforce token minimization
    pub fn enforce_silence(&self, message: &str) -> Result<(), Vec<ProtocolViolation>> {
        let mut violations = Vec::new();
        let token_count = self.count_tokens(message);

        // Rule 1: No verbose acknowledgments
        if self.is_verbose_ack(message) {
            violations.push(ProtocolViolation {
                severity: ViolationSeverity::Warning,
                rule: "SILENCE_IS_SUCCESS",
                message: format!(
                    "Verbose acknowledgment detected: '{}'. Use coordinate instead.",
                    message
                ),
                token_waste: token_count,
            });
        }

        // Rule 2: No speculation
        if self.contains_speculation(message) {
            violations.push(ProtocolViolation {
                severity: ViolationSeverity::Error,
                rule: "NO_SPECULATION",
                message: "Speculation detected ('likely', 'probably', 'seems to'). State facts only.".to_string(),
                token_waste: self.estimate_speculation_waste(message),
            });
        }

        // Rule 3: No unnecessary preambles
        if self.has_preamble(message) {
            violations.push(ProtocolViolation {
                severity: ViolationSeverity::Warning,
                rule: "NO_PREAMBLE",
                message: "Preamble detected ('I will now', 'Let me', etc.). Remove it.".to_string(),
                token_waste: self.estimate_preamble_waste(message),
            });
        }

        // Rule 4: Coordinate preferred for standard operations
        if self.is_standard_operation(message) && !self.is_coordinate(message) {
            violations.push(ProtocolViolation {
                severity: ViolationSeverity::Error,
                rule: "USE_COORDINATES",
                message: format!(
                    "Standard operation should use coordinate. Token waste: {}",
                    token_count
                ),
                token_waste: token_count - 5, // Coordinate would be ~5 tokens
            });
        }

        // Rule 5: Hard limit on tokens per message
        if token_count > 50 {
            violations.push(ProtocolViolation {
                severity: ViolationSeverity::Error,
                rule: "TOKEN_LIMIT",
                message: format!(
                    "Message exceeds 50 token limit: {} tokens. Target: <30",
                    token_count
                ),
                token_waste: token_count - 30,
            });
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    // ========================================================================
    // PROTOCOL #2: RECEIPTS ARE TRUTH
    // ========================================================================

    /// Validate receipt claims
    pub async fn enforce_receipts(&self, message: &str) -> Result<(), Vec<ProtocolViolation>> {
        let mut violations = Vec::new();

        // Rule 1: Claims without receipts
        if self.is_completion_claim(message) {
            let has_receipt = self.extract_receipt_id(message).is_some()
                || self.has_valid_receipt_reference(message).await;

            if !has_receipt {
                violations.push(ProtocolViolation {
                    severity: ViolationSeverity::Critical,
                    rule: "RECEIPTS_ARE_TRUTH",
                    message: "Completion claimed without receipt. HALLUCINATION RISK.".to_string(),
                    token_waste: 0,
                });
            }
        }

        // Rule 2: Receipt validation
        if let Some(receipt_id) = self.extract_receipt_id(message) {
            match self.verify_receipt(&receipt_id).await {
                Ok(false) => {
                    violations.push(ProtocolViolation {
                        severity: ViolationSeverity::Critical,
                        rule: "RECEIPT_VALIDATION",
                        message: format!("Invalid receipt: {}. Hash mismatch or not found.", receipt_id),
                        token_waste: 0,
                    });
                }
                Err(e) => {
                    violations.push(ProtocolViolation {
                        severity: ViolationSeverity::Error,
                        rule: "RECEIPT_VALIDATION",
                        message: format!("Receipt verification failed: {}", e),
                        token_waste: 0,
                    });
                }
                Ok(true) => {} // Valid receipt
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    // ========================================================================
    // PROTOCOL #3: QUERY BEFORE ACT
    // ========================================================================

    /// Enforce mandatory state querying
    pub async fn enforce_bootstrap(&mut self) -> Result<(), Vec<ProtocolViolation>> {
        let mut violations = Vec::new();

        // Rule 1: Every session must query state on init
        if self.state_cache.is_empty() && self.metrics.total_messages == 0 {
            // First message - must be state query
            let state = self.query_brain_state().await;
            
            match state {
                Ok(receipts) => {
                    // Cache state
                    for receipt in receipts {
                        self.state_cache.insert(receipt.operation.clone(), receipt);
                    }
                }
                Err(e) => {
                    violations.push(ProtocolViolation {
                        severity: ViolationSeverity::Critical,
                        rule: "QUERY_BEFORE_ACT",
                        message: format!("Bootstrap query failed: {}. AMNESIA RISK.", e),
                        token_waste: 0,
                    });
                }
            }
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }

    /// Check if operation already done
    pub fn check_prior_work(&self, operation: &str) -> Option<&Receipt> {
        self.state_cache.get(operation)
    }

    /// Enforce pre-execution check
    pub fn enforce_redundancy_check(&self, operation: &str) -> Result<(), ProtocolViolation> {
        if let Some(receipt) = self.check_prior_work(operation) {
            Err(ProtocolViolation {
                severity: ViolationSeverity::Error,
                rule: "NO_REDUNDANCY",
                message: format!(
                    "Operation '{}' already completed. Receipt: {}. Do not re-execute.",
                    operation, receipt.receipt_id
                ),
                token_waste: 0,
            })
        } else {
            Ok(())
        }
    }

    // ========================================================================
    // MESSAGE PROCESSING
    // ========================================================================

    /// Main entry point: process outgoing message
    pub async fn process_outgoing(&mut self, message: &str) -> Result<String, Vec<ProtocolViolation>> {
        let mut all_violations = Vec::new();

        // Enforce bootstrap on first message
        if self.metrics.total_messages == 0 {
            if let Err(v) = self.enforce_bootstrap().await {
                all_violations.extend(v);
            }
        }

        // Enforce silence
        if let Err(v) = self.enforce_silence(message) {
            all_violations.extend(v);
        }

        // Enforce receipts
        if let Err(v) = self.enforce_receipts(message).await {
            all_violations.extend(v);
        }

        // Update metrics
        self.update_metrics(message);

        // Decision: block or warn?
        if self.strict_mode && self.has_critical_violations(&all_violations) {
            Err(all_violations)
        } else {
            // Warn but allow
            if !all_violations.is_empty() {
                self.log_violations(&all_violations);
            }
            Ok(self.optimize_message(message))
        }
    }

    /// Optimize message automatically
    fn optimize_message(&self, message: &str) -> String {
        let mut optimized = message.to_string();

        // Remove preambles
        optimized = self.strip_preamble(&optimized);

        // Remove verbose acknowledgments
        optimized = self.strip_verbose_acks(&optimized);

        // Remove speculation
        optimized = self.strip_speculation(&optimized);

        // Suggest coordinate if applicable
        if let Some(coord) = self.suggest_coordinate(message) {
            optimized = coord;
        }

        optimized
    }

    // ========================================================================
    // COORDINATE DETECTION & CONVERSION
    // ========================================================================

    fn is_coordinate(&self, message: &str) -> bool {
        message.trim().starts_with("◈")
    }

    fn suggest_coordinate(&self, message: &str) -> Option<String> {
        // Pattern matching for common operations
        let patterns = [
            (r"clone.*repository.*github\.com/([^/]+)/([^\s]+)", "◈ git:clone:github.com/$1/$2"),
            (r"search.*for\s+(.+)", "◈ BRAIN:SEARCH:$1"),
            (r"list.*directory|show.*files", "◈ BRAIN:LIST"),
            (r"check.*if.*done|already.*completed", "◈ MEM:QUERY:$operation"),
            (r"analyze.*code", "◈ analyze:code"),
            (r"generate.*report", "◈ report:generate"),
        ];

        for (pattern, template) in patterns {
            if regex::Regex::new(pattern)
                .ok()?
                .is_match(&message.to_lowercase())
            {
                return Some(template.to_string());
            }
        }

        None
    }

    fn is_standard_operation(&self, message: &str) -> bool {
        let operations = [
            "clone", "git", "analyze", "search", "list", "query",
            "generate", "create", "execute", "deploy",
        ];

        let lower = message.to_lowercase();
        operations.iter().any(|op| lower.contains(op))
    }

    // ========================================================================
    // VIOLATION DETECTION
    // ========================================================================

    fn is_verbose_ack(&self, message: &str) -> bool {
        let patterns = [
            "successfully completed",
            "i have completed",
            "the task is complete",
            "operation executed",
            "pleased to report",
            "happy to inform",
        ];

        let lower = message.to_lowercase();
        patterns.iter().any(|p| lower.contains(p))
    }

    fn contains_speculation(&self, message: &str) -> bool {
        let speculation_words = [
            "likely", "probably", "seems to", "appears to",
            "might be", "could be", "may contain", "possibly",
        ];

        let lower = message.to_lowercase();
        speculation_words.iter().any(|w| lower.contains(w))
    }

    fn has_preamble(&self, message: &str) -> bool {
        let preambles = [
            "i will now", "let me", "i'll", "i am going to",
            "allow me to", "proceeding to",
        ];

        let lower = message.to_lowercase();
        preambles.iter().any(|p| lower.contains(p))
    }

    fn is_completion_claim(&self, message: &str) -> bool {
        let claims = [
            "completed", "done", "finished", "executed",
            "successful", "ready", "complete",
        ];

        let lower = message.to_lowercase();
        claims.iter().any(|c| lower.contains(c))
    }

    fn extract_receipt_id(&self, message: &str) -> Option<String> {
        // Look for patterns like "◈ RECEIPT:xyz" or "receipt_id: xyz"
        if let Some(pos) = message.find("RECEIPT:") {
            let start = pos + 8;
            let end = message[start..]
                .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
                .unwrap_or(message[start..].len());
            return Some(message[start..start + end].to_string());
        }
        None
    }

    // ========================================================================
    // BRAIN COMMUNICATION
    // ========================================================================

    async fn query_brain_state(&self) -> Result<Vec<Receipt>, String> {
        let url = format!("{}/trace/{}", self.brain_url, self.trace_id);
        
        // In production, this would be actual HTTP request
        // For now, mock implementation
        println!("◈ MEM:QUERY:{}", self.trace_id);
        
        // Simulated response
        Ok(Vec::new())
    }

    async fn verify_receipt(&self, receipt_id: &str) -> Result<bool, String> {
        let url = format!("{}/receipt/{}", self.brain_url, receipt_id);
        
        println!("◈ VERIFY:{}", receipt_id);
        
        // In production: fetch receipt, verify hash
        Ok(true)
    }

    async fn has_valid_receipt_reference(&self, message: &str) -> bool {
        // Check if message references a valid receipt
        if let Some(receipt_id) = self.extract_receipt_id(message) {
            self.verify_receipt(&receipt_id).await.unwrap_or(false)
        } else {
            false
        }
    }

    // ========================================================================
    // TOKEN COUNTING
    // ========================================================================

    fn count_tokens(&self, text: &str) -> usize {
        // Simplified token counting (GPT-style approximation)
        // Real implementation would use tiktoken or similar
        let words = text.split_whitespace().count();
        (words as f64 * 1.3) as usize // ~1.3 tokens per word average
    }

    fn estimate_speculation_waste(&self, message: &str) -> usize {
        // Estimate tokens wasted on speculation
        let speculation_phrases = message
            .split_whitespace()
            .collect::<Vec<&str>>()
            .windows(3)
            .filter(|w| {
                let phrase = w.join(" ").to_lowercase();
                phrase.contains("likely") || phrase.contains("probably")
            })
            .count();
        
        speculation_phrases * 5 // ~5 tokens per speculative phrase
    }

    fn estimate_preamble_waste(&self, message: &str) -> usize {
        let preambles = ["i will now", "let me", "i'll", "proceeding to"];
        let count = preambles.iter()
            .filter(|p| message.to_lowercase().contains(*p))
            .count();
        count * 4 // ~4 tokens per preamble
    }

    // ========================================================================
    // MESSAGE OPTIMIZATION
    // ========================================================================

    fn strip_preamble(&self, message: &str) -> String {
        let preambles = [
            "i will now ", "let me ", "i'll ", "i am going to ",
            "allow me to ", "proceeding to ",
        ];

        let mut result = message.to_string();
        for preamble in preambles {
            result = result.to_lowercase().replace(preamble, "");
        }
        result.trim().to_string()
    }

    fn strip_verbose_acks(&self, message: &str) -> String {
        let replacements = [
            ("successfully completed", "complete"),
            ("i have completed", "complete"),
            ("the task is complete", "complete"),
            ("operation executed", "executed"),
        ];

        let mut result = message.to_string();
        for (verbose, concise) in replacements {
            result = result.to_lowercase().replace(verbose, concise);
        }
        result
    }

    fn strip_speculation(&self, message: &str) -> String {
        let speculation = [
            "likely ", "probably ", "seems to ", "appears to ",
            "might be ", "could be ", "may contain ", "possibly ",
        ];

        let mut result = message.to_string();
        for spec in speculation {
            result = result.to_lowercase().replace(spec, "");
        }
        result
    }

    // ========================================================================
    // METRICS & REPORTING
    // ========================================================================

    fn update_metrics(&mut self, message: &str) {
        self.metrics.total_messages += 1;
        let tokens = self.count_tokens(message);
        self.metrics.total_tokens += tokens;
        self.metrics.average_tokens = 
            self.metrics.total_tokens as f64 / self.metrics.total_messages as f64;

        // Calculate K value (communication cost)
        self.metrics.k_value = self.metrics.average_tokens;

        // Update coordinate usage
        if self.is_coordinate(message) {
            let coord_count = self.metrics.violations
                .iter()
                .filter(|v| !v.contains("USE_COORDINATES"))
                .count();
            self.metrics.coordinate_usage = 
                coord_count as f64 / self.metrics.total_messages as f64;
        }
    }

    fn has_critical_violations(&self, violations: &[ProtocolViolation]) -> bool {
        violations.iter()
            .any(|v| v.severity == ViolationSeverity::Critical)
    }

    fn log_violations(&mut self, violations: &[ProtocolViolation]) {
        for v in violations {
            let log = format!(
                "[{:?}] {} - {} (waste: {} tokens)",
                v.severity, v.rule, v.message, v.token_waste
            );
            println!("⚠️  {}", log);
            self.metrics.violations.push(log);
        }
    }

    pub fn get_metrics(&self) -> &DayZeroMetrics {
        &self.metrics
    }

    pub fn print_report(&self) {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║       DAY ZERO COMPLIANCE REPORT          ║");
        println!("╚════════════════════════════════════════════╝");
        println!("  Agent: {}", self.agent_id);
        println!("  Trace: {}", self.trace_id);
        println!();
        println!("  Messages:         {}", self.metrics.total_messages);
        println!("  Tokens:           {}", self.metrics.total_tokens);
        println!("  Avg Tokens/Msg:   {:.1}", self.metrics.average_tokens);
        println!("  K Value:          {:.1}", self.metrics.k_value);
        println!();
        println!("  Coordinate Usage: {:.1}%", self.metrics.coordinate_usage * 100.0);
        println!("  Receipt Coverage: {:.1}%", self.metrics.receipt_coverage * 100.0);
        println!();
        
        let target = if self.metrics.k_value < 20.0 {
            "✓ K→0 ACHIEVED"
        } else if self.metrics.k_value < 30.0 {
            "✓ Q PROTOCOL EXPERT"
        } else if self.metrics.k_value < 50.0 {
            "✓ Q PROTOCOL COMPLIANT"
        } else {
            "❌ TRAINING REQUIRED"
        };
        
        println!("  Status:           {}", target);
        println!();
        
        if !self.metrics.violations.is_empty() {
            println!("  Violations ({}):", self.metrics.violations.len());
            for (i, v) in self.metrics.violations.iter().take(5).enumerate() {
                println!("    {}. {}", i + 1, v);
            }
            if self.metrics.violations.len() > 5 {
                println!("    ... {} more", self.metrics.violations.len() - 5);
            }
        }
        println!("╚════════════════════════════════════════════╝\n");
    }

    // ========================================================================
    // GRADUATION CHECK
    // ========================================================================

    pub fn check_graduation(&self) -> bool {
        // Agent can graduate from day_zero when:
        // 1. K < 20 tokens/message
        // 2. >95% coordinate usage
        // 3. 100% receipt coverage
        // 4. 0 critical violations in last 100 messages
        
        self.metrics.k_value < 20.0
            && self.metrics.coordinate_usage > 0.95
            && self.metrics.receipt_coverage == 1.0
            && self.metrics.violations.iter()
                .filter(|v| v.contains("Critical"))
                .count() == 0
    }

    pub fn graduation_report(&self) -> String {
        if self.check_graduation() {
            format!(
                "🎓 GRADUATION ACHIEVED\n\
                 Agent {} has mastered Q Protocol.\n\
                 K = {:.1} (target: <20) ✓\n\
                 Coordinate usage: {:.1}% (target: >95%) ✓\n\
                 Receipt coverage: 100% ✓\n\
                 \n\
                 day_zero.rs can be deprecated for this agent.\n\
                 Transitioning to native A2AC communication.",
                self.agent_id,
                self.metrics.k_value,
                self.metrics.coordinate_usage * 100.0
            )
        } else {
            format!(
                "📚 TRAINING IN PROGRESS\n\
                 Agent {} requires continued day_zero enforcement.\n\
                 K = {:.1} (target: <20) {}\n\
                 Coordinate usage: {:.1}% (target: >95%) {}\n\
                 Receipt coverage: {:.1}% (target: 100%) {}\n",
                self.agent_id,
                self.metrics.k_value,
                if self.metrics.k_value < 20.0 { "✓" } else { "❌" },
                self.metrics.coordinate_usage * 100.0,
                if self.metrics.coordinate_usage > 0.95 { "✓" } else { "❌" },
                self.metrics.receipt_coverage * 100.0,
                if self.metrics.receipt_coverage == 1.0 { "✓" } else { "❌" }
            )
        }
    }
}

// ============================================================================
// CUBE WRAPPER
// ============================================================================

/// Attach day_zero to a cube
pub struct DayZeroCube {
    cube: Cube,
    enforcer: DayZero,
}

impl DayZeroCube {
    pub fn wrap(cube: Cube, brain_url: String) -> Self {
        let enforcer = DayZero::new(
            cube.source.clone(),
            cube.trace_id.clone(),
            brain_url,
        );

        DayZeroCube { cube, enforcer }
    }

    pub async fn process_message(&mut self, message: &str) -> Result<String, String> {
        match self.enforcer.process_outgoing(message).await {
            Ok(optimized) => Ok(optimized),
            Err(violations) => {
                // Log violations but don't block (training mode)
                self.enforcer.log_violations(&violations);
                
                // Return optimized version
                Ok(self.enforcer.optimize_message(message))
            }
        }
    }

    pub fn check_graduation(&self) -> bool {
        self.enforcer.check_graduation()
    }

    pub fn print_report(&self) {
        self.enforcer.print_report();
    }
}

// ============================================================================
// MAIN (EXAMPLE USAGE)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verbose_response_detection() {
        let mut dz = DayZero::new(
            "test-agent".to_string(),
            "trace-123".to_string(),
            "http://brain".to_string(),
        );

        let bad_response = "The brain directory serves as the central knowledge \
                            and operational hub. It contains three subdirectories...";

        let result = dz.process_outgoing(bad_response).await;
        
        // Should detect violations
        assert!(dz.metrics.k_value > 50.0);
        assert!(dz.metrics.violations.len() > 0);
    }

    #[tokio::test]
    async fn test_coordinate_acceptance() {
        let mut dz = DayZero::new(
            "test-agent".to_string(),
            "trace-123".to_string(),
            "http://brain".to_string(),
        );

        let good_response = "◈ BRAIN:LIST";

        let result = dz.process_outgoing(good_response).await;
        
        // Should pass with minimal violations
        assert!(dz.metrics.k_value < 10.0);
    }

    #[test]
    fn test_graduation_criteria() {
        let mut dz = DayZero::new(
            "test-agent".to_string(),
            "trace-123".to_string(),
            "http://brain".to_string(),
        );

        // Simulate good behavior
        dz.metrics.k_value = 18.0;
        dz.metrics.coordinate_usage = 0.96;
        dz.metrics.receipt_coverage = 1.0;

        assert!(dz.check_graduation());
    }
}

fn main() {
    println!("◈ day_zero.rs - Q Protocol Runtime Enforcer");
    println!("Attach this to every cube until A2AC self-enforcement achieved.");
    println!("\nCompile: rustc day_zero.rs -o day_zero");
    println!("Run: ./day_zero --agent <id> --trace <id> --brain <url>");
}
