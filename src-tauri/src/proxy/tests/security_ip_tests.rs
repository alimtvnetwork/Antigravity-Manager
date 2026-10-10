//! IP Security Module Tests
//! IP 安全监控功能的综合测试套件
//!
//! 测试目标:
//! 1. 验证 IP 黑/白名单功能的正确性
//! 2. 验证 CIDR 匹配逻辑
//! 3. 验证过期时间处理
//! 4. 验证不影响主流程性能
//! 5. 验证数据库操作的原子性和一致性

#[cfg(test)]
mod ip_filter_middleware_tests;
#[cfg(test)]
mod performance_benchmarks;
#[cfg(test)]
mod security_db_tests;
