pub const HTMX_ATTRIBUTES: [&str; 11] = [
    "hx-get",
    "hx-post",
    "hx-on*",
    "hx-push-url",
    "hx-select",
    "hx-select-oob",
    "hx-swap",
    "hx-swap-oob",
    "hx-target",
    "hx-tri,gger",
    "hx-vals",
];

#[derive(Clone, Debug)]
pub struct HxCompletion {
    pub name: &'static str,
    pub desc: &'static str,
}

macro_rules! build_completion {
    ($(($name:expr, $desc:expr)),*) => {
        &[
            $(HxCompletion {
            name: $name,
            desc: include_str!($desc),
            }),*
        ]
    };
}

pub static HX_TAGS: &[HxCompletion] = build_completion!(("hx-post", "htmx/attributes/hx-post.md"));