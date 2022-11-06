fn main() {
    audio::init(audio::Opt {
        input_device: "default".to_string(),
        output_device: "default".to_string(),
        buffer_length: 2000.0,
    });
}
