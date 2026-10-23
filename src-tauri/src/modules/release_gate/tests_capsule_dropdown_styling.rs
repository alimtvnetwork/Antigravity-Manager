#[cfg(test)]
mod tests {
    use super::super::capsule_dropdown_styling_guard::{
        conforming_inline_select_classes, conforming_segmented_capsule_classes,
        conforming_window_controls_capsule_classes, has_dark_glass_styling, has_rounded_full,
        has_shared_border, has_subtle_dividers, lint_ui_markup_for_capsules_and_dropdowns,
        sanitize_dropdown_classes, validate_capsule_with_dropdown,
        validate_inline_select_classes, validate_segmented_capsule_classes,
        CapsuleStylingViolation, DropdownStylingViolation, MANDATORY_DROPDOWN_CLASSES,
    };

    #[test]
    fn test_conforming_classes_are_valid() {
        let capsule = conforming_segmented_capsule_classes();
        let violations = validate_segmented_capsule_classes(capsule, "capsule");
        assert!(violations.is_empty(), "Conforming capsule should have no violations: {:?}", violations);

        let window_capsule = conforming_window_controls_capsule_classes();
        let violations = validate_segmented_capsule_classes(window_capsule, "window_capsule");
        assert!(violations.is_empty(), "Conforming window capsule should have no violations: {:?}", violations);

        let select = conforming_inline_select_classes();
        let violations = validate_inline_select_classes(select, "<select>");
        assert!(violations.is_empty(), "Conforming select should have no violations: {:?}", violations);
    }

    #[test]
    fn test_helper_predicates() {
        assert!(has_rounded_full("rounded-full border divide-x"));
        assert!(!has_rounded_full("rounded-md border divide-x"));

        assert!(has_shared_border("rounded-full border border-white/10 divide-x"));
        assert!(!has_shared_border("rounded-full border-0 divide-x"));

        assert!(has_subtle_dividers("rounded-full border divide-x divide-white/10"));
        assert!(!has_subtle_dividers("rounded-full border shadow-sm"));

        assert!(has_dark_glass_styling("backdrop-blur-md bg-zinc-900/60"));
        assert!(has_dark_glass_styling("backdrop-blur-lg bg-black/50"));
        assert!(!has_dark_glass_styling("bg-white backdrop-blur-md"));
        assert!(!has_dark_glass_styling("bg-zinc-900"));
    }

    #[test]
    fn test_capsule_missing_rounded_full() {
        let classes = "inline-flex border border-white/10 bg-zinc-900/60 backdrop-blur-md divide-x divide-white/10";
        let violations = validate_segmented_capsule_classes(classes, "capsule");
        assert!(violations.iter().any(|v| matches!(v, CapsuleStylingViolation::MissingRoundedFull { .. })));
    }

    #[test]
    fn test_capsule_missing_shared_border() {
        let classes = "inline-flex rounded-full bg-zinc-900/60 backdrop-blur-md divide-x divide-white/10";
        let violations = validate_segmented_capsule_classes(classes, "capsule");
        assert!(violations.iter().any(|v| matches!(v, CapsuleStylingViolation::MissingSharedBorder { .. })));
    }

    #[test]
    fn test_capsule_missing_subtle_dividers() {
        let classes = "inline-flex rounded-full border border-white/10 bg-zinc-900/60 backdrop-blur-md";
        let violations = validate_segmented_capsule_classes(classes, "capsule");
        assert!(violations.iter().any(|v| matches!(v, CapsuleStylingViolation::MissingSubtleDividers { .. })));
    }

    #[test]
    fn test_capsule_missing_dark_glass() {
        let classes = "inline-flex rounded-full border border-white/10 divide-x divide-white/10";
        let violations = validate_segmented_capsule_classes(classes, "capsule");
        assert!(violations.iter().any(|v| matches!(v, CapsuleStylingViolation::MissingDarkGlassStyling { .. })));
    }

    #[test]
    fn test_dropdown_missing_mandatory_classes() {
        let classes = "text-xs text-white";
        let violations = validate_inline_select_classes(classes, "<select>");
        assert_eq!(violations.len(), 4);
        assert!(violations.iter().any(|v| matches!(v, DropdownStylingViolation::MissingTransparentBackground { .. })));
        assert!(violations.iter().any(|v| matches!(v, DropdownStylingViolation::MissingBorderZero { .. })));
        assert!(violations.iter().any(|v| matches!(v, DropdownStylingViolation::MissingFocusRingZero { .. })));
        assert!(violations.iter().any(|v| matches!(v, DropdownStylingViolation::MissingFocusOutlineNone { .. })));
    }

