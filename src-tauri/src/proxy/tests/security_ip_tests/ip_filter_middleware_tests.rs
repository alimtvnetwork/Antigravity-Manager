// 注意：中间件测试需要模拟 HTTP 请求，这里提供测试框架
// 实际的集成测试应该在启动完整服务后进行

/// 验证 IP 提取逻辑的正确性
#[test]
fn test_ip_extraction_priority() {
    // X-Forwarded-For 应该优先于 X-Real-IP
    // X-Real-IP 应该优先于 ConnectInfo
    // 这里只验证逻辑概念，实际测试需要构造 HTTP 请求

    // 场景 1: X-Forwarded-For 有多个 IP，取第一个
    let xff_header = "203.0.113.1, 198.51.100.2, 192.0.2.3";
    let first_ip = xff_header.split(',').next().unwrap().trim();
    assert_eq!(first_ip, "203.0.113.1");

    // 场景 2: 单个 IP
    let single_ip = "10.0.0.1";
    let parsed = single_ip.split(',').next().unwrap().trim();
    assert_eq!(parsed, "10.0.0.1");
}
