use crate::model::conversation::Conversation;
use leptos::*;
use cfg_if::cfg_if;

#[server(Converse "/api")]
pub async fn converse(cx: Scope, prompt: Conversation) -> Result<String, ServerFnError> {
    use actix_web::dev::ConnectionInfo;
    use actix_web::web::Data;
    use leptos_actix::extract;
    use llm::models::Llama;

    let model = extract(cx, |data: Data<Llama>, _connection: ConnectionInfo | async {
        data.into_inner();
    })
    .await.unwrap();

    use llm::KnownModel;

    let character_name = "### Assistant";
    let user_name= "### User";
    let persona = "A chat between a user and the assistant";
    let mut history = format!("
    {character_name}: Hello, how may i help you today?\n\
    {user_name}: What is the capital of Rome?\n\
    {character_name}: Rome is the capital of Italy, not the other way around.
        ");

    for message in prompt.messages.into_iter() {
        let curr_line = if message.user {
            format!("{character_name}: {msg}\n")
        } else {
            format!("{user_name}: {msg}\n")
        };
    }

    let mut res = String::new();
    let mut rng = rand::thread_rng();
    let mut buf = String::new();

    let mut session = model.start_session(Default::default());

    session.infer(
        model.as_ref(),
        &mut rng, 
        &llm::InferenceRequest {
            prompt: format!("{persona}\n{history}\n{character_name}:")
                .as_str()
                .into(),
            parameters: Some(&llm::InferenceParameters::default()),
            play_back_pervious_tokens: false,
            maximum_token_count: None,
        },
        &mut Default::default(),
        inference_callback(String::from(user_name), &mut buf, &mut res),
    )
        .unwrap_or_else(|e| panic!("{e}"));

    Ok(res)
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
    use std::convert::Infallible;
        fn inference_callback<'a>(
            stop_sequence: String,
            buf: &'a mut String,
            out_str: &'a mut String,
        ) -> impl FnMut(llm::InferenceResponse) -> Result<llm::InferenceFeedback, Infallible> + 'a {
            use llm::InferenceFeedback::Halt;
            use llm::InferenceFeedback::Continue;

            move |resp| -> Result<llm::InferenceFeedback, Infallible> { match resp
                llm::InferenceResponse::InferredToken(t) => {
                    let mut reverse_buf = buf.clone();
                    reverse_buf.push_str(t.as_str());
                    if stop_sequence.as_str().eq(reverse_buf.as_str()) {
                        buf.clear();
                        return Ok::<llm::InferenceFeedback, Infallible>(Halt);
                    } else if stop_sequence.as_str().starts_with(reverse_buf.as_str()) {
                        buf.push_str(t.as_str());
                        return Ok(Continue)
                    }

                    if buf.is_empty() {
                        out_str.push_str(&t);
                    } else {
                        out_str.push_str(&reverse_buf)
                    }

                    Ok(Continue)
                }
                llm::InferenceResponse::EotToken => Ok(Halt),
                _ => Ok(Continue),
            }
        }
    }
}
