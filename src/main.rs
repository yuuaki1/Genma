mod app;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use llm::models::Llama;
        use actix_web::*;
        use std::env;
        use dotenv::dotenv;

        fn get_language_model() -> Llama {
            use std::path::PathBuf;
            dotenv().ok();
            let model_path = env::var("MODEL_PATH").expect("MODEL_PATH must be set");

            llm::load::<Llama>(
                &PathBuf::from(&model_path),
                llm::TokenizerSource::Embeded,
                Default::default(),
                llm::load_progress_callback_stdout,
            )
                .unwrap_or_else(|err| {
                    panic!("Failed to load model from {model_path:?}: {err}")
                })
        }
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use actix_web::*;
        use leptos::*;
        use crate::app::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};

        #[get("/style.css")]
        async fn css() -> impl Responder {
            actix_files::NamedFile::open_async("./style/output.css").await
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            let conf = get_configuration(None).await.unwrap();

            let addr = conf.leptos_options.site_addr.clone();

            // Generate the list of routes in your Leptos App
            let routes = generate_route_list(|cx| view! { cx, <App/> });

            let model = web::Data::new(get_language_model());

            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let routes = &routes;
                App::new()
                    .app_data(model.clone())
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    .service(css)
                    .leptos_routes(leptos_options.to_owned(), routes.to_owned(), |cx| view! { cx, <App/> })
                    .service(Files::new("/", &site_root))
                    .wrap(middleware::Compress::default())
            })
            .bind(&addr)?
            .run()
            .await
        }
    }
    else {
        pub fn main() {}
    }
}
