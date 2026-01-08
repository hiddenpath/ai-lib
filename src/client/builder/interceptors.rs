use super::AiClientBuilder;

impl AiClientBuilder {
    #[cfg(feature = "interceptors")]
    pub fn with_interceptor_pipeline(
        mut self,
        pipeline: crate::interceptors::InterceptorPipeline,
    ) -> Self {
        self.interceptor_pipeline = Some(pipeline);
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_default_interceptors(mut self) -> Self {
        let p = crate::interceptors::create_default_interceptors();
        self.interceptor_pipeline = Some(p);
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_minimal_interceptors(mut self) -> Self {
        let p = crate::interceptors::default::DefaultInterceptorsBuilder::new()
            .enable_circuit_breaker(false)
            .enable_rate_limit(false)
            .build();
        self.interceptor_pipeline = Some(p);
        self
    }

    // --- Interceptor Configuration Proxies ---

    #[cfg(feature = "interceptors")]
    pub fn with_rate_limit(mut self, requests_per_minute: u32) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_rate_limit(requests_per_minute));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_circuit_breaker(mut self, threshold: u32, recovery: std::time::Duration) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_circuit_breaker(threshold, recovery));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_retry(
        mut self,
        max_attempts: u32,
        base_delay: std::time::Duration,
        max_delay: std::time::Duration,
    ) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_retry(max_attempts, base_delay, max_delay));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn with_interceptor_timeout(mut self, duration: std::time::Duration) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.with_timeout(duration));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_retry(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_retry(enable));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_circuit_breaker(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_circuit_breaker(enable));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_rate_limit(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_rate_limit(enable));
        self
    }

    #[cfg(feature = "interceptors")]
    pub fn enable_interceptor_timeout(mut self, enable: bool) -> Self {
        let builder = self.interceptor_builder.unwrap_or_default();
        self.interceptor_builder = Some(builder.enable_timeout(enable));
        self
    }
}
