use proptest::prelude::*;
use tempfile::tempdir;
use twolebot::config::{Args, Config};
use twolebot::storage::SecretsStore;

fn mk_args(data_dir: std::path::PathBuf) -> Args {
    Args {
        command: None,
        telegram_token: None,
        gemini_key: None,
        data_dir: Some(data_dir),
        memory_dir: None,
        host: "127.0.0.1".to_string(),
        port: 8080,
        cors_allow_all: false,
        claude_model: "claude-opus-4-6".to_string(),
        process_timeout_ms: 600000,
        typing_interval_secs: 4,
        cron_idle_threshold_secs: 600,
        disable_semantic: false,
        no_tunnel: false,
    }
}

fn arb_token() -> impl Strategy<Value = String> {
    prop::string::string_regex("[A-Za-z0-9_.-]{8,96}").unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_cli_keys_override_db_keys(
        cli_tg in arb_token(),
        cli_gm in arb_token(),
        db_tg in arb_token(),
        db_gm in arb_token(),
    ) {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        let store = SecretsStore::new(data_dir.join("runtime.sqlite3")).unwrap();
        store.set_telegram_token(db_tg).unwrap();
        store.set_gemini_key(db_gm).unwrap();

        let mut args = mk_args(data_dir);
        args.telegram_token = Some(cli_tg.clone());
        args.gemini_key = Some(cli_gm.clone());

        let config = Config::from_args(&args).unwrap();

        prop_assert_eq!(config.telegram_token, Some(cli_tg));
        prop_assert_eq!(config.gemini_key, Some(cli_gm));
    }
}
