//! Choose a profile picture: one of the built-in characters, a photo taken
//! with the camera, or an image from the device. Shown right after signing
//! up (with a "skip" option) and from the Profile page.

use codesprout_core::api::AVATAR_PRESETS;
use leptos::*;
use crate::components::icons::IconBack;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::api;
use crate::components::avatar::avatar_src;
use crate::i18n;
use crate::session::use_session;

#[wasm_bindgen(inline_js = r#"
export function read_picture(input) {
  const file = input && input.files && input.files[0];
  input.value = '';
  if (!file) return Promise.resolve(null);
  return new Promise((resolve) => {
    const url = URL.createObjectURL(file);
    const img = new Image();
    img.onload = () => {
      // Center-crop to a square and scale to 256 px, then compress as JPEG.
      const size = 256, side = Math.min(img.naturalWidth, img.naturalHeight);
      const sx = (img.naturalWidth - side) / 2, sy = (img.naturalHeight - side) / 2;
      const canvas = document.createElement('canvas');
      canvas.width = size; canvas.height = size;
      canvas.getContext('2d').drawImage(img, sx, sy, side, side, 0, 0, size, size);
      URL.revokeObjectURL(url);
      resolve(canvas.toDataURL('image/jpeg', 0.85));
    };
    img.onerror = () => { URL.revokeObjectURL(url); resolve(null); };
    img.src = url;
  });
}
"#)]
extern "C" {
    fn read_picture(input: JsValue) -> js_sys::Promise;
}

#[component]
pub fn AvatarPage() -> impl IntoView {
    let session = use_session();
    let navigate = use_navigate();
    let query = use_query_map();
    let first_time = move || query.with(|q| q.get("first").is_some());

    let current = session.profile.with_untracked(|p| p.as_ref().and_then(|p| p.avatar.clone()));
    let chosen = create_rw_signal(current);
    let busy = create_rw_signal(false);
    let error = create_rw_signal(None::<String>);

    let camera_ref = create_node_ref::<html::Input>();
    let file_ref = create_node_ref::<html::Input>();

    let on_file = move |ev: ev::Event| {
        let Some(target) = ev.target() else { return };
        let promise = read_picture(target.into());
        spawn_local(async move {
            match JsFuture::from(promise).await.ok().and_then(|v| v.as_string()) {
                Some(data) => {
                    error.set(None);
                    chosen.set(Some(data));
                }
                None => error.set(Some(i18n::t(session.lang.get_untracked(), "avatar.bad_image").to_string())),
            }
        });
    };

    let done = {
        let navigate = navigate.clone();
        move || {
            if first_time() {
                navigate("/", Default::default());
            } else {
                navigate("/profile", Default::default());
            }
        }
    };

    let save = {
        let done = done.clone();
        move |_| {
            if busy.get_untracked() {
                return;
            }
            busy.set(true);
            error.set(None);
            let token = session.token.get_untracked();
            let avatar = chosen.get_untracked();
            let lang = session.lang.get_untracked();
            let done = done.clone();
            spawn_local(async move {
                let result = api::set_avatar(token, avatar).await;
                busy.set(false);
                match result {
                    Ok(p) => {
                        session.profile.set(Some(p));
                        done();
                    }
                    Err(e) => error.set(Some(i18n::error(lang, &e.message))),
                }
            });
        }
    };

    let preview = move || {
        chosen.with(|c| match c.as_deref().and_then(avatar_src) {
            Some(src) => view! { <img class="avatar-preview" src=src alt=""/> }.into_view(),
            None => {
                let initial = session.profile.with(|p| {
                    p.as_ref().and_then(|p| p.username.chars().next()).unwrap_or('?').to_uppercase().to_string()
                });
                view! { <div class="avatar-preview avatar">{initial}</div> }.into_view()
            }
        })
    };

    view! {
        {move || (!session.logged_in()).then(|| view! { <Redirect path="/signup"/> })}
        <div class="auth avatar-page">
            {move || (!first_time()).then(|| view! { <a href="/profile" class="back-link"><IconBack/>{session.t("back.word")}</a> })}
            <h1 class="auth-title">{session.t("avatar.title")}</h1>
            <p class="avatar-sub">{session.t("avatar.sub")}</p>

            <div class="avatar-stage">{preview}</div>

            <div class="avatar-grid">
                {AVATAR_PRESETS.iter().map(|name| {
                    let value = format!("preset:{name}");
                    let src = avatar_src(&value).unwrap_or_default();
                    let is_on = {
                        let value = value.clone();
                        move || chosen.with(|c| c.as_deref() == Some(value.as_str()))
                    };
                    view! {
                        <button class="avatar-choice" class:selected=is_on on:click=move |_| chosen.set(Some(value.clone()))>
                            <img src=src alt=""/>
                        </button>
                    }
                }).collect_view()}
            </div>

            <div class="avatar-upload">
                <button class="btn btn-ghost btn-block" on:click=move |_| { if let Some(i) = camera_ref.get() { i.click(); } }>
                    "📷 " {session.t("avatar.camera")}
                </button>
                <button class="btn btn-ghost btn-block" on:click=move |_| { if let Some(i) = file_ref.get() { i.click(); } }>
                    "🖼️ " {session.t("avatar.upload")}
                </button>
                <input node_ref=camera_ref type="file" accept="image/*" capture="user" class="hidden-input" on:change=on_file/>
                <input node_ref=file_ref type="file" accept="image/*" class="hidden-input" on:change=on_file/>
            </div>

            {move || error.get().map(|e| view! { <div class="form-error">{e}</div> })}

            <button class="btn btn-primary btn-block" disabled=move || busy.get() on:click=save>
                {move || if busy.get() { session.t("auth.wait")() } else { session.t("avatar.save")() }}
            </button>
            <button class="link-btn avatar-skip" on:click={
                let done = done.clone();
                move |_| done()
            }>
                {move || if first_time() { session.t("avatar.skip")() } else { session.t("avatar.cancel")() }}
            </button>
        </div>
    }
}
