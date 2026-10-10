use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    pub(crate) fn test_csv_line_parser() {
        let line = "My Alias,user@example.com,smtp.example.com,587";
        let fields = parse_csv_line(line);
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0], "My Alias");
        assert_eq!(fields[1], "user@example.com");
    }

    #[test]
    pub(crate) fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("simple"), "simple");
        assert_eq!(escape_csv_field("has,comma"), "\"has,comma\"");
    }

    #[test]
    pub(crate) fn test_unescape_xml() {
        assert_eq!(unescape_xml("a &amp; b &lt; c &gt; d"), "a & b < c > d");
    }

    #[test]
    pub(crate) fn test_parse_xml_row() {
        let row = r#"   <Row>
    <Cell><Data ss:Type="String">Alias &amp; Name</Data></Cell>
    <Cell><Data ss:Type="String">test@example.com</Data></Cell>
    <Cell><Data ss:Type="Number">587</Data></Cell>
   </Row>"#;
        let cells = parse_xml_row(row);
        assert_eq!(cells.len(), 3);
        assert_eq!(cells[0], "Alias & Name");
        assert_eq!(cells[1], "test@example.com");
        assert_eq!(cells[2], "587");
    }

    #[test]
    pub(crate) fn test_base64_multi_pass_roundtrip() {
        let secret = "super-secret-password-123!@#$%^&*()";
        let encoded_3 = base64_encode_multi(secret, 3);
        assert_ne!(encoded_3, secret);
        let decoded = base64_decode_multi(&encoded_3, 3).expect("3-pass decode failed");
        assert_eq!(decoded, secret);

        let encoded_1 = base64_encode_multi(secret, 1);
        let decoded_1 = base64_decode_multi(&encoded_1, 1).expect("1-pass decode failed");
        assert_eq!(decoded_1, secret);
    }

    #[test]
    pub(crate) fn test_base64_multi_pass_invalid_input() {
        let result = base64_decode_multi("not-valid-base64!!!", 1);
        assert!(result.is_err());
    }
}
