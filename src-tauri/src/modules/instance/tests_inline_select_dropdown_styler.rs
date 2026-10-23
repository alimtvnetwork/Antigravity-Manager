//! Unit Tests for Inline Select Dropdown Styler (`Imp-691`).
//!
//! Validates compliance reporting, forbidden class detection, class normalization,
//! and HTML generation for inline select controls inside segmented capsules.

#[cfg(test)]
mod tests {
    use super::super::inline_select_dropdown_styler::{
        format_capsule_select_html, format_capsule_select_html_with_classes,
        is_forbidden_select_class, normalize_select_classes, validate_select_classes,
        CANONICAL_INLINE_OPTION_CLASSES, CANONICAL_INLINE_SELECT_CLASSES,
    };

    #[test]
    fn test_canonical_select_classes_compliance() {
        let report = validate_select_classes(CANONICAL_INLINE_SELECT_CLASSES);
        assert!(report.is_compliant);
        assert!(report.has_transparent_bg);
        assert!(report.has_zero_border);
        assert!(report.has_zero_focus_ring);
        assert!(report.has_zero_outline);
        assert!(report.forbidden_classes_found.is_empty());
        assert!(report.violations.is_empty());
        assert!(report.summary().contains("Compliant"));
    }

    #[test]
    fn test_missing_transparent_background() {
        let classes = "border-0 focus:ring-0 focus:outline-none text-xs text-zinc-300";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(!report.has_transparent_bg);
        assert!(report.has_zero_border);
        assert!(report.has_zero_focus_ring);
        assert!(report.has_zero_outline);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("bg-transparent")));
    }

    #[test]
    fn test_missing_zero_border() {
        let classes = "bg-transparent focus:ring-0 focus:outline-none text-xs text-zinc-300";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(!report.has_zero_border);
        assert!(report.has_transparent_bg);
        assert!(report.violations.iter().any(|v| v.contains("border-0")));
    }

    #[test]
    fn test_missing_focus_ring_zero() {
        let classes = "bg-transparent border-0 focus:outline-none text-xs text-zinc-300";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(!report.has_zero_focus_ring);
        assert!(report.violations.iter().any(|v| v.contains("focus:ring-0")));
    }

    #[test]
    fn test_missing_focus_outline_none() {
        let classes = "bg-transparent border-0 focus:ring-0 text-xs text-zinc-300";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(!report.has_zero_outline);
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("focus:outline-none")));
    }

    #[test]
    fn test_forbidden_white_background_detected() {
        let classes = "bg-white border-0 focus:ring-0 focus:outline-none text-xs";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(!report.has_transparent_bg);
        assert!(report
            .forbidden_classes_found
            .contains(&"bg-white".to_string()));
        assert!(report
            .violations
            .iter()
            .any(|v| v.contains("nested pill defect")));
    }

    #[test]
    fn test_forbidden_nested_pill_classes_detected() {
        let classes = "bg-transparent border-0 focus:ring-0 focus:outline-none rounded-full rounded-md text-xs";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(report
            .forbidden_classes_found
            .contains(&"rounded-full".to_string()));
        assert!(report
            .forbidden_classes_found
            .contains(&"rounded-md".to_string()));
    }

    #[test]
    fn test_forbidden_borders_and_rings_detected() {
        let classes =
            "bg-transparent border border-zinc-700 focus:ring-2 ring-indigo-500 shadow-md";
        let report = validate_select_classes(classes);
        assert!(!report.is_compliant);
        assert!(report
            .forbidden_classes_found
            .contains(&"border".to_string()));
        assert!(report
            .forbidden_classes_found
            .contains(&"border-zinc-700".to_string()));
        assert!(report
            .forbidden_classes_found
            .contains(&"focus:ring-2".to_string()));
        assert!(report
            .forbidden_classes_found
            .contains(&"shadow-md".to_string()));
    }

    #[test]
    fn test_is_forbidden_select_class_helper() {
        assert!(is_forbidden_select_class("bg-white"));
        assert!(is_forbidden_select_class("bg-zinc-800"));
        assert!(is_forbidden_select_class("rounded-full"));
        assert!(is_forbidden_select_class("border"));
        assert!(is_forbidden_select_class("border-2"));
        assert!(is_forbidden_select_class("focus:ring-2"));
        assert!(is_forbidden_select_class("shadow"));

        assert!(!is_forbidden_select_class("bg-transparent"));
        assert!(!is_forbidden_select_class("border-0"));
        assert!(!is_forbidden_select_class("border-none"));
        assert!(!is_forbidden_select_class("focus:ring-0"));
        assert!(!is_forbidden_select_class("focus:outline-none"));
        assert!(!is_forbidden_select_class("text-xs"));
        assert!(!is_forbidden_select_class("cursor-pointer"));
    }

    #[test]
    fn test_normalize_empty_classes_yields_canonical() {
        let normalized = normalize_select_classes("");
        assert_eq!(normalized, CANONICAL_INLINE_SELECT_CLASSES);

        let report = validate_select_classes(&normalized);
        assert!(report.is_compliant);
    }

    #[test]
    fn test_normalize_dirty_classes_purges_forbidden_and_adds_mandatory() {
        let dirty = "bg-white rounded-full border border-zinc-700 shadow-sm text-xs text-zinc-300 py-0.5 px-1.5";
        let normalized = normalize_select_classes(dirty);

        let report = validate_select_classes(&normalized);
        assert!(report.is_compliant);
        assert!(report.has_transparent_bg);
        assert!(report.has_zero_border);
        assert!(report.has_zero_focus_ring);
        assert!(report.has_zero_outline);
        assert!(report.forbidden_classes_found.is_empty());

        // Custom retain classes are preserved
        assert!(normalized.contains("text-xs"));
        assert!(normalized.contains("text-zinc-300"));
        assert!(normalized.contains("py-0.5"));
        assert!(normalized.contains("px-1.5"));

        // Offending classes are removed
        assert!(!normalized.contains("bg-white"));
        assert!(!normalized.contains("rounded-full"));
        assert!(!normalized.contains("shadow-sm"));
    }

    #[test]
    fn test_normalize_idempotent_on_canonical() {
        let normalized = normalize_select_classes(CANONICAL_INLINE_SELECT_CLASSES);
        let report = validate_select_classes(&normalized);
        assert!(report.is_compliant);
    }

    #[test]
    fn test_format_capsule_select_html_generation() {
        let options = &[
            ("flash", "Gemini 2.5 Flash"),
            ("pro", "Gemini 2.5 Pro"),
            ("ultra", "Gemini 2.5 Ultra"),
        ];

        let html = format_capsule_select_html("model_selector", options, "pro");

        assert!(html.starts_with("<select name=\"model_selector\" class=\"bg-transparent border-0"));
        assert!(html.contains("<option value=\"flash\" class=\"bg-zinc-900 text-zinc-200 text-xs py-1 px-2\">Gemini 2.5 Flash</option>"));
        assert!(html.contains("<option value=\"pro\" class=\"bg-zinc-900 text-zinc-200 text-xs py-1 px-2\" selected>Gemini 2.5 Pro</option>"));
        assert!(html.contains("<option value=\"ultra\" class=\"bg-zinc-900 text-zinc-200 text-xs py-1 px-2\">Gemini 2.5 Ultra</option>"));
        assert!(html.ends_with("</select>"));
    }

    #[test]
    fn test_format_capsule_select_html_escaping() {
        let options = &[("val<1>", "Label with <brackets> & \"quotes\"")];

        let html = format_capsule_select_html("sel&test", options, "val<1>");

        assert!(html.contains("name=\"sel&amp;test\""));
        assert!(html.contains("value=\"val&lt;1&gt;\""));
        assert!(html.contains("Label with &lt;brackets&gt; &amp; &quot;quotes&quot;"));
        assert!(html.contains("selected"));
    }

    #[test]
    fn test_format_capsule_select_html_with_custom_classes() {
        let options = &[("opt1", "Option 1")];
        let html = format_capsule_select_html_with_classes(
            "custom_sel",
            options,
            "opt1",
            "custom-select-cls",
            "custom-opt-cls",
        );

        assert!(html.contains("class=\"custom-select-cls\""));
        assert!(html.contains("class=\"custom-opt-cls\""));
    }
}