    #[test]
    fn test_dropdown_nested_pill_defect() {
        let classes = "bg-transparent border-0 focus:ring-0 focus:outline-none rounded-full bg-white";
        let violations = validate_inline_select_classes(classes, "<select>");
        assert!(violations.iter().any(|v| matches!(v, DropdownStylingViolation::NestedPillVisualDefect { offending_class, .. } if offending_class == "rounded-full")));
        assert!(violations.iter().any(|v| matches!(v, DropdownStylingViolation::NestedPillVisualDefect { offending_class, .. } if offending_class == "bg-white")));
    }

    #[test]
    fn test_mandatory_constants() {
        assert_eq!(MANDATORY_DROPDOWN_CLASSES.len(), 4);
        assert!(MANDATORY_DROPDOWN_CLASSES.contains(&"bg-transparent"));
        assert!(MANDATORY_DROPDOWN_CLASSES.contains(&"border-0"));
        assert!(MANDATORY_DROPDOWN_CLASSES.contains(&"focus:ring-0"));
        assert!(MANDATORY_DROPDOWN_CLASSES.contains(&"focus:outline-none"));
    }

    #[test]
    fn test_sanitize_dropdown_classes() {
        let dirty = "bg-white rounded-full border-2 ring-2 text-xs text-zinc-300 py-1";
        let cleaned = sanitize_dropdown_classes(dirty);
        let tokens: Vec<&str> = cleaned.split_whitespace().collect();
        assert!(tokens.contains(&"bg-transparent"));
        assert!(tokens.contains(&"border-0"));
        assert!(tokens.contains(&"focus:ring-0"));
        assert!(tokens.contains(&"focus:outline-none"));
        assert!(tokens.contains(&"text-xs"));
        assert!(tokens.contains(&"text-zinc-300"));
        assert!(tokens.contains(&"py-1"));
        assert!(!tokens.contains(&"bg-white"));
        assert!(!tokens.contains(&"rounded-full"));
        assert!(!tokens.contains(&"border-2"));
        assert!(!tokens.contains(&"ring-2"));

        let violations = validate_inline_select_classes(&cleaned, "<select>");
        assert!(violations.is_empty(), "Sanitized dropdown classes should have no violations: {:?}", violations);
    }

    #[test]
    fn test_validate_capsule_with_dropdown() {
        let capsule = conforming_segmented_capsule_classes();
        let select = conforming_inline_select_classes();
        assert!(validate_capsule_with_dropdown(capsule, select).is_ok());

        let bad_capsule = "rounded-none";
        let bad_select = "bg-white border-2";
        let res = validate_capsule_with_dropdown(bad_capsule, bad_select);
        assert!(res.is_err());
        let report = res.unwrap_err();
        assert!(!report.is_clean);
        assert!(!report.capsule_violations.is_empty());
        assert!(!report.dropdown_violations.is_empty());
    }

    #[test]
    fn test_lint_ui_markup_for_capsules_and_dropdowns() {
        let clean_markup = r#"
            <div class="inline-flex items-center rounded-full border border-white/10 bg-zinc-900/60 backdrop-blur-md divide-x divide-white/10 shadow-sm">
                <select className="bg-transparent border-0 focus:ring-0 focus:outline-none text-xs text-zinc-300 py-1 px-2.5">
                    <option value="1">Option 1</option>
                </select>
            </div>
        "#;
        let report = lint_ui_markup_for_capsules_and_dropdowns(clean_markup);
        assert!(report.is_clean);
        assert!(report.capsule_violations.is_empty());
        assert!(report.dropdown_violations.is_empty());

        let dirty_markup = r#"
            <button class="rounded-full w-6 h-6" aria-label="minimize">_</button>
            <select class="bg-white rounded-full border">
                <option value="1">Option 1</option>
            </select>
        "#;
        let report2 = lint_ui_markup_for_capsules_and_dropdowns(dirty_markup);
        assert!(!report2.is_clean);
        assert!(!report2.capsule_violations.is_empty());
        assert!(!report2.dropdown_violations.is_empty());
    }
}
