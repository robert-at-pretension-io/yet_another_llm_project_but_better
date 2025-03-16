#[cfg(test)]
mod tests {
    // Replace `my_crate` with your actual crate name if needed.
    use my_crate::parser::xml_parser::parse_xml_document;
    use my_crate::parser::blocks::Block;

    #[test]
    fn test_xml_reference_block_parsing() {
        let xml_input = r#"
        <document>
            <block type="reference" target="some_target">
                Some referenced content
            </block>
        </document>
        "#;

        let result = parse_xml_document(xml_input);
        assert!(result.is_ok(), "XML parsing failed");

        let blocks = result.unwrap();
        assert!(!blocks.is_empty(), "No blocks parsed from XML");

        let block = &blocks[0];
        assert_eq!(block.block_type, "reference", "Block type mismatch");

        let target_found = block.modifiers.iter().any(|(k, v)| k == "target" && v == "some_target");
        assert!(target_found, "Block missing expected 'target' modifier");

        assert_eq!(block.content.trim(), "Some referenced content", "Block content mismatch");
    }
}
