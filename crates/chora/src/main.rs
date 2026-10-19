//! The chora Wayland compositor.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    chora::run(&args)
}
