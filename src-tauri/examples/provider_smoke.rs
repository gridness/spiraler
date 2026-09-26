use spiraler_lib::{
    domain::*,
    provider::{CodexProvider, ImageProvider},
    storage::Store,
};
use std::sync::{atomic::AtomicBool, Arc};

#[tokio::main]
async fn main() {
    let root = std::env::args()
        .nth(1)
        .expect("Pass an empty scratch directory");
    let mut store = Store::open(root.into()).expect("open storage");
    let mut project = Project::new(
        "Woodland study".into(),
        "Provider acceptance test".into(),
        "game prop".into(),
    )
    .unwrap();
    project.style.direction = "A handcrafted stop-motion woodland prop, rounded matte clay, warm moss-green and ochre palette, subtle handmade texture, calm studio lighting. A single object centered against a plain warm ivory background. No text, no border, no labels.".into();
    let request = GenerationRequest::build(
        &project.style,
        "A tiny red-capped mushroom house with a round wooden door",
        &project.asset_type,
        None,
    );
    store.put(project.clone()).unwrap();
    let dir = store.dir(&project.id).unwrap();
    let result = CodexProvider
        .generate(
            &request,
            &dir,
            &dir.join("jobs/smoke"),
            Arc::new(AtomicBool::new(false)),
        )
        .await;
    match result {
        Ok(path) => println!("Generated and validated image: {}", path.display()),
        Err(error) => {
            eprintln!("Provider error: {}", error.message);
            std::process::exit(1);
        }
    }
}
