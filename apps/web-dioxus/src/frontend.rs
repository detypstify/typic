use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use web_sys::{window, ContextAttributes2d, DomParser, Navigator, SupportedType, SvgsvgElement};

use crate::inference::{inference, process_data, ImageClassifier, MLBackend};
use crate::model::mnist::Model;

#[component]
pub(crate) fn App() -> Element {
    let pos = Point::new();
    let pos_down = pos.clone();
    let pos_enter = pos.clone();
    let pos_move = pos.clone();
    let window = window().expect("Missing Window");
    let navigator = window.navigator();
    let adapter_promise = JsValue::from(Navigator::gpu(&navigator));
    let has_wgpu = !adapter_promise.is_undefined();
    let classifier = if has_wgpu {
        tracing::error!("WGPU");
        let device = Default::default();
        ImageClassifier {
            model: MLBackend::Candle(Box::new(Model::new(&device))),
        }
    } else {
        tracing::error!("Falling back to CPU");
        let device = Default::default();
        ImageClassifier {
            model: MLBackend::NdArray(Box::new(Model::new(&device))),
        }
    };

    rsx! {
        link { rel: "stylesheet", href: "main.css" }
        link {
            rel: "stylesheet",
            href: "https://cdnjs.cloudflare.com/ajax/libs/font-awesome/4.7.0/css/font-awesome.min.css"
        }
        section { class: "container max-w-fit",
            div {
                id: "header",
                class: "flex flex-row sticky items-center justify-center z-10",
                img { src: "typic_logo.webp", class: "w-10 h-10 mr-8 rounded-md" }
                h1 {
                    class: "text-4xl font-bold mb-4",
                    style: "font-family:'0xProto Regular'",
                    "Typic"
                }
                div { class: "flex justify-center ml-6",
                    a {
                        target: "_blank",
                        href: "https://github.com/detypstify/typic",
                        i { class: "fa fa-github" }
                    }
                }
            }
            div { class: "flex align-middle justify-center mb-4",
                canvas {
                    id: "canvas",
                    class: "border border-gray-700 bg-gray-800",
                    width: "400",
                    height: "300",
                    onmousedown: move |evt| {
                        set_position(evt, pos_down.clone());
                    },
                    onmouseenter: move |evt| {
                        set_position(evt, pos_enter.clone());
                    },
                    onmouseup: move |_| {
                        set_output("out1", "1. 0");
                    },
                    onmousemove: move |event| {
                        let classifier_ = classifier.clone();
                        let pos_move_ = pos_move.clone();
                        async move {
                            let need_reprocess = draw(event, pos_move_);
                            let canvas = web_sys::window()
                                .unwrap()
                                .document()
                                .unwrap()
                                .get_element_by_id("canvas")
                                .unwrap();
                            let mut options = ContextAttributes2d::new();
                            options.will_read_frequently(true);
                            let context = canvas
                                .dyn_into::<web_sys::HtmlCanvasElement>()
                                .unwrap()
                                .get_context_with_context_options("2d", &options)
                                .unwrap()
                                .unwrap()
                                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                                .unwrap();
                            if need_reprocess {
                                if let Some(processed_data) = process_data(&context) {
                                    let results = inference(&classifier_, processed_data.as_slice())
                                        .await;
                                    set_output("out1", &results[0].to_string());
                                    set_output("out2", &results[1].to_string());
                                    set_output("out3", &results[2].to_string());
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex align-middle justify-center mb-4",
                div {
                    // make the class be dark and matching with the theme
                    class: "button-like",
                    tabindex: "0",
                    id: "btn",
                    onclick: move |_| {
                        let canvas = web_sys::window()
                            .unwrap()
                            .document()
                            .unwrap()
                            .get_element_by_id("canvas")
                            .unwrap();
                        let mut options = ContextAttributes2d::new();
                        options.will_read_frequently(true);
                        let context = canvas
                            .dyn_into::<web_sys::HtmlCanvasElement>()
                            .unwrap()
                            .get_context_with_context_options("2d", &options)
                            .unwrap()
                            .unwrap()
                            .dyn_into::<web_sys::CanvasRenderingContext2d>()
                            .unwrap();
                        context.clear_rect(0.0, 0.0, 400.0, 300.0);
                        clear_outputs();
                    },
                    style: "font-family:'0xProto Regular",
                    "Clear"
                }
                div {
                    class: "flex justify-center mb-4 button-like",
                    style: "font-family:'0xProto Regular",
                    "Upload Image"
                    input {
                        r#type: "hidden",
                        id: "imageUpload",
                        accept: "image/*",
                        onchange: move |evt| {
                            async move {
                                if let Some(file_engine) = evt.files() {
                                    let files = file_engine.files();
                                    for file_name in &files {
                                        if let Some(bytes) = file_engine.read_file(file_name).await {
                                            let _image = ImageReader::new(Cursor::new(bytes))
                                                .with_guessed_format()
                                                .unwrap()
                                                .decode()
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            OutputModel { name: "out1", num: "1.", formula: "1." }
            OutputModel { name: "out2", num: "2.", formula: "2." }
            OutputModel { name: "out3", num: "3.", formula: "3." }
        }
    }
}

fn draw(event: MouseEvent, pos: Point) -> bool {
    if event.held_buttons().contains(MouseButton::Primary) {
        let coords = event.element_coordinates();
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap();
        let mut options = ContextAttributes2d::new();
        options.will_read_frequently(true);
        let context = canvas
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap()
            .get_context_with_context_options("2d", &options)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        context.begin_path();
        let (x, y) = pos.get_coords();
        context.move_to(x, y);
        set_position(event, pos);

        context.line_to(coords.x, coords.y);
        context.set_stroke_style(&JsValue::from_str("white"));
        context.set_line_width(5.0);
        context.stroke();
        true
    } else {
        false
    }
}

fn mutate_svg(formula: &str, svg_id: &str) {
    let resulting_svg = compiler::set_svg(formula);

    let parser = DomParser::new().unwrap();
    let svg_doc = parser
        .parse_from_string(&resulting_svg, SupportedType::ImageSvgXml)
        .unwrap();
    let svg_doc_ele = svg_doc.document_element().unwrap();

    tracing::error!("svg_doc: {:?}", svg_doc);
    tracing::error!("svg_doc_ele: {:?}", svg_doc_ele);

    let svg = svg_doc_ele.dyn_into::<SvgsvgElement>().unwrap();

    let body = web_sys::window().unwrap().document().unwrap();
    let container = body.get_element_by_id(svg_id).unwrap();
    while let Some(child) = container.first_child() {
        container
            .remove_child(&child)
            .expect("Failed to remove child");
    }

    container.append_child(&svg).unwrap();

    let bbox = svg.get_b_box().unwrap();
    svg.set_attribute(
        "viewBox",
        &format!(
            "{} {} {} {}",
            bbox.x(),
            bbox.y(),
            bbox.width(),
            bbox.height()
        ),
    )
    .unwrap();

    svg.set_attribute("width", "100").unwrap();
    svg.set_attribute("height", "100").unwrap();
}

fn set_output(name: &str, formula: &str) {
    let out_formula = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(&format!("{name}_formula"))
        .unwrap();
    out_formula.set_text_content(Some(formula));
    mutate_svg(formula, &format!("{name}_img"));
}

fn clear_outputs() {
    set_output("out1", "");
    set_output("out2", "");
    set_output("out3", "");
}

#[component]
fn OutputModel(name: String, num: String, formula: String) -> Element {
    rsx! {
        div { class: "flex justify-left",
            h1 { class: "text-2xl font-bold ", "{num}" }
            p { id: "{name}_formula", class: "ml-8 mt-1", "{formula}" }
            div { id: "{name}_img" }
        }
    }
}

#[derive(Debug, Clone)]
struct Point {
    x: Arc<AtomicU64>,
    y: Arc<AtomicU64>,
}

impl Point {
    fn new() -> Self {
        Self {
            x: Arc::default(),
            y: Arc::default(),
        }
    }

    fn get_coords(&self) -> (f64, f64) {
        (
            self.x.load(Ordering::SeqCst) as f64,
            self.y.load(Ordering::SeqCst) as f64,
        )
    }
}

fn set_position(event: MouseEvent, pos: Point) {
    let coords = event.element_coordinates();
    pos.x.store(coords.x as u64, Ordering::SeqCst);
    pos.y.store(coords.y as u64, Ordering::SeqCst);
}
