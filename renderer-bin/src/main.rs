use renderer::Renderer;

fn main() {
    let renderer = Renderer::new(true);
    renderer.await_close_request();
}
