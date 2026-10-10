use super::*;

#[test]
fn test_sync_jeikcode_toml_content() {
    let initial_toml = r#"default_model = "old-model"
default_provider = "old-model"
language = "zh-CN"

[provider_accounts.local-gemini]
api_key = "sk-old1"
base_url = "http://127.0.0.1:8046/v1"
provider = "openai"

[provider_accounts."local_antigravity测试"]
api_key = "sk-old2"
base_url = "http://127.0.0.1:8046"
provider = "anthropic"

[provider_accounts.other-provider]
api_key = "sk-other"
base_url = "https://other.com/v1"
provider = "openai"

[models."gemini-3.7"]
account = "local-gemini"
model = "gemini-3.7-flash-high"
reasoning_model = false

[models."other-model"]
account = "other-provider"
model = "deepseek-chat"
"#;

    let res = sync_jeikcode_toml_content(
        initial_toml,
        "http://127.0.0.1:8046",
        "sk-newkey123",
        Some("gemini-3.8-flash-high"),
    )
    .expect("sync should succeed");

    use toml_edit::DocumentMut;
    let doc: DocumentMut = res.parse().expect("result should be valid toml");

    // 验证 antigravity-manager 存在且正确
    let accounts = doc.get("provider_accounts").unwrap().as_table().unwrap();
    let ag = accounts.get("antigravity-manager").unwrap();
    assert_eq!(ag.get("provider").unwrap().as_str().unwrap(), "anthropic");
    assert_eq!(
        ag.get("base_url").unwrap().as_str().unwrap(),
        "http://127.0.0.1:8046/v1"
    );
    assert_eq!(ag.get("api_key").unwrap().as_str().unwrap(), "sk-newkey123");

    // 验证同 URL 旧账号被清除
    assert!(!accounts.contains_key("local-gemini"));
    assert!(!accounts.contains_key("local_antigravity测试"));
    // 验证不同 URL 账号被保留
    assert!(accounts.contains_key("other-provider"));

    // 验证核心模型已添加
    let models = doc.get("models").unwrap().as_table().unwrap();
    assert!(models.contains_key("claude-sonnet-4-6"));
    assert!(models.contains_key("claude-sonnet-4-6-thinking"));
    assert!(models.contains_key("claude-opus-4-6"));
    assert!(models.contains_key("gemini-3.8-flash"));
    assert!(models.contains_key("gemini-3.8-flash-tiered"));
    assert!(models.contains_key("gemini-3.8-flash-high"));
    assert!(models.contains_key("gemini-3.7-flash"));
    assert!(models.contains_key("gemini-3.1-pro"));

    // 验证旧关联模型已被清除，且保留其他账号模型
    assert!(!models.contains_key("gemini-3.7"));
    assert!(models.contains_key("other-model"));

    // 验证模型属性：开启思考模式和思考不回传，以及上下文大小（Gemini 1M，Claude 256k）
    let m = models.get("gemini-3.8-flash-high").unwrap();
    assert_eq!(
        m.get("account").unwrap().as_str().unwrap(),
        "antigravity-manager"
    );
    assert_eq!(
        m.get("context_window").unwrap().as_integer().unwrap(),
        1_000_000
    );
    assert_eq!(m.get("reasoning_model").unwrap().as_bool().unwrap(), true);
    assert_eq!(
        m.get("reasoning_history").unwrap().as_str().unwrap(),
        "exclude"
    );
    assert_eq!(m.get("image_input").unwrap().as_bool().unwrap(), true);

    let m_claude = models.get("claude-sonnet-4-6").unwrap();
    assert_eq!(
        m_claude.get("account").unwrap().as_str().unwrap(),
        "antigravity-manager"
    );
    assert_eq!(
        m_claude
            .get("context_window")
            .unwrap()
            .as_integer()
            .unwrap(),
        256_000
    );
    assert_eq!(
        m_claude.get("reasoning_model").unwrap().as_bool().unwrap(),
        true
    );
    assert_eq!(
        m_claude.get("reasoning_history").unwrap().as_str().unwrap(),
        "exclude"
    );

    // 验证默认模型
    assert_eq!(
        doc.get("default_model").unwrap().as_str().unwrap(),
        "gemini-3.8-flash-high"
    );
}
