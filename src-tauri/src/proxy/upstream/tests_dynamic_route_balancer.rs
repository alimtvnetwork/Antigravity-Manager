#[cfg(test)]
mod tests {
    use super::dynamic_route_balancer::DynamicRouteBalancer;

    #[test]
    fn test_unregistered_route_returns_base_weight() {
        let balancer = DynamicRouteBalancer::new(100);
        assert_eq!(balancer.get_weight("https://unknown-host.com"), 100);
    }

    #[test]
    fn test_register_route_sets_initial_weight() {
        let balancer = DynamicRouteBalancer::new(100);
        balancer.register_route("https://api.openai.com");
        assert_eq!(balancer.get_weight("https://api.openai.com"), 100);
    }

    #[test]
    fn test_failure_halves_weight() {
        let balancer = DynamicRouteBalancer::new(100);
        balancer.register_route("https://failing-route.com");

        // 100 -> 50 on first failure
        let w1 = balancer.record_result("https://failing-route.com", false);
        assert_eq!(w1, 50);
        assert_eq!(balancer.get_weight("https://failing-route.com"), 50);

        // 50 -> 25 on second failure
        let w2 = balancer.record_result("https://failing-route.com", false);
        assert_eq!(w2, 25);
    }

    #[test]
    fn test_weight_clamps_to_minimum_floor() {
        let balancer = DynamicRouteBalancer::new(100);
        balancer.register_route("https://persistently-failing.com");

        for _ in 0..10 {
            balancer.record_result("https://persistently-failing.com", false);
        }

        // Weight should clamp to minimum 5
        assert_eq!(balancer.get_weight("https://persistently-failing.com"), 5);
    }

    #[test]
    fn test_success_gradually_restores_weight() {
        let balancer = DynamicRouteBalancer::new(100);
        balancer.register_route("https://recovering-route.com");

        balancer.record_result("https://recovering-route.com", false); // 50
        assert_eq!(balancer.get_weight("https://recovering-route.com"), 50);

        // Success: (50 * 9 + 100) / 10 = (450 + 100) / 10 = 55
        let w_rec1 = balancer.record_result("https://recovering-route.com", true);
        assert_eq!(w_rec1, 55);

        // Success: (55 * 9 + 100) / 10 = (495 + 100) / 10 = 59
        let w_rec2 = balancer.record_result("https://recovering-route.com", true);
        assert_eq!(w_rec2, 59);
    }

    #[test]
    fn test_independent_route_isolation() {
        let balancer = DynamicRouteBalancer::new(100);
        balancer.register_route("https://route-a.com");
        balancer.register_route("https://route-b.com");

        balancer.record_result("https://route-a.com", false);
        assert_eq!(balancer.get_weight("https://route-a.com"), 50);
        assert_eq!(balancer.get_weight("https://route-b.com"), 100);
    }
}
