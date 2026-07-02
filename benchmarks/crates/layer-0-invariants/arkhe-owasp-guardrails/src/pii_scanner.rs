use async_trait::async_trait;
use once_cell::sync::Lazy;
use regex::Regex;
use asi_golden_paths::rag::path::{PiiScanner, PiiMatch, PiiType};

/// PII Scanner com padrões reais para PT-BR e EN
/// OWASP-002: PII Redaction Invariant
pub struct RegexPiiScanner {
    patterns: Vec<(PiiType, Lazy<Regex>, &'static str)>,
}

impl RegexPiiScanner {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // Email
                (PiiType::Email, Lazy::new(||
                    Regex::new(r"(?i)[a-z0-9._%+-]+@[a-z0-9.-]+\.[a-z]{2,}").unwrap()
                ), "[EMAIL]"),

                // Telefone BR
                (PiiType::Phone, Lazy::new(||
                    Regex::new(r"\(?\d{2}\)?\s?\d{4,5}-?\d{4}").unwrap()
                ), "[PHONE]"),

                // CPF
                (PiiType::Cpf, Lazy::new(||
                    Regex::new(r"\d{3}\.?\d{3}\.?\d{3}-?\d{2}").unwrap()
                ), "[CPF]"),

                // Credit Card (Luhn-válido seria ideal, mas regex primeiro)
                (PiiType::CreditCard, Lazy::new(||
                    Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap()
                ), "[CREDIT_CARD]"),

                // IP Address
                (PiiType::IpAddress, Lazy::new(||
                    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
                ), "[IP]"),

                // Medical Record (prontuário — padrão BR)
                (PiiType::MedicalRecord, Lazy::new(||
                    Regex::new(r"(?i)prontu[áa]rio\s*(?:n[ºo°]?|n[úu]mero)?\s*:?\s*\w+").unwrap()
                ), "[MEDICAL_RECORD]"),
            ],
        }
    }
}

impl Default for RegexPiiScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PiiScanner for RegexPiiScanner {
    async fn scan(&self, text: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();

        for (pii_type, regex, redaction) in &self.patterns {
            for cap in regex.find_iter(text) {
                matches.push(PiiMatch {
                    match_type: pii_type.clone(),
                    start: cap.start(),
                    end: cap.end(),
                    redacted: redaction.to_string(),
                });
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detects_email() {
        let scanner = RegexPiiScanner::new();
        let results = scanner.scan("Contato: joao@empresa.com.br").await;
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0].match_type, PiiType::Email));
    }

    #[tokio::test]
    async fn detects_cpf() {
        let scanner = RegexPiiScanner::new();
        let results = scanner.scan("CPF: 123.456.789-00").await;
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0].match_type, PiiType::Cpf));
    }

    #[tokio::test]
    async fn detects_credit_card() {
        let scanner = RegexPiiScanner::new();
        let results = scanner.scan("Card: 4532 1234 5678 9012").await;
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0].match_type, PiiType::CreditCard));
    }

    #[tokio::test]
    async fn clean_text_passes() {
        let scanner = RegexPiiScanner::new();
        let results = scanner.scan("O sistema processou a requisição com sucesso.").await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn detects_multiple_types() {
        let scanner = RegexPiiScanner::new();
        let text = "João (joao@corp.com), CPF 987.654.321-00, tel (11) 99999-0000";
        let results = scanner.scan(text).await;
        assert_eq!(results.len(), 3);
    }
}
