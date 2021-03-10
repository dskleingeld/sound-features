use rodio::{Decoder, Source};
use itertools::Itertools;
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use audio_feature::band_energy;

fn main() {
    let f = File::open("data/piano.wav").unwrap();
    let mut reader = BufReader::new(f);
    let decoder = Decoder::new(reader).unwrap();

    let bands = [0..1000, 1..2000, 
        1000..2000, 2000..3000, 
        3000..4000, 5000..6000];
    type Builder = band_energy::Builder::<6, 512>;
    let mut eng = Builder::new(bands, decoder.sample_rate())
        .build();

    let chunks = &decoder.chunks(512);
    let energies: Vec<_> = chunks.into_iter()
        .map(|chunk| eng.process_iter(chunk))
        .collect();
    dbg!(energies);
}
