mod dev_db;

// can't use OnceLock because we need to support async
use tokio::sync::OnceCell;
use tracing::info;

/// Initialize env for local dev
/// For early dev, will be called from main()

pub async fn init_dev() {
    static INIT: OnceCell<()> = OnceCell::const_new();

    INIT.get_or_init(|| async {
        info!("{:<12} - init_dev_all", "FOR DEV ONLY");

        dev_db::init_dev_db().await.unwrap(); // we want to break early in case of pb
    })
    .await;
}
