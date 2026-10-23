#[cfg(test)]
mod tests {
    use super::super::coauthor_attribution_verifier::{
        format_coauthor_trailer, parse_coauthor_trailers, CoAuthor,
    };

    #[test]
    fn test_format_coauthor_trailer_valid() {
        let trailer = format_coauthor_trailer("Alice Smith", "alice@example.com").unwrap();
        assert_eq!(trailer, "Co-authored-by: Alice Smith <alice@example.com>");
    }

    #[test]
    fn test_format_coauthor_trailer_invalid_email() {
        let err = format_coauthor_trailer("Alice Smith", "invalid-email").unwrap_err();
        assert!(err.contains("email address format"));
    }

    #[test]
    fn test_format_coauthor_trailer_empty_name() {
        let err = format_coauthor_trailer("", "alice@example.com").unwrap_err();
        assert!(err.contains("name cannot be empty"));
    }

    #[test]
    fn test_parse_coauthor_trailers_valid() {
        let msg = r#"Feature: proxy - add dynamic route balancer

Co-authored-by: Bob Jones <bob@example.com>
Co-authored-by: Carol White <carol@example.org>
"#;
        let report = parse_coauthor_trailers(msg);
        assert!(report.has_coauthors);
        assert_eq!(report.coauthors.len(), 2);
        assert_eq!(report.coauthors[0].name, "Bob Jones");
        assert_eq!(report.coauthors[0].email, "bob@example.com");
        assert!(report.invalid_trailers.is_empty());
    }

    #[test]
    fn test_parse_coauthor_trailers_malformed() {
        let msg = "Feature: something\n\nCo-authored-by: Bob Jones without email";
        let report = parse_coauthor_trailers(msg);
        assert!(!report.has_coauthors);
        assert_eq!(report.invalid_trailers.len(), 1);
    }
}
