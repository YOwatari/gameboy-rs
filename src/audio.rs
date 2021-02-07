use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub fn beep() {
    let device = cpal::default_host().default_output_device().unwrap();
    let config = device.default_output_config().unwrap();
    println!("Output device: {}", device.name().unwrap());
    println!("Default output config: {:?}", config);

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    };
}

pub fn run<T: cpal::Sample>(device: &cpal::Device, config: &cpal::StreamConfig) {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprint!("error {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            err_fn,
        )
        .unwrap();
    stream.play();

    std::thread::sleep(std::time::Duration::from_millis(1000));
}

fn write_data<T: cpal::Sample>(
    output: &mut [T],
    channels: usize,
    next_sample: &mut dyn FnMut() -> f32,
) {
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
