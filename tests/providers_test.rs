use pi_agent_rust::config::AppConfig;
use pi_agent_rust::providers::ProviderRouter;

#[test]
fn test_router_model_aliases() {
    let config = AppConfig::default();
    let router = ProviderRouter::new(&config);

    // Test alias resolutions
    let (p, m) = router.resolve_alias("sonnet");
    assert_eq!(p, "anthropic");
    assert_eq!(m, "claude-3-7-sonnet-latest");

    let (p, m) = router.resolve_alias("4o");
    assert_eq!(p, "openai");
    assert_eq!(m, "gpt-4o");

    let (p, m) = router.resolve_alias("flash");
    assert_eq!(p, "gemini");
    assert_eq!(m, "gemini-2.0-flash");

    let (p, m) = router.resolve_alias("groq");
    assert_eq!(p, "groq");
    assert_eq!(m, "llama-3.3-70b-versatile");

    let (p, m) = router.resolve_alias("free-r1");
    assert_eq!(p, "openrouter");
    assert_eq!(m, "deepseek/deepseek-r1:free");

    let (p, m) = router.resolve_alias("ollama");
    assert_eq!(p, "ollama");
    assert_eq!(m, "llama3.3:latest");
}

#[test]
fn test_model_listing() {
    let config = AppConfig::default();
    let router = ProviderRouter::new(&config);
    let models = router.list_all_models();
    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.is_free));
}
