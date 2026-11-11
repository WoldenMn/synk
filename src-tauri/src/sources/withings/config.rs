pub const API_MAX_RETRIES: u32 = 3;
pub const API_BASE_BACKOFF_MS: u64 = 1000;

#[cfg(test)]
mod tests {
    #[test]
    fn the_port_and_the_retry_constants_are_defined_nowhere_else() {
        let sources = [
            ("auth/mod.rs", include_str!("auth/mod.rs")),
            ("auth/commands.rs", include_str!("auth/commands.rs")),
            ("spec.rs", include_str!("spec.rs")),
            ("client/oauth.rs", include_str!("client/oauth.rs")),
            ("client/session.rs", include_str!("client/session.rs")),
        ];
        for (name, source) in sources {
            for forbidden in [
                "17483",
                "const REDIRECT_URI",
                "const MAX_RETRIES",
                "const BASE_BACKOFF_MS",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "{name} definit encore {forbidden}"
                );
            }
        }
    }
}
