use nih_plug::prelude::*;
use std::sync::Arc;

// This is a shortened version of the gain example with most comments removed, check out
// https://github.com/robbert-vdh/nih-plug/blob/master/plugins/examples/gain/src/lib.rs to get
// started

struct Dontpanic {
    params: Arc<DontpanicParams>,
}

#[derive(Params)]
struct DontpanicParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "gain"]
    pub crush: FloatParam,
}

impl Default for Dontpanic {
    fn default() -> Self {
        Self {
            params: Arc::new(DontpanicParams::default()),
        }
    }
}

impl Default for DontpanicParams {
    fn default() -> Self {
        Self {
            // This gain is stored as linear gain. NIH-plug comes with useful conversion functions
            // to treat these kinds of parameters as if we were dealing with decibels. Storing this
            // as decibels is easier to work with, but requires a conversion for every sample.
            crush: FloatParam::new(
                "Crush",
                0.0,
                FloatRange::Linear {
                    min: 20.0,
                    max: 50.0,
                },
            )
            // Because the gain parameter is stored as linear gain instead of storing the value as
            // decibels, we need logarithmic smoothing
            .with_smoother(SmoothingStyle::Linear(50.0)),
        }
    }
}

impl Plugin for Dontpanic {
    const NAME: &'static str = "cool bitcrusher";
    const VENDOR: &'static str = "mayk";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "jamie@pond.audio";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    //
    // from plain byte buffers.
    type SysExMessage = ();

    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let crush = self.params.crush.smoothed.next();

            let mut channel = 0;

            for sample in channel_samples {
                let mut channel_crush = crush;
                // left
                if channel == 0 {
                    channel_crush *= 1.05;
                }

                let x = *sample;
                let x = x * channel_crush;
                let x = x.round();
                let x = x / channel_crush;

                let x = x.abs();
                let x = x.tanh();

                *sample = x;

                channel += 1;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for Dontpanic {
    const CLAP_ID: &'static str = "audio.pond.dontpanic";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("some stupid bit crushing, probably");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for Dontpanic {
    const VST3_CLASS_ID: [u8; 16] = *b"audioponddontpan";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(Dontpanic);
nih_export_vst3!(Dontpanic);
